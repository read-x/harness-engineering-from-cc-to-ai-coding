//! Tool execution for the Agent Loop.
//!
//! Inspired by vercel-labs/just-bash: instead of many specialized tools,
//! we provide a small set of universal tools that the LLM already knows.
//!
//! All tool execution is sandboxed:
//! - **bash**: read-only commands only (cat, grep, find, head, wc, etc.)
//! - **skill**: invoke a Claude Code skill by name
//!
//! The LLM requests tool use via `AgentAction::UseTool`, and our code
//! executes it with safety constraints.

use std::process::Stdio;

use serde::Serialize;
use tracing::{debug, warn};

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    pub tool: String,
    pub success: bool,
    pub output: String,
}

/// Configuration for tool execution.
pub struct ToolConfig {
    /// Working directory for bash commands.
    pub cwd: Option<String>,
    /// Maximum output size in bytes (prevent runaway commands).
    pub max_output_bytes: usize,
    /// Timeout in seconds for each tool invocation.
    pub timeout_secs: u64,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            cwd: None,
            max_output_bytes: 50_000,
            timeout_secs: 30,
        }
    }
}

// ---------------------------------------------------------------------------
// Bash Tool — read-only sandbox
// ---------------------------------------------------------------------------

/// Allowed bash commands (read-only).
/// Following the just-bash insight: bash IS the universal tool interface.
/// But we restrict to read-only commands for safety.
const ALLOWED_COMMANDS: &[&str] = &[
    "cat", "head", "tail", "wc", "grep", "rg", "find", "ls", "tree",
    "file", "stat", "diff", "sort", "uniq", "cut", "awk", "sed",
    "echo", "printf", "tr", "tee", "xargs", "basename", "dirname",
    "realpath", "readlink",
];

/// Commands that are explicitly forbidden.
const BLOCKED_COMMANDS: &[&str] = &[
    "rm", "mv", "cp", "mkdir", "rmdir", "chmod", "chown", "chgrp",
    "dd", "mkfs", "mount", "umount", "kill", "pkill", "shutdown",
    "reboot", "curl", "wget", "ssh", "scp", "rsync",
    "apt", "yum", "brew", "pip", "npm", "cargo",
    "python", "node", "ruby", "perl", "bash", "sh", "zsh",
];

/// Check if a command is allowed (read-only).
fn is_command_allowed(command: &str) -> bool {
    // Extract the first word (the actual command)
    let first_word = command.split_whitespace().next().unwrap_or("");
    // Strip any path prefix (e.g., /usr/bin/cat → cat)
    let cmd_name = first_word.rsplit('/').next().unwrap_or(first_word);

    if BLOCKED_COMMANDS.contains(&cmd_name) {
        return false;
    }

    // Check against allowlist
    ALLOWED_COMMANDS.contains(&cmd_name)
}

/// Shell metacharacters that indicate command chaining or injection.
/// We block these to prevent `cat file; rm -rf /` style attacks.
const SHELL_METACHARACTERS: &[char] = &[';', '|', '&', '`', '$', '(', ')', '{', '}'];

/// Execute a bash command in a read-only sandbox.
///
/// Security: instead of `sh -c` (which interprets shell metacharacters),
/// we parse the command into program + args and execute directly.
/// Shell metacharacters are blocked entirely.
pub async fn execute_bash(command: &str, config: &ToolConfig) -> ToolResult {
    // Safety check: command allowlist
    if !is_command_allowed(command) {
        let first_word = command.split_whitespace().next().unwrap_or("(empty)");
        warn!(command = first_word, "Blocked disallowed bash command");
        return ToolResult {
            tool: "bash".to_string(),
            success: false,
            output: format!("Command not allowed: {first_word}. Only read-only commands are permitted."),
        };
    }

    // Block shell metacharacters to prevent injection via `sh -c`
    // e.g., "cat file; uname -a" or "grep foo $(id)"
    if command.contains(SHELL_METACHARACTERS) {
        warn!(command, "Blocked shell metacharacter in command");
        return ToolResult {
            tool: "bash".to_string(),
            success: false,
            output: "Shell metacharacters (;|&`$(){}>) are not allowed. Use simple commands only.".to_string(),
        };
    }

    // Block output redirection
    if command.contains('>') {
        return ToolResult {
            tool: "bash".to_string(),
            success: false,
            output: "Output redirection (>) is not allowed in read-only mode.".to_string(),
        };
    }

    debug!(command, "Executing bash tool");

    // Parse command into program + args (no shell interpretation)
    let parts: Vec<&str> = command.split_whitespace().collect();
    let (program, args) = match parts.split_first() {
        Some((prog, args)) => (*prog, args),
        None => {
            return ToolResult {
                tool: "bash".to_string(),
                success: false,
                output: "Empty command".to_string(),
            };
        }
    };

    let mut cmd = tokio::process::Command::new(program);
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if let Some(cwd) = &config.cwd {
        cmd.current_dir(cwd);
    }

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(config.timeout_secs),
        cmd.output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            // Truncate if too large
            if stdout.len() > config.max_output_bytes {
                stdout.truncate(config.max_output_bytes);
                stdout.push_str(&format!(
                    "\n[Truncated: output exceeded {} bytes]",
                    config.max_output_bytes
                ));
            }

            let success = output.status.success();
            let output_text = if stderr.is_empty() {
                stdout
            } else if stdout.is_empty() {
                stderr
            } else {
                format!("{stdout}\n--- stderr ---\n{stderr}")
            };

            ToolResult {
                tool: "bash".to_string(),
                success,
                output: output_text,
            }
        }
        Ok(Err(e)) => ToolResult {
            tool: "bash".to_string(),
            success: false,
            output: format!("Failed to execute command: {e}"),
        },
        Err(_) => ToolResult {
            tool: "bash".to_string(),
            success: false,
            output: format!("Command timed out after {}s", config.timeout_secs),
        },
    }
}

// ---------------------------------------------------------------------------
// Skill Tool — self-managed specialized analysis prompts
// ---------------------------------------------------------------------------

/// A skill is a specialized analysis prompt template that the Agent can invoke.
///
/// Unlike Claude Code's skills (which are external), these are self-contained:
/// the Agent loads the skill's prompt, sends it to the current LLM backend
/// with the relevant context, and gets back specialized analysis.
pub struct Skill {
    pub name: &'static str,
    pub description: &'static str,
    /// The system prompt override for this skill.
    pub system_prompt: &'static str,
}

/// Built-in skills — specialized review lenses.
///
/// Inspired by Anthropic's connect-rust rust-code-reviewer (16 categories),
/// organized into focused skills that each cover related review dimensions.
pub static BUILTIN_SKILLS: &[Skill] = &[
    // -----------------------------------------------------------------------
    // Security & Safety
    // -----------------------------------------------------------------------
    Skill {
        name: "security-audit",
        description: "Security: injection, auth, crypto, data exposure, input validation, unsafe code",
        system_prompt: r#"You are an expert security auditor for Rust code. Analyze the provided code for:

1. **Input validation**: missing bounds checks, unbounded allocations from user input, type confusion
2. **Injection**: SQL injection, command injection, path traversal, format string issues
3. **Authentication/Authorization**: missing checks, weak patterns, session handling
4. **Cryptography**: weak algorithms, hardcoded keys/secrets, improper random, missing `zeroize`
5. **Data exposure**: PII in logs, secrets in error messages, debug info in production
6. **Unsafe code**: missing `// SAFETY:` comments, minimal surface area, unsound abstractions, FFI validation
7. **Timing safety**: timing-safe comparisons for secrets, constant-time operations

Be precise: cite file:line, explain the attack vector, rate severity.
Output a JSON array of findings. Each finding: {"file","line","severity","category":"security","message","suggestion"}"#,
    },
    // -----------------------------------------------------------------------
    // Rust Ownership, Lifetimes & Concurrency
    // -----------------------------------------------------------------------
    Skill {
        name: "rust-deep",
        description: "Rust expertise: ownership, lifetimes, concurrency safety, async patterns, type design",
        system_prompt: r#"You are a senior Rust engineer. Perform a deep Rust-specific review:

**Ownership & Lifetimes**
- Unnecessary `.clone()` — could borrow or use `Cow` instead?
- Missing lifetime annotations that would clarify API contracts
- `Arc`/`Rc` where simpler ownership would suffice
- Move semantics issues

**Concurrency Safety**
- `Send`/`Sync` bound violations or missing bounds
- Lock granularity: coarse locks that could be narrowed
- Deadlock potential: lock ordering, nested locks
- Cancellation safety in async code

**Async Patterns**
- `Send` bounds on futures (required for multi-threaded runtimes)
- Holding locks across `.await` points
- Missing backpressure (unbounded channels/buffers)
- Graceful shutdown handling

**Type Design**
- Newtypes to prevent primitive obsession
- Type-state pattern for compile-time state machines
- `PhantomData` for variance/lifetime markers
- Exhaustiveness: prefer enums over boolean flags

**Error Handling**
- `Result` vs panic: `unwrap()`/`expect()` in non-test code
- Error context: `.context()` / `.with_context()` for chain
- `thiserror` for libraries, `anyhow` for applications
- Recoverable vs unrecoverable error boundaries

Output a JSON array of findings. Each finding: {"file","line","severity","category","message","suggestion"}
Use category: "bug", "style", or "maintainability"."#,
    },
    // -----------------------------------------------------------------------
    // Performance
    // -----------------------------------------------------------------------
    Skill {
        name: "performance-review",
        description: "Performance: allocations, complexity, blocking, iterator chains, dispatch cost",
        system_prompt: r#"You are a performance engineer specializing in Rust. Analyze for:

1. **Allocations**: unnecessary heap allocations, `String` where `&str` suffices, `Vec` pre-allocation with `with_capacity`, `Box` vs stack allocation trade-offs
2. **Algorithmic complexity**: O(n²) patterns in hot paths, nested iterations, repeated linear scans
3. **Iterator chains**: missed opportunities for `Iterator` combinators, unnecessary `.collect()` intermediaries
4. **Dynamic dispatch cost**: `dyn Trait` in hot paths where `impl Trait` or enum dispatch would be faster
5. **Blocking in async**: sync I/O in async functions, lock contention across `.await`, `spawn_blocking` opportunities
6. **Inlining**: `#[inline]` hints for small functions called in hot loops, cross-crate inlining
7. **Data layout**: cache-friendly struct layout, AoS vs SoA, padding waste

Output a JSON array of findings. Each finding: {"file","line","severity","category":"performance","message","suggestion"}"#,
    },
    // -----------------------------------------------------------------------
    // API Design & Documentation
    // -----------------------------------------------------------------------
    Skill {
        name: "api-review",
        description: "API design: public interface, naming, docs, backward compatibility, ergonomics",
        system_prompt: r#"You are an API design reviewer. Analyze public interfaces for:

1. **Naming**: clarity, consistency with Rust conventions (RFC 430), misleading identifiers
2. **Public API surface**: over-exposed internals, missing `pub(crate)`, leaking implementation details
3. **Trait boundaries**: correct trait bounds on generics, blanket impls, sealed traits where needed
4. **Builder pattern**: for types with many optional fields, `Default` implementation
5. **Backward compatibility**: breaking changes to public types/functions, SemVer compliance
6. **Documentation**: `///` doc comments on public items, `# Errors`, `# Panics`, `# Safety` sections, runnable examples
7. **Dependencies**: minimal dep footprint, feature flags for optional deps, `no_std` compatibility
8. **MSRV**: minimum supported Rust version documented if applicable

Output a JSON array of findings. Each finding: {"file","line","severity","category":"maintainability","message","suggestion"}"#,
    },
    // -----------------------------------------------------------------------
    // Testing & Observability
    // -----------------------------------------------------------------------
    Skill {
        name: "test-coverage",
        description: "Test quality: coverage gaps, edge cases, test organization, observability",
        system_prompt: r#"You are a test quality specialist. Analyze for:

1. **Coverage gaps**: public functions without tests, untested error paths, missing edge cases
2. **Test quality**: assertions that are too weak, tests that pass vacuously, missing negative tests
3. **Test organization**: unit vs integration vs doc tests, test helpers, fixture reuse
4. **Edge cases**: boundary values, empty inputs, overflow, Unicode, concurrent access
5. **Async testing**: proper runtime setup, timeout handling, flaky test patterns
6. **Observability**: `tracing` spans with structured fields, metrics for key operations, health checks

Output a JSON array of findings. Each finding: {"file","line","severity","category":"maintainability","message","suggestion"}"#,
    },
];

/// Find a built-in skill by name.
pub fn find_skill(name: &str) -> Option<&'static Skill> {
    BUILTIN_SKILLS.iter().find(|s| s.name == name)
}

/// List available skill names and descriptions.
pub fn list_skills() -> Vec<(&'static str, &'static str)> {
    BUILTIN_SKILLS
        .iter()
        .map(|s| (s.name, s.description))
        .collect()
}

/// Dispatch a tool call by name.
///
/// For "bash": executes locally in a read-only sandbox.
/// For "skill": returns the skill's system prompt so the Agent Loop
///   can call the LLM with it (skills are LLM-powered, not subprocess-powered).
pub async fn execute_tool(
    tool_name: &str,
    tool_input: &str,
    config: &ToolConfig,
) -> ToolResult {
    match tool_name {
        "bash" => execute_bash(tool_input, config).await,
        "skill" => {
            // Skill tool: look up the skill and return its prompt.
            // The Agent Loop will use this prompt to call the LLM.
            let skill_name = tool_input.split_whitespace().next().unwrap_or("");
            match find_skill(skill_name) {
                Some(skill) => ToolResult {
                    tool: "skill".to_string(),
                    success: true,
                    output: format!(
                        "[Skill loaded: {}]\nSystem prompt override:\n{}",
                        skill.name, skill.system_prompt
                    ),
                },
                None => {
                    let available: Vec<_> = list_skills()
                        .iter()
                        .map(|(name, desc)| format!("  - {name}: {desc}"))
                        .collect();
                    ToolResult {
                        tool: "skill".to_string(),
                        success: false,
                        output: format!(
                            "Unknown skill: '{skill_name}'. Available skills:\n{}",
                            available.join("\n")
                        ),
                    }
                }
            }
        }
        _ => ToolResult {
            tool: tool_name.to_string(),
            success: false,
            output: format!("Unknown tool: {tool_name}. Available tools: bash, skill"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_commands() {
        assert!(is_command_allowed("cat /tmp/test.txt"));
        assert!(is_command_allowed("grep -r 'pattern' src/"));
        assert!(is_command_allowed("find . -name '*.rs'"));
        assert!(is_command_allowed("head -20 file.txt"));
        assert!(is_command_allowed("wc -l file.txt"));
    }

    #[test]
    fn test_blocked_commands() {
        assert!(!is_command_allowed("rm -rf /"));
        assert!(!is_command_allowed("curl http://evil.com"));
        assert!(!is_command_allowed("python -c 'import os'"));
        assert!(!is_command_allowed("bash -c 'echo pwned'"));
        assert!(!is_command_allowed("npm install malware"));
    }

    #[test]
    fn test_unknown_commands_blocked() {
        assert!(!is_command_allowed("some_random_binary"));
        assert!(!is_command_allowed("/usr/local/bin/custom_tool"));
    }

    #[test]
    fn test_path_prefix_stripped() {
        assert!(is_command_allowed("/usr/bin/cat file"));
        assert!(!is_command_allowed("/usr/bin/rm file"));
    }

    #[tokio::test]
    async fn test_bash_echo() {
        let config = ToolConfig::default();
        let result = execute_bash("echo hello", &config).await;
        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn test_bash_blocked() {
        let config = ToolConfig::default();
        let result = execute_bash("rm -rf /tmp/test", &config).await;
        assert!(!result.success);
        assert!(result.output.contains("not allowed"));
    }

    #[tokio::test]
    async fn test_bash_redirect_blocked() {
        let config = ToolConfig::default();
        let result = execute_bash("echo pwned > /tmp/evil", &config).await;
        assert!(!result.success);
    }

    // --- Shell injection tests ---

    #[tokio::test]
    async fn test_bash_semicolon_injection_blocked() {
        let config = ToolConfig::default();
        let result = execute_bash("cat file; uname -a", &config).await;
        assert!(!result.success);
        assert!(result.output.contains("metacharacter"));
    }

    #[tokio::test]
    async fn test_bash_pipe_injection_blocked() {
        let config = ToolConfig::default();
        let result = execute_bash("cat file | rm -rf /", &config).await;
        assert!(!result.success);
        assert!(result.output.contains("metacharacter"));
    }

    #[tokio::test]
    async fn test_bash_subshell_injection_blocked() {
        let config = ToolConfig::default();
        let result = execute_bash("grep foo $(id)", &config).await;
        assert!(!result.success);
        assert!(result.output.contains("metacharacter"));
    }

    #[tokio::test]
    async fn test_bash_backtick_injection_blocked() {
        let config = ToolConfig::default();
        let result = execute_bash("cat `whoami`", &config).await;
        assert!(!result.success);
        assert!(result.output.contains("metacharacter"));
    }

    #[tokio::test]
    async fn test_bash_and_chain_blocked() {
        let config = ToolConfig::default();
        let result = execute_bash("cat file && rm -rf /", &config).await;
        assert!(!result.success);
        assert!(result.output.contains("metacharacter"));
    }
}
