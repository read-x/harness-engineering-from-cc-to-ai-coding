# Code Review Agent

A **Rust AI Agent** with custom agent loop, pluggable LLM backends, and tool system. Reads a Git diff, reviews each file with multi-turn reasoning, and outputs a structured report.

Supports **4 LLM backends** (Claude via cc-sdk, GPT via Codex subscription, plus WebSocket variants), **2 tool types** (bash sandbox + skill analysis), and runs as **CLI**, **MCP Server**, or via **Claude Code Skill**.

Demo project for the book *驾驭工程 — Claude Code 源码分析*.

## Prerequisites

- **Rust** (edition 2024 / rustc 1.85+)
- **At least one** LLM backend:
  - **cc-sdk** (default): Claude Code CLI installed and authenticated (`claude` on PATH)
  - **Codex**: Codex CLI authenticated (`codex login` completed)

## Quick Start

```bash
# Generate a diff
git diff > /tmp/my.diff

# Review with Claude (default)
cargo run -- --diff /tmp/my.diff

# Review with Codex/GPT
cargo run -- --diff /tmp/my.diff --backend codex --codex-model gpt-5.4

# Output as Markdown
cargo run -- --diff /tmp/my.diff --output-format markdown
```

## LLM Backends

| Backend | Flag | Auth | Transport |
|---------|------|------|-----------|
| **cc-sdk** (default) | `--backend cc-sdk` | CC subscription | Subprocess |
| **cc-sdk WebSocket** | `--backend cc-sdk-ws --ws-url ws://...` | Bearer token | WebSocket |
| **Codex** | `--backend codex` | `~/.codex/auth.json` (auto) | HTTP SSE |
| **Codex WebSocket** | `--backend codex-ws --ws-url ws://...` | Bearer token | WebSocket |

```bash
# Codex with custom model
cargo run -- --diff my.diff --backend codex --codex-model gpt-5.4

# Codex via local proxy
cargo run -- --diff my.diff --backend codex --codex-url http://127.0.0.1:8080

# cc-sdk via WebSocket to remote instance
cargo run -- --diff my.diff --backend cc-sdk-ws --ws-url ws://remote:8080/ws/cli/session --auth-token TOKEN
```

## Agent Loop

Each file is reviewed with a multi-turn agent loop:

```
Turn 1: Review diff → findings (JSON)
Turn 2: Decide next action:
  → done                          (stop)
  → review_related { file }       (check a related file in the changeset)
  → use_tool { bash, "grep ..." } (run read-only bash command)
  → use_tool { skill, "security-audit" } (run specialized analysis)
Turn 3+: Tool results fed back, loop continues (max 3 tool calls/file)
```

The LLM requests actions via JSON (`AgentAction`), our Rust code validates and executes them.

### Tools

| Tool | What it does | Safety |
|------|-------------|--------|
| **bash** | Read-only commands (`cat`, `grep`, `find`, `head`, `wc`, ...) | Whitelist + blacklist + no redirects + 30s timeout + 50KB output cap |
| **skill** | Specialized analysis prompts sent to current LLM backend | 4 built-in: `security-audit`, `performance-review`, `rust-idioms`, `api-review` |

## MCP Server Mode

Run as an MCP server for Claude Code integration:

```bash
cargo run -- --serve
```

Register in `.mcp.json` (project-level):

```json
{
  "mcpServers": {
    "code-review": {
      "command": "/path/to/target/debug/code-review-agent",
      "args": ["--serve"]
    }
  }
}
```

## Architecture

```
src/
  main.rs        CLI entry + backend selection
  agent.rs       Shared Agent Loop (the core)
  llm.rs         LlmBackend trait + 4 implementations
  tools.rs       bash sandbox + skill system
  mcp.rs         MCP Server (rmcp)
  prompts.rs     System prompt (constitution + runtime + followup)
  context.rs     Diff parsing + token budget
  review.rs      Finding/Report structs + AgentAction enum
  resilience.rs  Retry + circuit breaker
```

## Design Principles

- **Custom Agent Loop**: We control think → act → observe, not the LLM backend
- **Pluggable Backends**: Same agent logic, swap the LLM via `--backend`
- **Tool Sandbox**: LLM requests tools, our code validates and executes (read-only bash, curated skills)
- **Budget-Controlled**: Per-file + total token budgets with truncation metadata
- **Resilient**: Exponential backoff retry + circuit breaker (3 consecutive failures → stop)
- **Observable**: Structured tracing events at every decision point
- **Triple Mode**: CLI + MCP Server + Claude Code Skill

## Environment Variables

- `RUST_LOG` — Tracing verbosity (e.g., `RUST_LOG=code_review_agent=debug`)
