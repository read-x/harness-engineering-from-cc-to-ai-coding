# 附录 B：环境变量参考

本附录列出 Claude Code v2.1.88 中用户可配置的关键环境变量。按功能域分组，仅列出影响用户可见行为的变量，省略内部遥测和平台检测类变量。

## 上下文压缩

| 变量 | 效果 | 默认值 |
|------|------|-------|
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | 覆盖上下文窗口大小（token） | 模型默认值 |
| `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` | 以百分比覆盖自动压缩阈值（0-100） | 计算值 |
| `DISABLE_AUTO_COMPACT` | 完全禁用自动压缩 | `false` |

## Effort 与推理

| 变量 | 效果 | 有效值 |
|------|------|-------|
| `CLAUDE_CODE_EFFORT_LEVEL` | 覆盖 effort 级别 | `low`、`medium`、`high`、`max`、`auto`、`unset` |
| `CLAUDE_CODE_DISABLE_FAST_MODE` | 禁用 Fast Mode 加速输出 | `true`/`false` |
| `DISABLE_INTERLEAVED_THINKING` | 禁用扩展思考 | `true`/`false` |
| `MAX_THINKING_TOKENS` | 覆盖思考 token 上限 | 模型默认值 |

## 工具与输出限制

| 变量 | 效果 | 默认值 |
|------|------|-------|
| `BASH_MAX_OUTPUT_LENGTH` | Bash 命令最大输出字符数 | 8,000 |
| `CLAUDE_CODE_GLOB_TIMEOUT_SECONDS` | Glob 搜索超时（秒） | 默认值 |

## 权限与安全

| 变量 | 效果 | 注意 |
|------|------|------|
| `CLAUDE_CODE_DUMP_AUTO_MODE` | 导出 YOLO 分类器请求/响应 | 仅调试用 |
| `CLAUDE_CODE_DISABLE_COMMAND_INJECTION_CHECK` | 禁用 Bash 命令注入检测 | 降低安全性 |

## API 与认证

| 变量 | 效果 | 安全等级 |
|------|------|---------|
| `ANTHROPIC_API_KEY` | Anthropic API 认证密钥 | 凭证 |
| `ANTHROPIC_BASE_URL` | 自定义 API 端点（代理支持） | 可重定向 |
| `ANTHROPIC_MODEL` | 覆盖默认模型 | 安全 |
| `CLAUDE_CODE_USE_BEDROCK` | 通过 AWS Bedrock 路由推理 | 安全 |
| `CLAUDE_CODE_USE_VERTEX` | 通过 Google Vertex AI 路由推理 | 安全 |
| `CLAUDE_CODE_EXTRA_BODY` | 向 API 请求追加额外字段 | 高级用途 |
| `ANTHROPIC_CUSTOM_HEADERS` | 自定义 HTTP 请求头 | 安全 |

## 模型选择

| 变量 | 效果 | 示例 |
|------|------|------|
| `ANTHROPIC_DEFAULT_HAIKU_MODEL` | 自定义 Haiku 模型 ID | 模型字符串 |
| `ANTHROPIC_DEFAULT_SONNET_MODEL` | 自定义 Sonnet 模型 ID | 模型字符串 |
| `ANTHROPIC_DEFAULT_OPUS_MODEL` | 自定义 Opus 模型 ID | 模型字符串 |
| `ANTHROPIC_SMALL_FAST_MODEL` | 快速推理模型（如用于摘要） | 模型字符串 |
| `CLAUDE_CODE_SUBAGENT_MODEL` | 子 Agent 使用的模型 | 模型字符串 |

## 提示词缓存

| 变量 | 效果 | 默认值 |
|------|------|-------|
| `CLAUDE_CODE_ENABLE_PROMPT_CACHING` | 启用提示词缓存 | `true` |
| `DISABLE_PROMPT_CACHING` | 完全禁用提示词缓存 | `false` |

## 会话与调试

| 变量 | 效果 | 用途 |
|------|------|------|
| `CLAUDE_CODE_DEBUG_LOG_LEVEL` | 日志详细程度 | `silent`/`error`/`warn`/`info`/`verbose` |
| `CLAUDE_CODE_PROFILE_STARTUP` | 启用启动性能剖析 | 调试 |
| `CLAUDE_CODE_PROFILE_QUERY` | 启用查询管线剖析 | 调试 |
| `CLAUDE_CODE_JSONL_TRANSCRIPT` | 将会话记录写为 JSONL | 文件路径 |
| `CLAUDE_CODE_TMPDIR` | 覆盖临时目录 | 路径 |

## 输出与格式

| 变量 | 效果 | 默认值 |
|------|------|-------|
| `CLAUDE_CODE_SIMPLE` | 最小系统提示词模式 | `false` |
| `CLAUDE_CODE_DISABLE_TERMINAL_TITLE` | 禁用设置终端标题 | `false` |
| `CLAUDE_CODE_NO_FLICKER` | 减少全屏模式闪烁 | `false` |

## MCP（Model Context Protocol）

| 变量 | 效果 | 默认值 |
|------|------|-------|
| `MCP_TIMEOUT` | MCP 服务器连接超时（ms） | 10,000 |
| `MCP_TOOL_TIMEOUT` | MCP 工具调用超时（ms） | 30,000 |
| `MAX_MCP_OUTPUT_TOKENS` | MCP 工具输出 token 上限 | 默认值 |

## 网络与代理

| 变量 | 效果 | 注意 |
|------|------|------|
| `HTTP_PROXY` / `HTTPS_PROXY` | HTTP/HTTPS 代理 | 可重定向 |
| `NO_PROXY` | 绕过代理的主机列表 | 安全 |
| `NODE_EXTRA_CA_CERTS` | 额外 CA 证书 | 影响 TLS 信任 |

## 路径与配置

| 变量 | 效果 | 默认值 |
|------|------|-------|
| `CLAUDE_CONFIG_DIR` | 覆盖 Claude 配置目录 | `~/.claude` |

---

## 版本演化：v2.1.91 新增变量

| 变量 | 效果 | 说明 |
|------|------|------|
| `CLAUDE_CODE_AGENT_COST_STEER` | 子代理成本引导 | 控制多代理场景下的资源消耗 |
| `CLAUDE_CODE_RESUME_THRESHOLD_MINUTES` | 会话恢复时间阈值 | 控制会话恢复的时间窗口 |
| `CLAUDE_CODE_RESUME_TOKEN_THRESHOLD` | 会话恢复 Token 阈值 | 控制会话恢复的 Token 预算 |
| `CLAUDE_CODE_USE_ANTHROPIC_AWS` | AWS 认证路径 | 启用 Anthropic AWS 基础设施认证 |
| `CLAUDE_CODE_SKIP_ANTHROPIC_AWS_AUTH` | 跳过 AWS 认证 | AWS 不可用时的回退路径 |
| `CLAUDE_CODE_DISABLE_CLAUDE_API_SKILL` | 禁用 Claude API 技能 | 企业合规场景控制 |
| `CLAUDE_CODE_PLUGIN_KEEP_MARKETPLACE_ON_FAILURE` | 插件市场容错 | 市场获取失败时保留缓存版本 |
| `CLAUDE_CODE_REMOTE_SETTINGS_PATH` | 远程设置路径覆盖 | 企业部署自定义设置 URL |

### v2.1.91 移除的变量

| 变量 | 原效果 | 移除原因 |
|------|--------|---------|
| `CLAUDE_CODE_DISABLE_COMMAND_INJECTION_CHECK` | 禁用命令注入检查 | Tree-sitter 基础设施整体移除 |
| `CLAUDE_CODE_DISABLE_MOUSE_CLICKS` | 禁用鼠标点击 | 功能废弃 |
| `CLAUDE_CODE_MCP_INSTR_DELTA` | MCP 指令增量 | 功能重构 |
