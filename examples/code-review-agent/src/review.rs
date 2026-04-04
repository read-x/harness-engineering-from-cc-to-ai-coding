//! Review output structures and response parsing.
//!
//! Defines the structured review report, individual findings,
//! severity levels, and serialization to JSON / Markdown.

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Agent decision types (used by the Agent Loop in agent.rs)
// ---------------------------------------------------------------------------

/// What the agent decides to do after reviewing a file.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action")]
pub enum AgentAction {
    /// Done reviewing this file. Findings are final.
    #[serde(rename = "done")]
    Done,
    /// Request to also review another file from the changeset.
    #[serde(rename = "review_related")]
    ReviewRelated {
        file: String,
        reason: String,
    },
    /// Request to run a tool (bash or skill) and continue with the result.
    #[serde(rename = "use_tool")]
    UseTool {
        tool: String,
        input: String,
        reason: String,
    },
}

/// Parse an agent action from the LLM's text response.
///
/// The LLM is instructed to output JSON like `{"action": "done"}` or
/// `{"action": "review_related", "file": "path", "reason": "..."}`.
pub fn parse_agent_action(text: &str) -> Option<AgentAction> {
    // Try direct parse first
    if let Ok(action) = serde_json::from_str::<AgentAction>(text) {
        return Some(action);
    }

    // Try to extract JSON object from surrounding text — scan for each `{`
    let mut search_from = 0;
    while let Some(start) = text[search_from..].find('{') {
        let start = search_from + start;
        let candidate = &text[start..];
        let mut stream =
            serde_json::Deserializer::from_str(candidate).into_iter::<AgentAction>();
        if let Some(Ok(action)) = stream.next() {
            return Some(action);
        }
        search_from = start + 1;
    }

    debug!("Could not parse agent action from response");
    None
}

// ---------------------------------------------------------------------------
// Review findings and reports
// ---------------------------------------------------------------------------

/// Severity level for a finding.
///
/// Accepts both title-case ("Critical") and lowercase ("critical") from LLM output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl<'de> serde::Deserialize<'de> for Severity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "critical" => Ok(Severity::Critical),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            _ => Err(serde::de::Error::unknown_variant(&s, &["Critical", "Warning", "Info"])),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::Warning => write!(f, "Warning"),
            Severity::Info => write!(f, "Info"),
        }
    }
}

/// A single review finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// File path the finding applies to.
    pub file: String,
    /// Line number (if applicable).
    pub line: Option<u32>,
    /// Severity level.
    pub severity: Severity,
    /// Category (e.g., "bug", "security", "performance", "maintainability", "style").
    pub category: String,
    /// Human-readable description of the issue.
    pub message: String,
    /// Optional concrete suggestion for a fix.
    pub suggestion: Option<String>,
}

/// Aggregate review report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    /// Number of files that were reviewed.
    pub files_reviewed: usize,
    /// Number of files skipped due to budget constraints.
    pub files_skipped: usize,
    /// Total tokens consumed (from the result message, if available).
    pub total_tokens_used: u64,
    /// Wall-clock duration of the review in milliseconds.
    pub duration_ms: u64,
    /// All findings extracted from the model's response.
    pub findings: Vec<Finding>,
    /// Estimated cost in USD (from the SDK result message, if available).
    pub cost_usd: Option<f64>,
}

impl ReviewReport {
    /// Serialize the report to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Render the report as a Markdown document.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Code Review Report\n\n");
        md.push_str(&format!(
            "- **Files reviewed:** {}\n",
            self.files_reviewed
        ));
        md.push_str(&format!("- **Files skipped:** {}\n", self.files_skipped));
        md.push_str(&format!(
            "- **Total tokens used:** {}\n",
            self.total_tokens_used
        ));
        md.push_str(&format!(
            "- **Duration:** {:.1}s\n",
            self.duration_ms as f64 / 1000.0
        ));

        if let Some(cost) = self.cost_usd {
            md.push_str(&format!("- **Estimated cost:** ${cost:.4}\n"));
        }

        md.push_str(&format!(
            "- **Findings:** {} total",
            self.findings.len()
        ));

        let critical = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count();
        let warnings = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count();
        let info = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .count();

        md.push_str(&format!(
            " ({critical} critical, {warnings} warnings, {info} info)\n\n"
        ));

        if self.findings.is_empty() {
            md.push_str("No issues found.\n");
            return md;
        }

        // Group by severity
        for severity in &[Severity::Critical, Severity::Warning, Severity::Info] {
            let group: Vec<_> = self
                .findings
                .iter()
                .filter(|f| &f.severity == severity)
                .collect();

            if group.is_empty() {
                continue;
            }

            md.push_str(&format!("## {severity}\n\n"));

            for finding in &group {
                let location = match finding.line {
                    Some(line) => format!("{}:{line}", finding.file),
                    None => finding.file.clone(),
                };
                md.push_str(&format!(
                    "### [{category}] {location}\n\n",
                    category = finding.category
                ));
                md.push_str(&format!("{}\n\n", finding.message));

                if let Some(suggestion) = &finding.suggestion {
                    md.push_str(&format!("**Suggestion:** {suggestion}\n\n"));
                }
            }
        }

        md
    }

    /// Summary counts for display.
    pub fn summary_line(&self) -> String {
        let critical = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count();
        let warnings = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count();
        let info = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .count();

        format!(
            "{} findings ({} critical, {} warnings, {} info) across {} files in {:.1}s",
            self.findings.len(),
            critical,
            warnings,
            info,
            self.files_reviewed,
            self.duration_ms as f64 / 1000.0,
        )
    }
}

/// Parse findings from the model's text response.
///
/// The model is instructed to output a raw JSON array of finding objects.
/// This function attempts to extract that array even if surrounded by
/// markdown fences or extra text.
pub fn parse_findings_from_response(text: &str) -> Vec<Finding> {
    debug!(
        response_length = text.len(),
        "Parsing findings from model response"
    );

    // Try direct parse first
    if let Ok(findings) = serde_json::from_str::<Vec<Finding>>(text) {
        debug!(count = findings.len(), "Parsed findings directly");
        return findings;
    }

    // Try to extract JSON array from surrounding text / markdown fences
    if let Some(findings) = extract_json_array(text) {
        return findings;
    }

    warn!("Could not parse any findings from model response");
    Vec::new()
}

/// Attempt to find and parse a JSON array within arbitrary text.
///
/// Instead of bracket-matching (which breaks on brackets inside JSON strings),
/// we try `serde_json::from_str` starting from each `[` position. The first
/// successful parse wins.
fn extract_json_array(text: &str) -> Option<Vec<Finding>> {
    let mut search_from = 0;

    while let Some(start) = text[search_from..].find('[') {
        let start = search_from + start;
        let candidate = &text[start..];

        // Use serde_json's streaming deserializer to find how far a valid
        // JSON array extends. This correctly handles brackets inside strings.
        let mut stream = serde_json::Deserializer::from_str(candidate).into_iter::<Vec<Finding>>();
        if let Some(Ok(findings)) = stream.next() {
            debug!(
                count = findings.len(),
                "Extracted findings from embedded JSON array"
            );
            return Some(findings);
        }

        // This `[` didn't start a valid array — try the next one.
        search_from = start + 1;
    }

    warn!("No valid JSON array found in model response");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_finding() -> Finding {
        Finding {
            file: "src/main.rs".to_string(),
            line: Some(42),
            severity: Severity::Warning,
            category: "bug".to_string(),
            message: "Potential null dereference".to_string(),
            suggestion: Some("Add a None check before unwrapping".to_string()),
        }
    }

    #[test]
    fn test_report_to_json_roundtrip() {
        let report = ReviewReport {
            files_reviewed: 3,
            files_skipped: 1,
            total_tokens_used: 5000,
            duration_ms: 12345,
            findings: vec![sample_finding()],
            cost_usd: Some(0.0123),
        };

        let json = report.to_json().expect("serialize");
        let parsed: ReviewReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.findings.len(), 1);
        assert_eq!(parsed.files_reviewed, 3);
    }

    #[test]
    fn test_report_to_markdown_contains_sections() {
        let report = ReviewReport {
            files_reviewed: 2,
            files_skipped: 0,
            total_tokens_used: 3000,
            duration_ms: 5000,
            findings: vec![sample_finding()],
            cost_usd: None,
        };

        let md = report.to_markdown();
        assert!(md.contains("# Code Review Report"));
        assert!(md.contains("Warning"));
        assert!(md.contains("src/main.rs:42"));
    }

    #[test]
    fn test_parse_findings_direct_json() {
        let json = r#"[{"file":"a.rs","line":1,"severity":"Critical","category":"bug","message":"oops","suggestion":null}]"#;
        let findings = parse_findings_from_response(json);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_parse_findings_with_markdown_fence() {
        let text = "Here are the findings:\n```json\n[{\"file\":\"b.rs\",\"line\":10,\"severity\":\"Info\",\"category\":\"style\",\"message\":\"naming\",\"suggestion\":null}]\n```\n";
        let findings = parse_findings_from_response(text);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn test_parse_findings_empty_array() {
        let findings = parse_findings_from_response("[]");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_findings_garbage_returns_empty() {
        let findings = parse_findings_from_response("no json here at all");
        assert!(findings.is_empty());
    }

    // --- AgentAction tests ---

    #[test]
    fn test_parse_agent_action_done() {
        let action = parse_agent_action(r#"{"action": "done"}"#);
        assert!(matches!(action, Some(AgentAction::Done)));
    }

    #[test]
    fn test_parse_agent_action_review_related() {
        let action = parse_agent_action(
            r#"{"action": "review_related", "file": "src/lib.rs", "reason": "shared types"}"#,
        );
        match action {
            Some(AgentAction::ReviewRelated { file, reason }) => {
                assert_eq!(file, "src/lib.rs");
                assert_eq!(reason, "shared types");
            }
            other => panic!("expected ReviewRelated, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_agent_action_from_surrounding_text() {
        let text = r#"Based on the findings, I recommend: {"action": "review_related", "file": "src/utils.rs", "reason": "caller"} since it calls the changed function."#;
        let action = parse_agent_action(text);
        assert!(matches!(action, Some(AgentAction::ReviewRelated { .. })));
    }

    #[test]
    fn test_parse_agent_action_garbage_returns_none() {
        let action = parse_agent_action("I think we're done here, no issues.");
        assert!(action.is_none());
    }
}
