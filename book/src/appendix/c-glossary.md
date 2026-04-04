# 附录 C：术语表

本附录收录本书中首次出现时附英文原文的技术术语，按中文拼音排序。中英双语格式，供翻译时作为术语一致性的 single source of truth。

| 中文术语 | 英文术语 | 定义 | 首见章节 |
|----------|---------|------|---------|
| Agent Loop | Agent Loop | AI Agent 的核心执行循环：接收输入 → 调用模型 → 执行工具 → 判断是否继续。The core execution loop of an AI Agent: receive input → call model → execute tools → decide whether to continue. | 第3章 |
| 并发分区 | Partition | 将工具调用分为可并行和必须串行的批次，基于 `isConcurrencySafe` 属性。Splitting tool calls into parallelizable and sequential batches based on `isConcurrencySafe`. | 第4章 |
| 抽象语法树 | AST (Abstract Syntax Tree) | 源代码的树状结构表示，保留语义关系（而非纯文本）。A tree representation of source code that preserves semantic relationships. | 第28章 |
| 大纲 | Outline | 书籍目录结构和各章主题的概览文档。Overview document of the book's table of contents and chapter topics. | 前言 |
| 动态边界 | Dynamic Boundary | 系统提示词中分隔静态可缓存内容与动态会话内容的标记。Marker in the system prompt separating cacheable static content from dynamic session content. | 第5章 |
| 防御性 Git | Defensive Git | 在 AI 执行 Git 操作时通过显式安全规则防止数据丢失的模式。A pattern of preventing data loss during AI-driven Git operations via explicit safety rules. | 第27章 |
| 工具 Schema | Tool Schema | 工具的 JSON Schema 定义，包含名称、描述、输入参数格式。A tool's JSON Schema definition including name, description, and input parameter format. | 第2章 |
| 驾驭工程 | Harness Engineering | 通过提示词、工具和配置（而非代码逻辑）引导 AI 模型行为的实践。The practice of steering AI model behavior through prompts, tools, and configuration rather than code logic. | 第1章 |
| 渐进式自主 | Graduated Autonomy | 从手动确认到全自动的多级权限模式，每级都有安全回退。A multi-tier permission model ranging from manual confirmation to full automation, with safety fallbacks at each level. | 第27章 |
| 技能 | Skill | 可调用的提示词模板，通过 SkillTool 注入对话上下文。An invokable prompt template injected into conversation context via SkillTool. | 第22章 |
| 缓存中断 | Cache Break | 提示词缓存前缀因内容变化而失效的事件。An event where the prompt cache prefix is invalidated due to content changes. | 第14章 |
| 锁存 | Latch | 一旦进入即保持稳定的会话级状态，防止缓存振荡或行为抖动。A session-level state that stays stable once entered, preventing cache oscillation or behavioral jitter. | 第13章、第25章 |
| 模式提炼 | Pattern Extraction | 从源码分析中提取可复用的设计模式，包含名称、问题、解决方案。Extracting reusable design patterns from source code analysis, including name, problem, and solution. | 全书 |
| 熔断器 | Circuit Breaker | 连续 N 次失败后强制停止自动化流程，降级到安全状态。Forcefully stopping an automated process after N consecutive failures, degrading to a safe state. | 第9章、第26章 |
| 死代码消除 | DCE (Dead Code Elimination) | Bun 的 `feature()` 函数实现编译时移除门控代码。Compile-time removal of gated code via Bun's `feature()` function. | 第1章 |
| 失败关闭 | Fail-Closed | 系统默认选择最安全的选项，需显式声明才能解锁危险操作。The system defaults to the safest option; dangerous operations require explicit opt-in. | 第2章、第25章 |
| 提示词缓存 | Prompt Cache | Anthropic API 特性，缓存消息前缀以减少重复 token 处理。An Anthropic API feature that caches message prefixes to reduce redundant token processing. | 第13章 |
| 微压缩 | Microcompact | 精准移除特定工具结果（而非完整压缩整个对话），保持缓存前缀稳定。Precisely removing specific tool results (rather than compacting the entire conversation), keeping cache prefixes stable. | 第11章 |
| 压缩 | Compaction | 总结对话历史以释放上下文窗口空间。Summarizing conversation history to free up context window space. | 第9章 |
| 压缩后恢复 | Post-Compact Restore | 压缩完成后选择性恢复最关键的文件内容和技能信息。Selectively restoring the most critical file content and skill information after compaction completes. | 第10章 |
| YOLO 分类器 | YOLO Classifier | 二次 Claude API 调用用于在自动模式下做出权限批准/拒绝决策。A secondary Claude API call used to make permission approve/deny decisions in auto mode. | 第17章 |
| Feature Flag | Feature Flag (tengu_*) | 通过 GrowthBook 运行时配置的实验门控，控制功能启用/禁用。Experiment gates configured via GrowthBook runtime, controlling feature enable/disable. | 第1章、第23章 |
| Hooks | Hooks | 用户自定义的 Shell 命令，在特定事件（如工具调用前后）时执行。User-defined shell commands executed on specific events (e.g., before/after tool calls). | 第18章 |
| MCP | Model Context Protocol | 模型上下文协议，标准化 AI 模型与外部工具/数据源的交互。A protocol standardizing interaction between AI models and external tools/data sources. | 第22章 |
| Token 预算 | Token Budget | 为上下文窗口中的各类内容分配的 token 使用上限。The token usage ceiling allocated to various content types within the context window. | 第12章、第26章 |
| Bridge | Bridge | 远程会话转发架构，通过 JWT 认证将 CLI 会话投射到远程 Agent。Remote session forwarding architecture that projects CLI sessions to remote agents via JWT authentication. | 第20章 |
| 任务图 | Task DAG | 带有 blocks/blockedBy 依赖关系的任务有向无环图，Teams 调度的核心数据结构。A directed acyclic graph of tasks with blocks/blockedBy dependencies, the core data structure for Teams scheduling. | 第20b章 |
| 配置优先级 | Settings Priority | 5 层配置覆盖体系：env > MDM > user > project > defaults。A 5-layer configuration override system: env > MDM > user > project > defaults. | 附录 B |
