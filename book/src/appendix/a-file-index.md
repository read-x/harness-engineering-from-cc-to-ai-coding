# 附录 A：关键文件索引

本附录列出 Claude Code v2.1.88 源码中的关键文件及其职责，按子系统分组。文件路径相对于 `restored-src/src/`。

## 入口点与核心循环

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `main.tsx` | CLI 入口点，并行预取、延迟导入、Feature Flag 门控 | 第1章 |
| `query.ts` | Agent Loop 主循环，`queryLoop` 状态机 | 第3章 |
| `query/transitions.ts` | 循环转换类型：`Continue`、`Terminal` | 第3章 |

## 工具系统

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `Tool.ts` | 工具接口契约，`TOOL_DEFAULTS` 失败关闭默认值 | 第2章、第25章 |
| `tools.ts` | 工具注册，Feature Flag 条件加载 | 第2章 |
| `services/tools/toolOrchestration.ts` | 工具执行编排，`partitionToolCalls` 并发分区 | 第4章 |
| `services/tools/toolExecution.ts` | 单工具执行生命周期 | 第4章 |
| `services/tools/StreamingToolExecutor.ts` | 流式工具执行器 | 第4章 |
| `tools/BashTool/` | Bash 工具实现，含 Git 安全协议 | 第8章、第27章 |
| `tools/FileEditTool/` | 文件编辑工具，"编辑前先读取"强制 | 第8章、第27章 |
| `tools/FileReadTool/` | 文件读取工具，默认 2000 行 | 第8章 |
| `tools/GrepTool/` | 基于 ripgrep 的搜索工具 | 第8章 |
| `tools/AgentTool/` | 子 Agent 生成工具 | 第8章、第20章 |
| `tools/SkillTool/` | 技能调用工具 | 第8章、第22章 |
| `tools/SkillTool/prompt.ts` | 技能列表预算：1% 上下文窗口 | 第12章、第26章 |

## 系统提示词

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `constants/prompts.ts` | 系统提示词构建，`SYSTEM_PROMPT_DYNAMIC_BOUNDARY` | 第5章、第6章、第25章 |
| `constants/systemPromptSections.ts` | 段落注册表，带缓存控制 scope | 第5章 |
| `constants/toolLimits.ts` | 工具结果预算常量 | 第12章、第26章 |

## API 与缓存

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `services/api/claude.ts` | API 调用构建，缓存断点放置 | 第13章 |
| `services/api/promptCacheBreakDetection.ts` | 缓存中断检测，`PreviousState` 追踪 | 第14章、第25章 |
| `utils/api.ts` | `splitSysPromptPrefix()` 三路缓存分割 | 第5章、第13章 |

## 上下文压缩

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `services/compact/compact.ts` | 压缩编排，`POST_COMPACT_MAX_FILES_TO_RESTORE` | 第9章、第10章 |
| `services/compact/autoCompact.ts` | 自动压缩阈值与熔断器 | 第9章、第25章、第26章 |
| `services/compact/prompt.ts` | 压缩提示词模板 | 第9章、第28章 |
| `services/compact/microCompact.ts` | 基于时间的微压缩 | 第11章 |
| `services/compact/apiMicrocompact.ts` | API 原生缓存微压缩 | 第11章 |

## 权限与安全

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `utils/permissions/yoloClassifier.ts` | YOLO 自动模式分类器 | 第17章 |
| `utils/permissions/denialTracking.ts` | 拒绝追踪，`DENIAL_LIMITS` | 第17章、第27章 |
| `tools/BashTool/bashPermissions.ts` | Bash 命令权限检查 | 第16章 |

## CLAUDE.md 与技能

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `utils/claudemd.ts` | CLAUDE.md 加载与注入，4 层优先级 | 第19章 |
| `skills/bundled/` | 内置技能目录 | 第22章 |
| `skills/loadSkillsDir.ts` | 用户自定义技能发现 | 第22章 |
| `skills/mcpSkillBuilders.ts` | MCP 到技能桥接 | 第22章 |

## 多 Agent 编排

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `coordinator/coordinatorMode.ts` | 协调器模式实现 | 第20章 |
| `utils/teammate.ts` | 队友 Agent 工具 | 第20章 |
| `utils/swarm/teammatePromptAddendum.ts` | 队友提示词附加内容 | 第20章 |

## 工具结果与存储

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `utils/toolResultStorage.ts` | 大结果持久化，截断预览 | 第12章、第28章 |
| `utils/toolSchemaCache.ts` | 工具 Schema 缓存 | 第15章 |

## 跨会话记忆

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `memdir/memdir.ts` | MEMORY.md 索引与主题文件加载，注入系统提示词 | 第24章 |
| `memdir/paths.ts` | 记忆目录路径解析，三级优先链 | 第24章 |
| `services/extractMemories/extractMemories.ts` | Fork agent 自动提取记忆 | 第24章 |
| `services/SessionMemory/sessionMemory.ts` | 滚动会话摘要，用于压缩 | 第24章 |
| `utils/sessionStorage.ts` | JSONL 会话记录存储与恢复 | 第24章 |
| `tools/AgentTool/agentMemory.ts` | 子 Agent 持久化与 VCS 快照 | 第24章 |
| `services/autoDream/autoDream.ts` | 夜间记忆整合与修剪 | 第24章 |

## 遥测与可观测性

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `services/analytics/index.ts` | 事件入口，队列-附着模式，PII 标记类型 | 第29章 |
| `services/analytics/sink.ts` | 双路分发（Datadog + 1P），采样 | 第29章 |
| `services/analytics/firstPartyEventLogger.ts` | OTel BatchLogRecordProcessor 集成 | 第29章 |
| `services/analytics/firstPartyEventLoggingExporter.ts` | 自定义 Exporter，磁盘持久化重试 | 第29章 |
| `services/analytics/metadata.ts` | 事件元数据，工具名清洗，PII 分级 | 第29章 |
| `services/analytics/datadog.ts` | Datadog 允许列表，批量刷新 | 第29章 |
| `services/analytics/sinkKillswitch.ts` | 远程熔断（tengu_frond_boric） | 第29章 |
| `services/api/logging.ts` | API 三事件模型（query/success/error） | 第29章 |
| `services/api/withRetry.ts` | 重试遥测，网关指纹检测 | 第29章 |
| `utils/debug.ts` | 调试日志，--debug 标志 | 第29章 |
| `utils/diagLogs.ts` | PII-free 容器诊断 | 第29章 |
| `utils/errorLogSink.ts` | 错误文件日志 | 第29章 |
| `utils/telemetry/sessionTracing.ts` | OTel span，三级追踪 | 第29章 |
| `utils/telemetry/perfettoTracing.ts` | Perfetto 可视化追踪 | 第29章 |
| `utils/gracefulShutdown.ts` | 级联超时优雅关闭 | 第29章 |
| `cost-tracker.ts` | 成本追踪，会话间持久化 | 第29章 |

## 配置与状态

| 文件 | 职责 | 相关章节 |
|------|------|---------|
| `utils/effort.ts` | Effort 级别解析 | 第21章 |
| `utils/fastMode.ts` | Fast Mode 管理 | 第21章 |
| `utils/managedEnvConstants.ts` | 托管环境变量白名单 | 附录 B |
| `screens/REPL.tsx` | 主交互界面（5000+ 行 React 组件） | 第1章 |
