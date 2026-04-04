# Appendix B: Environment Variable Reference

This appendix lists the key user-configurable environment variables in Claude Code v2.1.88. Grouped by functional domain, only variables affecting user-visible behavior are listed; internal telemetry and platform detection variables are omitted.

## Context Compaction

| Variable | Effect | Default |
|----------|--------|---------|
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | Override context window size (tokens) | Model default |
| `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` | Override auto-compaction threshold as percentage (0-100) | Computed value |
| `DISABLE_AUTO_COMPACT` | Completely disable auto-compaction | `false` |

## Effort and Reasoning

| Variable | Effect | Valid Values |
|----------|--------|-------------|
| `CLAUDE_CODE_EFFORT_LEVEL` | Override effort level | `low`, `medium`, `high`, `max`, `auto`, `unset` |
| `CLAUDE_CODE_DISABLE_FAST_MODE` | Disable Fast Mode accelerated output | `true`/`false` |
| `DISABLE_INTERLEAVED_THINKING` | Disable extended thinking | `true`/`false` |
| `MAX_THINKING_TOKENS` | Override thinking token limit | Model default |

## Tools and Output Limits

| Variable | Effect | Default |
|----------|--------|---------|
| `BASH_MAX_OUTPUT_LENGTH` | Max output characters for Bash commands | 8,000 |
| `CLAUDE_CODE_GLOB_TIMEOUT_SECONDS` | Glob search timeout (seconds) | Default |

## Permissions and Security

| Variable | Effect | Note |
|----------|--------|------|
| `CLAUDE_CODE_DUMP_AUTO_MODE` | Export YOLO classifier requests/responses | Debug only |
| `CLAUDE_CODE_DISABLE_COMMAND_INJECTION_CHECK` | Disable Bash command injection detection | Reduces security |

## API and Authentication

| Variable | Effect | Security Level |
|----------|--------|---------------|
| `ANTHROPIC_API_KEY` | Anthropic API authentication key | Credential |
| `ANTHROPIC_BASE_URL` | Custom API endpoint (proxy support) | Redirectable |
| `ANTHROPIC_MODEL` | Override default model | Safe |
| `CLAUDE_CODE_USE_BEDROCK` | Route inference through AWS Bedrock | Safe |
| `CLAUDE_CODE_USE_VERTEX` | Route inference through Google Vertex AI | Safe |
| `CLAUDE_CODE_EXTRA_BODY` | Append extra fields to API requests | Advanced use |
| `ANTHROPIC_CUSTOM_HEADERS` | Custom HTTP request headers | Safe |

## Model Selection

| Variable | Effect | Example |
|----------|--------|---------|
| `ANTHROPIC_DEFAULT_HAIKU_MODEL` | Custom Haiku model ID | Model string |
| `ANTHROPIC_DEFAULT_SONNET_MODEL` | Custom Sonnet model ID | Model string |
| `ANTHROPIC_DEFAULT_OPUS_MODEL` | Custom Opus model ID | Model string |
| `ANTHROPIC_SMALL_FAST_MODEL` | Fast inference model (e.g., for summaries) | Model string |
| `CLAUDE_CODE_SUBAGENT_MODEL` | Model used by sub-Agents | Model string |

## Prompt Caching

| Variable | Effect | Default |
|----------|--------|---------|
| `CLAUDE_CODE_ENABLE_PROMPT_CACHING` | Enable prompt caching | `true` |
| `DISABLE_PROMPT_CACHING` | Completely disable prompt caching | `false` |

## Session and Debugging

| Variable | Effect | Purpose |
|----------|--------|---------|
| `CLAUDE_CODE_DEBUG_LOG_LEVEL` | Log verbosity | `silent`/`error`/`warn`/`info`/`verbose` |
| `CLAUDE_CODE_PROFILE_STARTUP` | Enable startup performance profiling | Debug |
| `CLAUDE_CODE_PROFILE_QUERY` | Enable query pipeline profiling | Debug |
| `CLAUDE_CODE_JSONL_TRANSCRIPT` | Write session transcript as JSONL | File path |
| `CLAUDE_CODE_TMPDIR` | Override temporary directory | Path |

## Output and Formatting

| Variable | Effect | Default |
|----------|--------|---------|
| `CLAUDE_CODE_SIMPLE` | Minimal system prompt mode | `false` |
| `CLAUDE_CODE_DISABLE_TERMINAL_TITLE` | Disable setting terminal title | `false` |
| `CLAUDE_CODE_NO_FLICKER` | Reduce fullscreen mode flickering | `false` |

## MCP (Model Context Protocol)

| Variable | Effect | Default |
|----------|--------|---------|
| `MCP_TIMEOUT` | MCP server connection timeout (ms) | 10,000 |
| `MCP_TOOL_TIMEOUT` | MCP tool call timeout (ms) | 30,000 |
| `MAX_MCP_OUTPUT_TOKENS` | MCP tool output token limit | Default |

## Network and Proxy

| Variable | Effect | Note |
|----------|--------|------|
| `HTTP_PROXY` / `HTTPS_PROXY` | HTTP/HTTPS proxy | Redirectable |
| `NO_PROXY` | Host list to bypass proxy | Safe |
| `NODE_EXTRA_CA_CERTS` | Additional CA certificates | Affects TLS trust |

## Paths and Configuration

| Variable | Effect | Default |
|----------|--------|---------|
| `CLAUDE_CONFIG_DIR` | Override Claude configuration directory | `~/.claude` |

---

## Version Evolution: v2.1.91 New Variables

| Variable | Effect | Notes |
|----------|--------|-------|
| `CLAUDE_CODE_AGENT_COST_STEER` | Sub-agent cost steering | Controls resource consumption in multi-agent scenarios |
| `CLAUDE_CODE_RESUME_THRESHOLD_MINUTES` | Session resume time threshold | Controls the time window for session resumption |
| `CLAUDE_CODE_RESUME_TOKEN_THRESHOLD` | Session resume token threshold | Controls the token budget for session resumption |
| `CLAUDE_CODE_USE_ANTHROPIC_AWS` | AWS authentication path | Enables Anthropic AWS infrastructure authentication |
| `CLAUDE_CODE_SKIP_ANTHROPIC_AWS_AUTH` | Skip AWS authentication | Fallback path when AWS is unavailable |
| `CLAUDE_CODE_DISABLE_CLAUDE_API_SKILL` | Disable Claude API skill | Enterprise compliance scenario control |
| `CLAUDE_CODE_PLUGIN_KEEP_MARKETPLACE_ON_FAILURE` | Plugin marketplace fault tolerance | Retain cached version when marketplace fetch fails |
| `CLAUDE_CODE_REMOTE_SETTINGS_PATH` | Remote settings path override | Custom settings URL for enterprise deployment |

### v2.1.91 Removed Variables

| Variable | Original Effect | Removal Reason |
|----------|----------------|----------------|
| `CLAUDE_CODE_DISABLE_COMMAND_INJECTION_CHECK` | Disable command injection check | Tree-sitter infrastructure entirely removed |
| `CLAUDE_CODE_DISABLE_MOUSE_CLICKS` | Disable mouse clicks | Feature deprecated |
| `CLAUDE_CODE_MCP_INSTR_DELTA` | MCP instruction delta | Feature refactored |
