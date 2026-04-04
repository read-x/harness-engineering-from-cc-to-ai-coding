//! MCP Server mode — exposes the code review agent as a tool for Claude Code.
//!
//! When started with `--serve`, the agent runs as an MCP server on stdio,
//! allowing Claude Code (or any MCP client) to call `review_diff` as a tool.

use std::path::PathBuf;

use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use tracing::info;

use crate::agent::{self, ReviewConfig};
use crate::llm::CcSdkBackend;
use crate::review::ReviewReport;

/// Input schema for the `review_diff` tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReviewDiffRequest {
    /// Path to a unified diff file to review.
    #[schemars(description = "Absolute path to a unified diff file (from git diff)")]
    pub diff_path: String,

    /// Total token budget (default: 50000).
    #[schemars(description = "Maximum total tokens across all files (default: 50000)")]
    pub max_tokens: Option<usize>,

    /// Per-file token budget (default: 5000).
    #[schemars(description = "Maximum tokens per file (default: 5000)")]
    pub max_file_tokens: Option<usize>,
}

/// The MCP server wrapping our code review agent.
#[derive(Debug, Clone)]
pub struct ReviewServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ReviewServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Review a unified diff file and return structured findings.
    #[tool(description = "Review a unified diff file for bugs, security issues, and code quality. Returns structured JSON findings with file, line, severity, and suggestions.")]
    async fn review_diff(
        &self,
        Parameters(req): Parameters<ReviewDiffRequest>,
    ) -> String {
        match self.do_review(req).await {
            Ok(report) => {
                serde_json::to_string_pretty(&report).unwrap_or_else(|e| {
                    serde_json::json!({"error": format!("Failed to serialize report: {e}")})
                        .to_string()
                })
            }
            Err(e) => serde_json::json!({"error": format!("{e}")}).to_string(),
        }
    }
}

impl ReviewServer {
    async fn do_review(&self, req: ReviewDiffRequest) -> anyhow::Result<ReviewReport> {
        let diff_path = PathBuf::from(&req.diff_path);

        // Validate the diff path
        if !diff_path.is_absolute() {
            anyhow::bail!("diff_path must be an absolute path, got: {}", diff_path.display());
        }
        if !diff_path.exists() {
            anyhow::bail!("diff file does not exist: {}", diff_path.display());
        }
        if !diff_path.is_file() {
            anyhow::bail!("diff path is not a regular file: {}", diff_path.display());
        }

        // Validate budget parameters
        let max_tokens = req.max_tokens.unwrap_or(50_000);
        let max_file_tokens = req.max_file_tokens.unwrap_or(5_000);
        if max_tokens == 0 || max_file_tokens == 0 {
            anyhow::bail!("token budgets must be > 0 (got max_tokens={max_tokens}, max_file_tokens={max_file_tokens})");
        }

        let config = ReviewConfig {
            max_tokens,
            max_file_tokens,
            ..Default::default()
        };

        // MCP mode always uses cc-sdk (it runs inside Claude Code)
        let llm = CcSdkBackend;
        agent::run_review(&diff_path, &config, &llm).await
    }
}

#[tool_handler]
impl ServerHandler for ReviewServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "Code Review Agent: reviews unified diffs for bugs, security issues, and code quality."
                    .to_string(),
            )
    }
}

/// Start the MCP server on stdio transport.
pub async fn serve() -> anyhow::Result<()> {
    info!("Starting MCP server on stdio");

    let service = ReviewServer::new()
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("MCP server error: {e:?}"))?;

    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("MCP server wait error: {e:?}"))?;

    Ok(())
}
