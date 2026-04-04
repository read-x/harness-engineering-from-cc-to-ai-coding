# Appendix E: Version Evolution Log

The core analysis in this book is based on Claude Code v2.1.88 (with full source map, enabling recovery of 4,756 source files). This appendix records key changes in subsequent versions and their impact on each chapter.

> **Navigation tip**: Each change links to the corresponding chapter's version evolution section. Click the chapter number to jump.

> Since Anthropic removed source map distribution starting from v2.1.89, the following analysis is based on bundle string signal comparison + v2.1.88 source code-assisted inference, with limited depth.

## v2.1.88 -> v2.1.91

**Overview**: cli.js +115KB | Tengu events +39/-6 | Environment variables +8/-3 | Source Map removed

### High-Impact Changes

| Change | Affected Chapters | Details |
|--------|-------------------|---------|
| Tree-sitter WASM removal | [ch16 Permission System](../part5/ch16.md#version-evolutionv2191-changes) | Bash security reverted from AST analysis to regex/shell-quote; due to CC-643 performance issues |
| `"auto"` permission mode formalized | [ch16](../part5/ch16.md#version-evolutionv2191-changes)-[ch17](../part5/ch17.md#version-evolutionv2191-changes) Permissions/YOLO | SDK public API added auto mode |
| Cold compaction + dialog + quick backfill circuit breaker | [ch11 Micro-compaction](../part3/ch11.md#version-evolutionv2191-changes) | Added deferred compaction strategy and user confirmation UI |

### Medium-Impact Changes

| Change | Affected Chapters | Details |
|--------|-------------------|---------|
| `staleReadFileStateHint` | [ch09](../part3/ch09.md#version-evolutionv2191-changes)-[ch10](../part3/ch10.md#version-evolutionv2191-changes) Context Management | File mtime change detection during tool execution |
| Ultraplan remote multi-agent planning | [ch20 Agent Clusters](../part6/ch20.md) | CCR remote sessions + Opus 4.6 + 30min timeout |
| Sub-agent enhancements | [ch20](../part6/ch20.md)-[ch21](../part6/ch21.md#version-evolutionv2191-changes) Multi-agent/Effort | Turn limits, lean schema, cost steering |

### Low-Impact Changes

| Change | Affected Chapters |
|--------|-------------------|
| `hook_output_persisted` + `pre_tool_hook_deferred` | ch19 Hooks |
| `memory_toggled` + `extract_memories_skipped_no_prose` | ch12 Token Budget |
| `rate_limit_lever_hint` | ch06 Prompt Behavior Steering |
| `bridge_client_presence_enabled` | ch22 Skills System |
| +8/-3 environment variables | Appendix B |

### v2.1.91 New Features in Detail

The following three features **did not exist at all** in v2.1.88 source code and are new in v2.1.91. Analysis is based on v2.1.91 bundle reverse engineering.

#### 1. Powerup Lessons — Interactive Feature Tutorial System

**Events**: `tengu_powerup_lesson_opened`, `tengu_powerup_lesson_completed`

**v2.1.88 status**: Did not exist. No powerup or lesson-related code in `restored-src/src/`.

**v2.1.91 reverse engineering findings**:

Powerup Lessons is a built-in interactive tutorial system containing 10 course modules that teach users how to use Claude Code's core features. The complete course registry extracted from the bundle:

| Course ID | Title | Related Features |
|-----------|-------|-----------------|
| `at-mentions` | Talk to your codebase | @ file references, line number references |
| `modes` | Steer with modes | Shift+Tab mode switching, plan, auto |
| `undo` | Undo anything | `/rewind`, Esc-Esc |
| `background` | Run in the background | Background tasks, `/tasks` |
| `memory` | Teach Claude your rules | CLAUDE.md, `/memory`, `/init` |
| `mcp` | Extend with tools | MCP servers, `/mcp` |
| `automate` | Automate your workflow | Skills, Hooks, `/hooks` |
| `subagents` | Multiply yourself | Sub-agents, `/agents`, `--worktree` |
| `cross-device` | Code from anywhere | `/remote-control`, `/teleport` |
| `model-dial` | Dial the model | `/model`, `/effort`, `/fast` |

**Technical implementation** (from bundle reverse engineering):

```javascript
// Course opened event
logEvent("tengu_powerup_lesson_opened", {
  lesson_id: lesson.id,           // Course ID
  was_already_unlocked: unlocked.has(lesson.id),  // Already unlocked?
  unlocked_count: unlocked.size   // Total unlocked count
})

// Course completed event
logEvent("tengu_powerup_lesson_completed", {
  lesson_id: id,
  unlocked_count: newUnlocked.size,
  all_unlocked: newUnlocked.size === lessons.length  // All completed?
})
```

Unlock state is persisted to user configuration via `powerupsUnlocked`. Each course contains a title, tagline, rich text content (with terminal animation demos), and the UI uses check/circle markers for completion status, triggering an "easter egg" animation when all courses are completed.

**Book relevance**: The 10 course modules of Powerup Lessons cover nearly all core topics from Parts 2 through 6 of this book — from permission modes (ch16-17) to sub-agents (ch20) to MCP (ch22). It represents Anthropic's official prioritization of "which features users should master" and can serve as a reference for this book's "What You Can Do" sections.

---

#### 2. Write Append Mode — File Append Writing

**Event**: `tengu_write_append_used`

**v2.1.88 status**: Did not exist. v2.1.88's Write tool only supported overwrite (complete replacement) mode.

**v2.1.91 reverse engineering findings**:

The Write tool's inputSchema gained a new `mode` parameter:

```typescript
// v2.1.91 bundle reverse engineering
inputSchema: {
  file_path: string,
  content: string,
  mode: "overwrite" | "append"  // New in v2.1.91
}
```

`mode` parameter description (extracted from bundle):

> Write mode. 'overwrite' (default) replaces the file. Use 'append' to add content to the end of an existing file instead of rewriting the full content — e.g. for logs, accumulating output, or adding entries to a list.

**Feature Gate**: Append mode is controlled by GrowthBook flag `tengu_maple_forge_w8k`. When the flag is off, the `mode` field is `.omit()`'d from the schema, making it invisible to the model.

```javascript
// v2.1.91 bundle reverse engineering
function getWriteSchema() {
  return getFeatureValue("tengu_maple_forge_w8k", false)
    ? fullSchema()           // Includes mode parameter
    : fullSchema().omit({ mode: true })  // Hides mode parameter
}
```

**Book relevance**: Affects ch02 (tool system overview) and ch08 (tool prompts). In v2.1.88, the Write tool's prompt explicitly stated "This tool will overwrite the existing file" — v2.1.91's append mode changes this constraint, and the model can now choose to append rather than overwrite.

---

#### 3. Message Rating — Message Rating Feedback

**Event**: `tengu_message_rated`

**v2.1.88 status**: Did not exist. v2.1.88 had `tengu_feedback_survey_*` series events (session-level feedback) but no message-level rating.

**v2.1.91 reverse engineering findings**:

Message Rating is a message-level user feedback mechanism that allows users to rate individual Claude responses. Implementation extracted from bundle reverse engineering:

```javascript
// v2.1.91 bundle reverse engineering
function rateMessage(messageUuid, sentiment) {
  const wasAlreadyRated = ratings.get(messageUuid) === sentiment
  // Clicking the same rating again → clear (toggle behavior)
  if (wasAlreadyRated) {
    ratings.delete(messageUuid)
  } else {
    ratings.set(messageUuid, sentiment)
  }

  logEvent("tengu_message_rated", {
    message_uuid: messageUuid,  // Message unique ID
    sentiment: sentiment,       // Rating direction (e.g., thumbs_up/thumbs_down)
    cleared: wasAlreadyRated    // Was the rating cleared?
  })

  // Show thank-you notification after rating
  if (!wasAlreadyRated) {
    addNotification({
      key: "message-rated",
      text: "thanks for improving claude!",
      color: "success",
      priority: "immediate"
    })
  }
}
```

**UI mechanics**:
- Rating functionality is injected into the message list via React Context (`MessageRatingProvider`)
- Rating state is stored in memory as `Map<messageUuid, sentiment>`
- Supports toggle — clicking the same rating again clears it
- After rating, a green notification "thanks for improving claude!" appears

**Book relevance**: Related to ch29 (Observability Engineering). v2.1.88's feedback system was session-level (`tengu_feedback_survey_*`); v2.1.91 adds message-level rating, refining feedback granularity from "was the whole session good" to "was this specific response good." This provides Anthropic with more fine-grained training signals for RLHF (Reinforcement Learning from Human Feedback).

---

### Experimental Codename Events

The following events with random codenames are A/B tests with undisclosed purposes:

| Event | Notes |
|-------|-------|
| `tengu_garnet_plover` | Unknown experiment |
| `tengu_gleaming_fair` | Unknown experiment |
| `tengu_gypsum_kite` | Unknown experiment |
| `tengu_slate_finch` | Unknown experiment |
| `tengu_slate_reef` | Unknown experiment |
| `tengu_willow_prism` | Unknown experiment |
| `tengu_maple_forge_w` | Related to Write Append mode's feature gate `tengu_maple_forge_w8k` |
| `tengu_lean_sub_pf` | Possibly related to sub-agent lean schema |
| `tengu_sub_nomdrep_q` | Possibly related to sub-agent behavior |
| `tengu_noreread_q` | Possibly related to `tengu_file_read_reread` file re-read skipping |

---

*Use `scripts/cc-version-diff.sh` to generate diff data; `docs/anchor-points.md` provides subsystem anchor point locations*
