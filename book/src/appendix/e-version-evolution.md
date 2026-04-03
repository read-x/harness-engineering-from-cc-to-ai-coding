# 附录 E：版本演化记录

本书核心分析基于 Claude Code v2.1.88（含完整 source map，可还原 4,756 个源文件）。本附录记录后续版本的关键变化及其对各章节的影响。

> 由于 v2.1.89 起 Anthropic 移除了 source map 分发，以下分析基于 bundle 字符串信号对比 + v2.1.88 源码辅助推断，深度有限。

## v2.1.88 → v2.1.91

**概览**：cli.js +115KB | Tengu 事件 +39/-6 | 环境变量 +8/-3 | Source Map 移除

### 高影响变化

| 变化 | 影响章节 | 详情 |
|------|---------|------|
| Tree-sitter WASM 移除 | ch16 权限系统 | Bash 安全从 AST 分析退回 regex/shell-quote；因 CC-643 性能问题 |
| `"auto"` 权限模式正式化 | ch16-17 权限/YOLO | SDK 公开 API 新增 auto mode |
| 冷压缩 + 对话框 + 快速回填熔断 | ch11 微压缩 | 新增延迟压缩策略和用户确认 UI |

### 中影响变化

| 变化 | 影响章节 | 详情 |
|------|---------|------|
| `staleReadFileStateHint` | ch09-10 上下文管理 | 工具执行期间文件 mtime 变化检测 |
| Ultraplan 远程多代理规划 | ch20 Agent 集群 | CCR 远程会话 + Opus 4.6 + 30min 超时 |
| 子代理增强 | ch20-21 多代理/Effort | 回合限制、精简 schema、成本引导 |

### 低影响变化

| 变化 | 影响章节 |
|------|---------|
| `hook_output_persisted` + `pre_tool_hook_deferred` | ch19 Hooks |
| `memory_toggled` + `extract_memories_skipped_no_prose` | ch12 Token 预算 |
| `rate_limit_lever_hint` | ch06 提示词行为引导 |
| `bridge_client_presence_enabled` | ch22 技能系统 |
| +8/-3 环境变量 | 附录 B |

### v2.1.91 新功能详解

以下三个功能在 v2.1.88 源码中**完全不存在**，是 v2.1.91 新增的。分析基于 v2.1.91 bundle 逆向。

#### 1. Powerup Lessons — 交互式功能教程系统

**事件**：`tengu_powerup_lesson_opened`、`tengu_powerup_lesson_completed`

**v2.1.88 状态**：不存在。`restored-src/src/` 中无任何 powerup 或 lesson 相关代码。

**v2.1.91 逆向发现**：

Powerup Lessons 是一个内置的交互式教程系统，包含 10 个课程模块，教用户如何使用 Claude Code 的核心功能。从 bundle 中提取到完整的课程注册表：

| 课程 ID | 标题 | 涉及功能 |
|---------|------|---------|
| `at-mentions` | Talk to your codebase | @ 文件引用、行号引用 |
| `modes` | Steer with modes | Shift+Tab 模式切换、plan、auto |
| `undo` | Undo anything | `/rewind`、Esc-Esc |
| `background` | Run in the background | 后台任务、`/tasks` |
| `memory` | Teach Claude your rules | CLAUDE.md、`/memory`、`/init` |
| `mcp` | Extend with tools | MCP 服务器、`/mcp` |
| `automate` | Automate your workflow | Skills、Hooks、`/hooks` |
| `subagents` | Multiply yourself | 子代理、`/agents`、`--worktree` |
| `cross-device` | Code from anywhere | `/remote-control`、`/teleport` |
| `model-dial` | Dial the model | `/model`、`/effort`、`/fast` |

**技术实现**（从 bundle 逆向）：

```javascript
// 课程打开事件
logEvent("tengu_powerup_lesson_opened", {
  lesson_id: lesson.id,           // 课程 ID
  was_already_unlocked: unlocked.has(lesson.id),  // 是否已解锁
  unlocked_count: unlocked.size   // 已解锁总数
})

// 课程完成事件
logEvent("tengu_powerup_lesson_completed", {
  lesson_id: id,
  unlocked_count: newUnlocked.size,
  all_unlocked: newUnlocked.size === lessons.length  // 是否全部完成
})
```

解锁状态通过 `powerupsUnlocked` 持久化到用户配置中。每个课程包含标题、标语（tagline）、富文本内容（含终端动画演示），UI 使用 ✓/○ 标记完成状态，全部完成后触发"彩蛋"动画。

**本书关联**：Powerup Lessons 的 10 个课程模块几乎覆盖了本书第二到六篇的所有核心主题——从权限模式（ch16-17）到子代理（ch20）到 MCP（ch22）。它是 Anthropic 官方对"用户应该掌握哪些功能"的优先级排序，可作为本书"用户能做什么"小节的参考。

---

#### 2. Write Append Mode — 文件追加写入

**事件**：`tengu_write_append_used`

**v2.1.88 状态**：不存在。v2.1.88 的 Write 工具只支持 overwrite（完整覆盖）模式。

**v2.1.91 逆向发现**：

Write 工具的 inputSchema 新增了 `mode` 参数：

```typescript
// v2.1.91 bundle 逆向
inputSchema: {
  file_path: string,
  content: string,
  mode: "overwrite" | "append"  // v2.1.91 新增
}
```

`mode` 参数描述（从 bundle 提取）：

> Write mode. 'overwrite' (default) replaces the file. Use 'append' to add content to the end of an existing file instead of rewriting the full content — e.g. for logs, accumulating output, or adding entries to a list.

**Feature Gate**：append mode 受 GrowthBook flag `tengu_maple_forge_w8k` 控制。当 flag 关闭时，schema 中的 `mode` 字段被 `.omit()` 移除，模型看不到该参数。

```javascript
// v2.1.91 bundle 逆向
function getWriteSchema() {
  return getFeatureValue("tengu_maple_forge_w8k", false)
    ? fullSchema()           // 包含 mode 参数
    : fullSchema().omit({ mode: true })  // 隐藏 mode 参数
}
```

**本书关联**：影响 ch02（工具系统概览）和 ch08（工具提示词）。v2.1.88 中 Write 工具的提示词明确说"This tool will overwrite the existing file"——v2.1.91 的 append 模式改变了这个约束，模型现在可以选择追加而非覆盖。

---

#### 3. Message Rating — 消息评分反馈

**事件**：`tengu_message_rated`

**v2.1.88 状态**：不存在。v2.1.88 有 `tengu_feedback_survey_*` 系列事件（会话级反馈），但没有消息级别的评分。

**v2.1.91 逆向发现**：

Message Rating 是一个消息级别的用户反馈机制，允许用户对单条 Claude 回复进行评分。从 bundle 逆向提取到的实现：

```javascript
// v2.1.91 bundle 逆向
function rateMessage(messageUuid, sentiment) {
  const wasAlreadyRated = ratings.get(messageUuid) === sentiment
  // 再次点击同一评分 → 清除（toggle 行为）
  if (wasAlreadyRated) {
    ratings.delete(messageUuid)
  } else {
    ratings.set(messageUuid, sentiment)
  }

  logEvent("tengu_message_rated", {
    message_uuid: messageUuid,  // 消息唯一 ID
    sentiment: sentiment,       // 评分方向（如 thumbs_up/thumbs_down）
    cleared: wasAlreadyRated    // 是否为取消评分
  })

  // 评分后显示感谢通知
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

**UI 机制**：
- 通过 React Context（`MessageRatingProvider`）在消息列表中注入评分功能
- 评分状态以 `Map<messageUuid, sentiment>` 存储在内存中
- 支持 toggle——再次点击同一评分会清除
- 评分后弹出绿色通知"thanks for improving claude!"

**本书关联**：与 ch29（可观测性工程）相关。v2.1.88 的反馈系统是会话级的（`tengu_feedback_survey_*`），v2.1.91 新增消息级评分，将反馈粒度从"整个会话好不好"细化到"这条回复好不好"。这为 Anthropic 的 RLHF（人类反馈强化学习）提供了更细粒度的训练信号。

---

### 实验代码名事件

以下带随机代码名的事件属于 A/B 测试，用途未公开：

| 事件 | 备注 |
|------|------|
| `tengu_garnet_plover` | 未知实验 |
| `tengu_gleaming_fair` | 未知实验 |
| `tengu_gypsum_kite` | 未知实验 |
| `tengu_slate_finch` | 未知实验 |
| `tengu_slate_reef` | 未知实验 |
| `tengu_willow_prism` | 未知实验 |
| `tengu_maple_forge_w` | 与 Write Append mode 的 feature gate `tengu_maple_forge_w8k` 相关 |
| `tengu_lean_sub_pf` | 可能与子代理精简 schema 相关 |
| `tengu_sub_nomdrep_q` | 可能与子代理行为相关 |
| `tengu_noreread_q` | 可能与 `tengu_file_read_reread` 文件重读跳过相关 |

---

*使用 `scripts/cc-version-diff.sh` 生成差异数据，`docs/anchor-points.md` 提供子系统锚点定位*
