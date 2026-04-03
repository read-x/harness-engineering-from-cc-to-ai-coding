spec: task
name: "版本演化追踪与内容更新"
inherits: project
tags: [version-tracking, reverse-engineering, v2.1.91, evolution]
depends: [part1-architecture, part2-prompt-engineering, part3-context-management, part4-prompt-cache, part5-safety-permissions, part6-advanced-subsystems]
estimate: ongoing
---

## 意图

建立系统化的版本追踪能力，在 Anthropic 移除 source map（v2.1.89+）后，仍能通过 bundle 字符串信号挖掘 + v2.1.88 基线源码辅助推断的方式，持续追踪 Claude Code 的演进，并将发现融入书籍内容。

## 背景

- v2.1.88 是本书的基线版本，含完整 source map（57MB），可还原 4,756 个源文件
- v2.1.89 起 Anthropic 移除了 source map 分发
- 新版本分析依赖：tengu_* 事件名、CLAUDE_CODE_* 环境变量、GrowthBook 配置名、sdk-tools.d.ts 类型定义、v2.1.88 源码辅助推断
- 逆向工具包（`scripts/extract-signals.sh`、`scripts/cc-version-diff.sh`）和参考文档（`docs/anchor-points.md`、`docs/reverse-engineering-guide.md`）已就绪

## 已定决策

### 工具链

| 工具 | 路径 | 用途 |
|------|------|------|
| `extract-signals.sh` | `scripts/` | 从单个 bundle 提取 tengu 事件、环境变量、GrowthBook 配置、工具名、API 端点 |
| `cc-version-diff.sh` | `scripts/` | 两版本结构化差异报告（支持单参数基线模式） |
| `version-track` skill | `.claude/skills/version-track/SKILL.md` | 6 步工作流：获取→逆向→差异→映射→草稿→提交 |
| `anchor-points.md` | `docs/` | 120+ 子系统锚点索引（用于无 source map 时的 bundle 定位） |
| `reverse-engineering-guide.md` | `docs/` | 逆向方法论参考（两条路径：有/无 source map） |

### 内容策略

- **核心分析保持基于 v2.1.88**：有完整源码的深度分析不可替代
- **版本演化作为补充**：在已有章节末尾追加"版本演化：vX.Y.Z 变化"小节
- **证据分级标注**：
  - "v2.1.88 源码证据"——有完整源码行号引用
  - "v2.1.91 bundle 逆向"——基于字符串信号 + v2.1.88 辅助推断
  - "推断"——仅从事件名推测，无直接证据
- **附录 E 作为速查**：`book/src/appendix/e-version-evolution.md` 集中记录各版本变化

### 章节更新格式

每个受影响章节追加的"版本演化"小节统一使用以下格式：

```markdown
---

## 版本演化：vX.Y.Z 变化

> 以下分析基于 vX.Y.Z bundle 信号对比，结合 v2.1.88 源码推断。

### [变化标题]

[分析内容，标注证据来源]
```

### 信号到章节的映射规则

| 信号模式 | 主要影响章节 |
|---------|------------|
| `tengu_api_*` | ch06 API/流式 |
| `tengu_compact_*` / `tengu_autocompact_*` | ch09 自动压缩, ch11 微压缩 |
| `tengu_prompt_cache_*` | ch13-15 提示词缓存 |
| `tengu_tool_use_*` / `tengu_bash_*` | ch04 工具执行, ch16 权限 |
| `tengu_permission_*` / `tengu_yolo_*` | ch17 YOLO 分类器 |
| `tengu_hook_*` | ch18 Hooks |
| `tengu_mcp_*` | ch22 技能系统 |
| `tengu_agent_*` / `tengu_forked_agent_*` | ch20 多代理 |
| `tengu_ultraplan_*` | ch20 多代理（Ultraplan 小节） |
| `tengu_session_*` / `tengu_cost_*` | ch09 压缩/会话 |
| `tengu_skill_*` | ch22 技能系统 |
| `tengu_memory_*` | ch12 Token 预算, ch24 跨会话记忆 |
| `tengu_streaming_*` | ch06 流式处理 |
| `tengu_bridge_*` | ch22 技能系统（IDE 桥接小节） |
| `sdk-tools.d.ts` 变化 | ch04 工具执行（公开 API） |
| 新增 `CLAUDE_CODE_*` | 附录 B + 对应子系统章节 |
| GrowthBook 配置变化 | ch23 Feature Flags |
| 带随机代码名事件 | ch23 实验系统 |

## v2.1.88 → v2.1.91 已完成的更新

### 差异报告

| 文件 | 说明 |
|------|------|
| `docs/version-diffs/v2.1.88-vs-v2.1.91.md` | 原始差异数据（自动生成） |
| `docs/version-diffs/v2.1.88-vs-v2.1.91-book-impact.md` | 书籍影响分析 |

### 章节更新（17 个文件，+871 行）

**高影响（核心内容修改）**：

| 章节 | 文件 | 变化摘要 |
|------|------|---------|
| ch16 权限系统 | `part5/ch16.md` | +auto 模式正式化 +tree-sitter 移除（CC-643 性能问题） |
| ch11 微压缩 | `part3/ch11.md` | +冷压缩 +压缩对话框 +快速回填熔断器 +手动压缩追踪 |
| ch17 YOLO 分类器 | `part5/ch17.md` | +auto 模式成为公开 API |

**中影响（新增小节）**：

| 章节 | 文件 | 变化摘要 |
|------|------|---------|
| ch09 自动压缩 | `part3/ch09.md` | +staleReadFileStateHint 文件陈旧检测 |
| ch10 文件状态保留 | `part3/ch10.md` | +staleReadFileStateHint 扩展文件追踪体系 |
| ch20 多代理 | `part6/ch20.md` | +Ultraplan 完整深度分析（371 行）：CCR 架构、状态机、关键字触发、轮询、GrowthBook 提示词变体、传送协议、错误处理、遥测、用户指南 |
| ch21 Effort/Thinking | `part6/ch21.md` | +CLAUDE_CODE_AGENT_COST_STEER 成本控制 |

**低影响（1-2 句补充）**：

| 章节 | 文件 | 变化摘要 |
|------|------|---------|
| ch04 工具执行 | `part1/ch04.md` | +staleReadFileStateHint 输出通道 |
| ch06 提示词行为 | `part2/ch06.md` | +rate_limit_lever_hint 限速引导 |
| ch12 Token 预算 | `part3/ch12.md` | +memory_toggled +无散文跳过 |
| ch18 Hooks | `part5/ch18.md` | 修正 `if` 字段表述（添加多模式示例） |
| ch19 CLAUDE.md | `part5/ch19.md` | +hook_output_persisted +pre_tool_hook_deferred |
| ch22 技能系统 | `part6/ch22.md` | +bridge_client_presence_enabled +DISABLE_CLAUDE_API_SKILL |

**附录**：

| 文件 | 变化摘要 |
|------|---------|
| `appendix/b-env-vars.md` | +8 新增变量 / -3 移除变量 |
| `appendix/e-version-evolution.md` | **新建**：版本演化速查 + Powerup Lessons / Write Append / Message Rating 详细逆向 |
| `SUMMARY.md` | +附录 E 链接 |

## 验收标准

### 新版本追踪（每次执行 version-track 工作流时）

- [ ] 差异报告已生成并保存到 `docs/version-diffs/`
- [ ] 书籍影响分析包含完整的章节映射表（影响级别 + 建议操作）
- [ ] 高影响章节已追加"版本演化"小节
- [ ] 中影响章节已追加简要说明
- [ ] 低影响章节和附录已更新
- [ ] 附录 E 已追加新版本条目
- [ ] `mdbook build` 通过
- [ ] 所有新增内容标注了证据来源（源码/bundle 逆向/推断）

### 逆向工具链维护

- [ ] `docs/anchor-points.md` 在发现新稳定锚点时更新
- [ ] `scripts/extract-signals.sh` 在遇到新信号模式时更新
- [ ] `scripts/cc-version-diff.sh` 在需要新对比维度时更新

## 边界

### 允许修改
- `book/src/**/ch*.md`（追加版本演化小节）
- `book/src/appendix/e-version-evolution.md`（追加新版本条目）
- `book/src/appendix/b-env-vars.md`（更新环境变量）
- `book/src/SUMMARY.md`（添加新附录链接）
- `docs/version-diffs/`（差异报告）
- `docs/anchor-points.md`（锚点更新）
- `scripts/`（工具链维护）

### 禁止
- 不修改已有章节的核心分析内容（v2.1.88 源码分析部分）
- 不将 bundle 逆向的推断结果伪装为确定性结论
- 不修改 `restored-src/` 下的任何文件
- 不将 tarball 二进制文件提交到 git

## 排除范围

- 不追踪 Claude Code 以外的 Anthropic 产品变化
- 不反编译/美化 minified JavaScript（仅提取字符串常量）
- 不对实验代码名事件做深度分析（仅记录存在性）
