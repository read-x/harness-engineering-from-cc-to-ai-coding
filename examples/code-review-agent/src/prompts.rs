//! System prompt construction for the code review agent.
//!
//! Implements a two-layer prompt architecture:
//! - Static "constitution" layer: review principles, severity levels, output format
//! - Dynamic "runtime" layer: PR metadata, changed file list, language-specific rules

use crate::context::DiffContext;

/// Metadata about the pull request or change set being reviewed.
pub struct PrInfo {
    /// Title or summary of the change (e.g., commit message first line).
    pub title: String,
    /// List of changed file paths.
    pub changed_files: Vec<String>,
}

impl PrInfo {
    /// Construct `PrInfo` from a parsed diff context.
    pub fn from_diff_context(diff: &DiffContext) -> Self {
        let changed_files: Vec<String> = diff.files.iter().map(|f| f.path.clone()).collect();
        let title = if changed_files.len() == 1 {
            format!("Review changes in {}", changed_files[0])
        } else {
            format!("Review changes across {} files", changed_files.len())
        };
        Self {
            title,
            changed_files,
        }
    }
}

/// Build the full system prompt from PR metadata.
///
/// The prompt has two sections:
/// 1. **Constitution** (static): review principles, severity definitions, output format spec.
/// 2. **Runtime** (dynamic): PR title, changed file list, inferred language rules.
pub fn build_system_prompt(pr_info: &PrInfo) -> String {
    let constitution = build_constitution();
    let runtime = build_runtime_section(pr_info);
    format!("{constitution}\n\n---\n\n{runtime}")
}

/// Static constitution section: review principles and output format.
fn build_constitution() -> String {
    r#"# Code Review Agent — Constitution

You are a code review agent. Your job is to review diffs and produce a structured list of findings.

## Review Principles

1. **Correctness first**: Flag logic errors, off-by-one bugs, null/undefined risks, and race conditions.
2. **Security**: Identify injection vulnerabilities, credential leaks, insecure defaults, and missing input validation.
3. **Maintainability**: Note overly complex code, missing error handling, unclear naming, and code duplication.
4. **Performance**: Highlight unnecessary allocations, O(n²) patterns in hot paths, and missing indices.
5. **Be specific**: Always reference the exact file and line number. Provide a concrete suggestion when possible.
6. **No false praise**: Do not add compliments or filler. Only report actionable findings.

## Severity Levels

- **Critical**: Bugs that will cause data loss, security vulnerabilities, or crashes in production.
- **Warning**: Code smells, potential bugs under edge cases, or violations of best practices that should be fixed before merge.
- **Info**: Style suggestions, minor improvements, or questions for the author to consider.

## Output Format

You MUST output a JSON array of finding objects. Each object has these fields:

```json
{
  "file": "path/to/file.ext",
  "line": 42,
  "severity": "Critical" | "Warning" | "Info",
  "category": "bug" | "security" | "performance" | "maintainability" | "style",
  "message": "Description of the issue",
  "suggestion": "Optional concrete fix or improvement"
}
```

If there are no findings, output an empty array: `[]`

IMPORTANT: Output ONLY the JSON array. Do not wrap it in markdown code fences. Do not add any text before or after the array."#
        .to_string()
}

/// Dynamic runtime section: PR metadata and language-specific hints.
fn build_runtime_section(pr_info: &PrInfo) -> String {
    let file_list = pr_info
        .changed_files
        .iter()
        .map(|f| format!("  - {f}"))
        .collect::<Vec<_>>()
        .join("\n");

    let language_rules = infer_language_rules(&pr_info.changed_files);

    format!(
        r#"# Runtime Context

## Change Summary
**Title:** {title}
**Files changed:** {count}

{file_list}

{language_rules}"#,
        title = pr_info.title,
        count = pr_info.changed_files.len(),
    )
}

/// Infer language-specific review rules from file extensions.
fn infer_language_rules(files: &[String]) -> String {
    let mut rules = Vec::new();
    let mut seen_rust = false;
    let mut seen_ts = false;
    let mut seen_py = false;

    for file in files {
        if !seen_rust && file.ends_with(".rs") {
            seen_rust = true;
            rules.push(
                "## Rust-Specific Rules\n\
                 - Check for `.unwrap()` in non-test code.\n\
                 - Flag `unsafe` blocks that lack a `// SAFETY:` comment.\n\
                 - Watch for unnecessary `.clone()` calls.",
            );
        }
        if !seen_ts && (file.ends_with(".ts") || file.ends_with(".tsx")) {
            seen_ts = true;
            rules.push(
                "## TypeScript-Specific Rules\n\
                 - Flag `any` type usage.\n\
                 - Check for missing `await` on async calls.\n\
                 - Verify error handling in `try/catch` blocks.",
            );
        }
        if !seen_py && file.ends_with(".py") {
            seen_py = true;
            rules.push(
                "## Python-Specific Rules\n\
                 - Flag bare `except:` clauses.\n\
                 - Check for mutable default arguments.\n\
                 - Verify type hints on public function signatures.",
            );
        }
    }

    if rules.is_empty() {
        String::new()
    } else {
        rules.join("\n\n")
    }
}

/// Build a follow-up prompt asking the LLM whether to review a related file.
///
/// The LLM should respond with JSON: `{"action": "done"}` or
/// `{"action": "review_related", "file": "path/to/file", "reason": "..."}`.
pub fn build_followup_prompt(
    reviewed_file: &str,
    findings: &[crate::review::Finding],
    available_files: &[&str],
) -> String {
    let findings_summary: String = findings
        .iter()
        .take(5) // limit to top 5 to save tokens
        .map(|f| format!("  - [{:?}] {}: {}", f.severity, f.file, f.message))
        .collect::<Vec<_>>()
        .join("\n");

    let files_list = available_files
        .iter()
        .map(|f| format!("  - {f}"))
        .collect::<Vec<_>>()
        .join("\n");

    let related_files_section = if available_files.is_empty() {
        "No other files in the changeset.".to_string()
    } else {
        format!(
            "The changeset also includes these other files:\n\n{files_list}"
        )
    };

    let review_related_option = if available_files.is_empty() {
        String::new()
    } else {
        "\n2. Review a related file from the changeset:\n   \
         {{\"action\": \"review_related\", \"file\": \"<path>\", \"reason\": \"<why>\"}}\n"
            .to_string()
    };

    format!(
        r#"You just reviewed `{reviewed_file}` and found these issues:

{findings_summary}

{related_files_section}

What should you do next? You can use read-only bash commands to get more context (e.g., look up a function definition, search for callers, check a config file).

Respond with ONLY a JSON object, no other text. Choose ONE action:

1. No follow-up needed:
   {{"action": "done"}}
{review_related_option}
3. Run a read-only bash command to get more context:
   {{"action": "use_tool", "tool": "bash", "input": "<command>", "reason": "<why>"}}
   Allowed commands: cat, grep, find, head, tail, wc, ls, sort, uniq, awk, sed, etc.
   Example: {{"action": "use_tool", "tool": "bash", "input": "grep -rn 'MAX_RETRIES' src/", "reason": "check the constant value"}}

4. Run a specialized analysis skill on the current file:
   {{"action": "use_tool", "tool": "skill", "input": "<skill_name>", "reason": "<why>"}}
   Available skills: security-audit, rust-deep, performance-review, api-review, test-coverage"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt_contains_constitution_and_runtime() {
        let pr_info = PrInfo {
            title: "Fix null pointer in parser".to_string(),
            changed_files: vec!["src/parser.rs".to_string(), "src/lib.rs".to_string()],
        };
        let prompt = build_system_prompt(&pr_info);
        assert!(prompt.contains("Code Review Agent"));
        assert!(prompt.contains("Fix null pointer in parser"));
        assert!(prompt.contains("src/parser.rs"));
        assert!(prompt.contains("Rust-Specific Rules"));
    }

    #[test]
    fn test_language_detection_typescript() {
        let files = vec!["app/index.tsx".to_string()];
        let rules = infer_language_rules(&files);
        assert!(rules.contains("TypeScript"));
    }

    #[test]
    fn test_no_language_rules_for_unknown() {
        let files = vec!["Makefile".to_string()];
        let rules = infer_language_rules(&files);
        assert!(rules.is_empty());
    }

    #[test]
    fn test_build_followup_prompt_contains_findings_and_files() {
        let findings = vec![crate::review::Finding {
            file: "src/main.rs".to_string(),
            line: Some(10),
            severity: crate::review::Severity::Warning,
            category: "bug".to_string(),
            message: "potential null deref".to_string(),
            suggestion: None,
        }];
        let available = vec!["src/lib.rs", "src/utils.rs"];
        let prompt = build_followup_prompt("src/main.rs", &findings, &available);
        assert!(prompt.contains("src/main.rs"));
        assert!(prompt.contains("potential null deref"));
        assert!(prompt.contains("src/lib.rs"));
        assert!(prompt.contains("src/utils.rs"));
        assert!(prompt.contains("review_related"));
    }
}
