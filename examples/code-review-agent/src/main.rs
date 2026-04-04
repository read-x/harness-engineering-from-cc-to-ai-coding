#![warn(clippy::all)]

//! Code Review Agent — a custom agent loop with pluggable LLM backends.
//!
//! Supports two LLM backends:
//! - **cc-sdk** (default): Routes through Claude Code subscription via LLM Proxy
//! - **codex**: Routes through Codex Responses API Proxy (OpenAI-compatible)
//!
//! Can run as CLI tool or MCP Server (`--serve`).

mod agent;
mod context;
mod llm;
mod mcp;
mod prompts;
mod resilience;
mod review;
mod tools;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use tracing::info;

use agent::ReviewConfig;
use llm::{CcSdkBackend, CcSdkWsBackend, CodexBackend, CodexWsBackend};


/// A mini Code Review Agent with a custom agent loop.
#[derive(Parser, Debug)]
#[command(name = "code-review-agent", version, about)]
struct Cli {
    /// Run as MCP server on stdio (for Claude Code integration).
    #[arg(long, default_value_t = false)]
    serve: bool,

    /// Path to a unified diff file to review (CLI mode).
    #[arg(long, required_unless_present = "serve")]
    diff: Option<PathBuf>,

    /// Total token budget for the review session.
    #[arg(long, default_value_t = 50_000)]
    max_tokens: usize,

    /// Per-file token budget.
    #[arg(long, default_value_t = 5_000)]
    max_file_tokens: usize,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output_format: OutputFormat,

    /// Which LLM backend to use.
    #[arg(long, value_enum, default_value_t = Backend::CcSdk)]
    backend: Backend,

    /// Codex Responses API proxy URL (required for codex/codex-ws backends).
    #[arg(long)]
    codex_url: Option<String>,

    /// Model name for Codex backend (default: o3).
    #[arg(long, default_value = "o3")]
    codex_model: String,

    /// WebSocket URL for cc-sdk-ws backend (e.g., ws://localhost:8080/ws/cli/session).
    #[arg(long)]
    ws_url: Option<String>,

    /// Auth token for WebSocket backends.
    #[arg(long)]
    auth_token: Option<String>,
}

/// Output format for the review report.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    Markdown,
}

/// LLM backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Backend {
    /// cc-sdk LLM Proxy (subprocess, CC subscription auth)
    CcSdk,
    /// cc-sdk WebSocket (persistent connection to remote CC instance)
    CcSdkWs,
    /// Codex Responses API Proxy (HTTP POST, OpenAI-compatible)
    Codex,
    /// Codex app-server WebSocket (JSON-RPC, persistent connection)
    CodexWs,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (MCP mode logs to stderr to keep stdout for JSON-RPC)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("code_review_agent=info")),
        )
        .with_target(true)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    // --- MCP Server mode ---
    if cli.serve {
        return mcp::serve().await;
    }

    // --- CLI mode ---
    let diff_path = cli.diff.as_ref().context("--diff is required in CLI mode")?;

    info!(
        diff = %diff_path.display(),
        max_tokens = cli.max_tokens,
        max_file_tokens = cli.max_file_tokens,
        backend = ?cli.backend,
        output_format = ?cli.output_format,
        "review_started"
    );

    // Build the LLM backend
    let backend: Box<dyn llm::LlmBackend> = match cli.backend {
        Backend::CcSdk => Box::new(CcSdkBackend),
        Backend::CcSdkWs => Box::new(CcSdkWsBackend::new(
            cli.ws_url
                .as_deref()
                .context("--ws-url is required with --backend cc-sdk-ws")?,
            cli.auth_token.clone(),
        )),
        Backend::Codex => {
            if let Some(url) = cli.codex_url.as_deref() {
                // Proxy mode: --codex-url http://127.0.0.1:PORT
                Box::new(CodexBackend::new(url, cli.auth_token.clone(), Some(cli.codex_model.as_str())))
            } else {
                // Direct mode: read token from ~/.codex/auth.json (Codex subscription)
                Box::new(CodexBackend::from_codex_auth(Some(cli.codex_model.as_str()))?)
            }
        }
        Backend::CodexWs => Box::new(CodexWsBackend::new(
            cli.ws_url
                .as_deref()
                .or(cli.codex_url.as_deref())
                .context("--ws-url or --codex-url is required with --backend codex-ws")?,
            cli.auth_token.clone(),
            Some(cli.codex_model.as_str()),
        )),
    };

    let config = ReviewConfig {
        max_tokens: cli.max_tokens,
        max_file_tokens: cli.max_file_tokens,
        ..Default::default()
    };

    // Run the agent loop
    let report = agent::run_review(diff_path, &config, backend.as_ref()).await?;

    info!(summary = %report.summary_line(), "review_completed");

    // Format and output
    let output = match cli.output_format {
        OutputFormat::Markdown => report.to_markdown(),
        OutputFormat::Json => report.to_json().context("Failed to serialize report")?,
    };

    println!("{output}");

    Ok(())
}
