//! Context management for the code review agent.
//!
//! Handles loading unified diffs, parsing them into per-file chunks,
//! and managing token budgets with explicit truncation.

use std::path::Path;

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// Token budget configuration for the review session.
pub struct ContextBudget {
    /// Maximum total tokens across all files.
    pub max_total_tokens: usize,
    /// Maximum tokens for a single file's diff content.
    pub max_file_tokens: usize,
    /// Tokens consumed so far.
    pub used_tokens: usize,
}

impl ContextBudget {
    /// Create a new budget with the given limits.
    pub fn new(max_total_tokens: usize, max_file_tokens: usize) -> Self {
        Self {
            max_total_tokens,
            max_file_tokens,
            used_tokens: 0,
        }
    }

    /// Returns the remaining token budget.
    pub fn remaining(&self) -> usize {
        self.max_total_tokens.saturating_sub(self.used_tokens)
    }

    /// Try to consume tokens. Returns true if within budget, false otherwise.
    pub fn try_consume(&mut self, tokens: usize) -> bool {
        match self.used_tokens.checked_add(tokens) {
            Some(sum) if sum <= self.max_total_tokens => {
                self.used_tokens = sum;
                true
            }
            _ => false,
        }
    }
}

/// A parsed unified diff broken into per-file chunks.
#[derive(Debug, Clone)]
pub struct DiffContext {
    /// Per-file change records.
    pub files: Vec<FileChange>,
}

/// A single file's diff content.
#[derive(Debug, Clone)]
pub struct FileChange {
    /// File path (relative).
    pub path: String,
    /// The raw unified diff text for this file.
    pub diff: String,
    /// Estimated token count for the diff content.
    pub estimated_tokens: usize,
}

/// Estimate token count from text using a conservative bytes/4 heuristic.
///
/// Uses byte length (`str::len()`), not character count. For ASCII-heavy code
/// this approximates chars/4. For multi-byte UTF-8 content (e.g., CJK), this
/// overestimates, which is intentionally conservative.
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

/// Truncate file content to fit within a token budget.
///
/// Returns `(truncated_content, was_truncated)`. If truncated, a metadata
/// annotation is appended indicating how much was cut.
pub fn truncate_file_content(content: &str, max_tokens: usize) -> (String, bool) {
    let estimated = estimate_tokens(content);
    if estimated <= max_tokens {
        return (content.to_string(), false);
    }

    // Approximate character budget from token budget
    let char_budget = max_tokens * 4;

    let mut truncated = String::with_capacity(char_budget + 100);
    let mut chars_used = 0;
    let mut lines_shown = 0;
    let mut total_lines = 0;
    let mut truncation_point_reached = false;

    for line in content.lines() {
        total_lines += 1;
        if !truncation_point_reached {
            let line_len = line.len() + 1; // +1 for newline
            if chars_used + line_len > char_budget {
                truncation_point_reached = true;
            } else {
                truncated.push_str(line);
                truncated.push('\n');
                chars_used += line_len;
                lines_shown += 1;
            }
        }
    }

    truncated.push_str(&format!(
        "\n[Truncated: full file has {total_lines} lines, showing first {lines_shown}]"
    ));

    (truncated, true)
}

/// Load a unified diff file and parse it into per-file chunks.
///
/// Expects standard `diff --git` or `--- a/` / `+++ b/` headers.
pub fn load_diff(path: &Path) -> Result<DiffContext> {
    info!(path = %path.display(), "Loading diff file");

    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read diff file: {}", path.display()))?;

    debug!(
        bytes = raw.len(),
        estimated_tokens = estimate_tokens(&raw),
        "Read diff file"
    );

    let files = parse_unified_diff(&raw);

    info!(
        file_count = files.len(),
        "Parsed diff into per-file chunks"
    );

    Ok(DiffContext { files })
}

/// Parse a unified diff string into per-file `FileChange` records.
///
/// Extracts file paths from `+++ b/path` lines (more reliable than parsing
/// `diff --git` headers, which are ambiguous when paths contain spaces).
fn parse_unified_diff(raw: &str) -> Vec<FileChange> {
    let mut files = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();
    // Track whether we've seen a `diff --git` boundary to know when to flush.
    let mut in_file_section = false;

    for line in raw.lines() {
        // Detect file boundary: `diff --git a/path b/path`
        if line.starts_with("diff --git ") {
            // Flush previous file
            if let Some(path) = current_path.take() {
                let diff_text = current_lines.join("\n");
                let tokens = estimate_tokens(&diff_text);
                files.push(FileChange {
                    path,
                    diff: diff_text,
                    estimated_tokens: tokens,
                });
                current_lines.clear();
            }
            in_file_section = true;
            current_lines.push(line);
        } else if line.starts_with("+++ ") {
            // Extract path from `+++ b/path` — this is the authoritative source,
            // more reliable than parsing `diff --git` which is ambiguous with spaces.
            // Skip /dev/null (deleted files) — use --- header instead.
            let raw = line.strip_prefix("+++ ").unwrap_or("");
            let raw = raw.strip_prefix("b/").unwrap_or(raw);
            if current_path.is_none() && raw != "/dev/null" && !raw.is_empty() {
                current_path = Some(raw.to_string());
            }
            current_lines.push(line);
        } else if in_file_section || current_path.is_some() {
            current_lines.push(line);
        } else {
            // Preamble lines before any file section — collect them so they
            // attach to the first file when its path is identified.
            current_lines.push(line);
        }
    }

    // Flush last file
    if let Some(path) = current_path.take() {
        let diff_text = current_lines.join("\n");
        let tokens = estimate_tokens(&diff_text);
        files.push(FileChange {
            path,
            diff: diff_text,
            estimated_tokens: tokens,
        });
    }

    files
}

/// Apply budget constraints to a diff context: truncate or skip files as needed.
///
/// Returns `(budget_constrained_context, files_skipped_count)`.
pub fn apply_budget(diff: &DiffContext, budget: &mut ContextBudget) -> (DiffContext, usize) {
    let mut files = Vec::new();
    let mut skipped = 0;

    for file in &diff.files {
        if budget.remaining() == 0 {
            warn!(
                file = %file.path,
                "Skipping file — total token budget exhausted"
            );
            skipped += 1;
            continue;
        }

        // Reserve ~20 tokens for the truncation metadata annotation so it
        // doesn't eat into the content budget.
        let metadata_overhead = 20;
        let effective_max = budget.max_file_tokens.min(budget.remaining()).saturating_sub(metadata_overhead);
        let (content, was_truncated) = truncate_file_content(&file.diff, effective_max);

        if was_truncated {
            debug!(
                file = %file.path,
                original_tokens = file.estimated_tokens,
                truncated_to = effective_max,
                "Truncated file diff"
            );
        }

        let tokens = estimate_tokens(&content);
        if !budget.try_consume(tokens) {
            warn!(
                file = %file.path,
                "Skipping file — would exceed total token budget"
            );
            skipped += 1;
            continue;
        }

        files.push(FileChange {
            path: file.path.clone(),
            diff: content,
            estimated_tokens: tokens,
        });
    }

    (DiffContext { files }, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        // 5 chars → (5+3)/4 = 2
        assert_eq!(estimate_tokens("abcde"), 2);
    }

    #[test]
    fn test_truncate_no_op_when_within_budget() {
        let content = "short content";
        let (result, truncated) = truncate_file_content(content, 1000);
        assert_eq!(result, content);
        assert!(!truncated);
    }

    #[test]
    fn test_truncate_appends_metadata() {
        let content = "line1\nline2\nline3\nline4\nline5\n";
        let (result, truncated) = truncate_file_content(content, 2);
        assert!(truncated);
        assert!(result.contains("[Truncated:"));
        assert!(result.contains("5 lines"));
    }

    #[test]
    fn test_parse_git_diff() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
index abc..def 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!(\"hello\");
 }
diff --git a/src/lib.rs b/src/lib.rs
index 111..222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1 +1,2 @@
+pub mod utils;
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[1].path, "src/lib.rs");
    }

    #[test]
    fn test_budget_skips_when_exhausted() {
        // Create multi-line content so truncation keeps many lines.
        let big_content = (0..200).map(|i| format!("line {i}: some code here")).collect::<Vec<_>>().join("\n");
        let tokens = estimate_tokens(&big_content); // ~1200 tokens
        let diff = DiffContext {
            files: vec![
                FileChange {
                    path: "a.rs".to_string(),
                    diff: big_content.clone(),
                    estimated_tokens: tokens,
                },
                FileChange {
                    path: "b.rs".to_string(),
                    diff: big_content,
                    estimated_tokens: tokens,
                },
            ],
        };
        // Total budget smaller than one file — truncation will fill most of it.
        let mut budget = ContextBudget::new(tokens / 2, tokens);
        let (result, skipped) = apply_budget(&diff, &mut budget);
        assert_eq!(result.files.len(), 1, "only one file should fit in half-budget");
        assert_eq!(skipped, 1);
    }
}
