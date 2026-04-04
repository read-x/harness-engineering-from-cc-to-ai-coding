//! Shared Agent Loop — the single implementation of the review cycle.
//!
//! Both CLI (`main.rs`) and MCP Server (`mcp.rs`) delegate to `run_review()`.
//! The loop is parameterized over `&dyn LlmBackend` so the LLM provider
//! can be swapped without changing any review logic.

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::context::{apply_budget, load_diff, ContextBudget, FileChange};
use crate::llm::{LlmBackend, TokenUsage};
use crate::prompts::{build_followup_prompt, build_system_prompt, PrInfo};
use crate::resilience::{CircuitBreaker, RetryConfig, with_retry};
use crate::review::{parse_agent_action, parse_findings_from_response, AgentAction, Finding, ReviewReport};
use crate::tools::{self, ToolConfig};

/// Configuration for a review session.
pub struct ReviewConfig {
    pub max_tokens: usize,
    pub max_file_tokens: usize,
    /// Maximum depth for cross-file follow-up (0 = no follow-up, 1 = one hop).
    pub max_related_depth: usize,
    /// Maximum number of tool calls per file (prevents runaway loops).
    pub max_tool_calls: usize,
    pub retry_config: RetryConfig,
    pub circuit_breaker_threshold: u32,
    pub tool_config: ToolConfig,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            max_tokens: 50_000,
            max_file_tokens: 5_000,
            max_related_depth: 1,
            max_tool_calls: 3,
            retry_config: RetryConfig {
                max_retries: 2,
                base_delay_ms: 2000,
            },
            circuit_breaker_threshold: 3,
            tool_config: ToolConfig::default(),
        }
    }
}

/// Run a complete code review.
///
/// This is the shared Agent Loop used by both CLI and MCP modes.
pub async fn run_review(
    diff_path: &Path,
    config: &ReviewConfig,
    llm: &(dyn LlmBackend + '_),
) -> Result<ReviewReport> {
    // Step 1: Load diff
    let diff_context = load_diff(diff_path).context("Failed to load diff")?;
    info!(files = diff_context.files.len(), "Loaded diff");

    // Step 2: Apply budget
    let mut budget = ContextBudget::new(config.max_tokens, config.max_file_tokens);
    let (constrained_diff, files_skipped) = apply_budget(&diff_context, &mut budget);

    let files_to_review = constrained_diff.files.len();
    info!(
        files_to_review,
        files_skipped,
        tokens_used = budget.used_tokens,
        "Budget applied"
    );

    // Step 3: Build system prompt (shared across all file reviews)
    let pr_info = PrInfo::from_diff_context(&constrained_diff);
    let system_prompt = build_system_prompt(&pr_info);

    // Step 4: Agent Loop — review each file with optional follow-up
    let start = Instant::now();
    let circuit_breaker = CircuitBreaker::new(config.circuit_breaker_threshold);

    let mut all_findings: Vec<Finding> = Vec::new();
    let mut files_reviewed: usize = 0;
    let mut files_failed: usize = 0;
    let mut total_usage = TokenUsage::default();

    // Build index for cross-file lookups
    let file_index: HashMap<&str, &FileChange> = constrained_diff
        .files
        .iter()
        .map(|f| (f.path.as_str(), f))
        .collect();

    let sp = system_prompt.clone();

    for file in &constrained_diff.files {
        if !circuit_breaker.check() {
            warn!(
                file = %file.path,
                "Circuit breaker OPEN — skipping remaining files"
            );
            break;
        }

        info!(file = %file.path, tokens = file.estimated_tokens, "Reviewing file");

        let result = review_file_with_followup(
            file,
            &file_index,
            &sp,
            llm,
            &config.retry_config,
            config.max_related_depth,
            config.max_tool_calls,
            &config.tool_config,
            &mut total_usage,
        )
        .await;

        match result {
            Ok(findings) => {
                circuit_breaker.record_success();
                info!(
                    file = %file.path,
                    findings = findings.len(),
                    "File review complete"
                );
                all_findings.extend(findings);
                files_reviewed += 1;
            }
            Err(e) => {
                warn!(file = %file.path, error = %e, "File review failed");
                circuit_breaker.record_failure();
                files_failed += 1;
            }
        }
    }

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    Ok(ReviewReport {
        files_reviewed,
        files_skipped: files_skipped + files_failed,
        total_tokens_used: total_usage.total(),
        duration_ms,
        findings: all_findings,
        cost_usd: None,
    })
}

/// Review a single file, with optional follow-up on related files and tool use.
///
/// Agent Loop per file:
///   Turn 1: Review the file's diff → findings
///   Turn 2: Decide next action → AgentAction (Done | ReviewRelated | UseTool)
///   Turn 3+: Execute tool and feed result back, or review related file
async fn review_file_with_followup(
    file: &FileChange,
    file_index: &HashMap<&str, &FileChange>,
    system_prompt: &str,
    llm: &(dyn LlmBackend + '_),
    retry_config: &RetryConfig,
    max_depth: usize,
    max_tool_calls: usize,
    tool_config: &ToolConfig,
    usage: &mut TokenUsage,
) -> Result<Vec<Finding>> {
    // Turn 1: Review the file's diff
    let user_prompt = format!(
        "Review this diff for `{}`:\n\n```diff\n{}\n```",
        file.path, file.diff
    );

    let response = with_retry(retry_config, || {
        let sp = system_prompt.to_string();
        let up = user_prompt.clone();
        async move { llm.complete(&sp, &up).await }
    })
    .await?;

    usage.accumulate(&response.usage);
    let mut findings = parse_findings_from_response(&response.text);

    // Turn 2+: Agent decision loop — the LLM can request tools or related file review.
    // Each iteration: ask for next action → execute → feed result back.
    // Capped at max_tool_calls to prevent runaway loops.
    let mut tool_calls_used = 0;
    let mut context_addendum = String::new(); // accumulated tool results

    if max_depth > 0 || max_tool_calls > 0 {
        let available_files: Vec<&str> = file_index
            .keys()
            .filter(|&&k| k != file.path)
            .copied()
            .collect();

        loop {
            let decision_prompt = if context_addendum.is_empty() {
                build_followup_prompt(&file.path, &findings, &available_files)
            } else {
                format!(
                    "{}\n\n--- Tool Results ---\n{}\n\nBased on these results, what's your next action?",
                    build_followup_prompt(&file.path, &findings, &available_files),
                    context_addendum
                )
            };

            let decision_result = with_retry(retry_config, || {
                let sp = system_prompt.to_string();
                let dp = decision_prompt.clone();
                async move { llm.complete(&sp, &dp).await }
            })
            .await;

            match decision_result {
                Ok(decision_response) => {
                    usage.accumulate(&decision_response.usage);

                    match parse_agent_action(&decision_response.text) {
                        Some(AgentAction::Done) | None => break,

                        Some(AgentAction::ReviewRelated { file: related, reason }) => {
                            if max_depth == 0 { break; }
                            if let Some(related_file) = file_index.get(related.as_str()) {
                                info!(
                                    from = %file.path, related = %related,
                                    reason = %reason, "Agent: review related file"
                                );
                                let related_prompt = format!(
                                    "You found issues in `{}`. Now review `{}`.\n\
                                     Focus on cross-file interactions.\n\n```diff\n{}\n```",
                                    file.path, related_file.path, related_file.diff
                                );
                                if let Ok(resp) = with_retry(retry_config, || {
                                    let sp = system_prompt.to_string();
                                    let rp = related_prompt.clone();
                                    async move { llm.complete(&sp, &rp).await }
                                }).await {
                                    usage.accumulate(&resp.usage);
                                    findings.extend(parse_findings_from_response(&resp.text));
                                }
                            }
                            break; // Only one related file per review
                        }

                        Some(AgentAction::UseTool { tool, input, reason }) => {
                            if tool_calls_used >= max_tool_calls {
                                info!(file = %file.path, "Tool call limit reached");
                                break;
                            }
                            info!(
                                file = %file.path, tool = %tool,
                                reason = %reason, "Agent: using tool"
                            );

                            if tool == "skill" {
                                // Skill tool: load the skill's system prompt and run
                                // a specialized review of the current file.
                                let skill_name = input.split_whitespace().next().unwrap_or("");
                                if let Some(skill) = tools::find_skill(skill_name) {
                                    info!(skill = skill.name, "Agent: running skill analysis");
                                    let skill_prompt = format!(
                                        "Analyze this code:\n\n```diff\n{}\n```",
                                        file.diff
                                    );
                                    if let Ok(resp) = with_retry(retry_config, || {
                                        let sp = skill.system_prompt.to_string();
                                        let up = skill_prompt.clone();
                                        async move { llm.complete(&sp, &up).await }
                                    }).await {
                                        usage.accumulate(&resp.usage);
                                        let skill_findings = parse_findings_from_response(&resp.text);
                                        info!(
                                            skill = skill.name,
                                            findings = skill_findings.len(),
                                            "Skill analysis complete"
                                        );
                                        findings.extend(skill_findings);
                                    }
                                } else {
                                    let available = tools::list_skills()
                                        .iter()
                                        .map(|(n, d)| format!("{n}: {d}"))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    context_addendum.push_str(&format!(
                                        "\n[Skill '{skill_name}' not found. Available: {available}]\n"
                                    ));
                                }
                            } else {
                                // Bash tool: execute and feed result back
                                let result = tools::execute_tool(&tool, &input, tool_config).await;
                                context_addendum.push_str(&format!(
                                    "\n[Tool: {} | Success: {}]\n{}\n",
                                    result.tool, result.success, result.output
                                ));
                            }
                            tool_calls_used += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!(file = %file.path, error = %e, "Follow-up decision call failed");
                    break;
                }
            }
        }
    }

    Ok(findings)
}
