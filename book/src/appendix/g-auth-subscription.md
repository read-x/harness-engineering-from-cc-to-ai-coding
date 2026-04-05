# 附录 G：认证与订阅系统 — 从 OAuth 到合规边界

> 本附录基于 Claude Code v2.1.88 源码分析其认证架构和订阅系统，并结合 2026 年 4 月 Anthropic 封禁第三方工具事件，分析开发者构建 Agent 时的合规边界。

## G.1 双轨 OAuth 认证架构

Claude Code 支持两条截然不同的认证路径，服务两类用户群体。

### G.1.1 Claude.ai 订阅用户

订阅用户（Pro/Max/Team/Enterprise）通过 Claude.ai 的 OAuth 端点认证：

```
用户 → claude login → claude.com/cai/oauth/authorize
  → 授权页面（PKCE flow）
  → 回调 → exchangeCodeForTokens()
  → OAuth access_token + refresh_token
  → 直接用 token 调用 Anthropic API（无需 API key）
```

```typescript
// restored-src/src/constants/oauth.ts:18-20
const CLAUDE_AI_INFERENCE_SCOPE = 'user:inference'
const CLAUDE_AI_PROFILE_SCOPE = 'user:profile'
```

关键 scope：
- `user:inference` — 调用模型的权限
- `user:profile` — 读取账户信息
- `user:sessions` — 会话管理
- `user:mcp` — MCP 服务器访问
- `user:file_upload` — 文件上传

OAuth 配置（`restored-src/src/constants/oauth.ts:60-234`）：

| 配置项 | 生产值 |
|--------|-------|
| 授权 URL | `https://claude.com/cai/oauth/authorize` |
| Token URL | `https://platform.claude.com/v1/oauth/token` |
| Client ID | `9d1c250a-e61b-44d9-88ed-5944d1962f5e` |
| PKCE | 必须（S256） |

### G.1.2 Console API 用户

Console 用户（按量付费）通过 Anthropic 开发平台认证：

```
用户 → claude login → platform.claude.com/oauth/authorize
  → 授权（scope: org:create_api_key）
  → 回调 → exchangeCodeForTokens()
  → OAuth token → createAndStoreApiKey()
  → 生成临时 API key → 用 key 调用 API
```

区别：Console 用户多了一步——OAuth 后创建 API key，实际 API 调用走 key 认证而非 token 认证。

### G.1.3 第三方提供商

除了 Anthropic 自有认证，Claude Code 还支持：

| 提供商 | 环境变量 | 认证方式 |
|--------|---------|---------|
| AWS Bedrock | `CLAUDE_CODE_USE_BEDROCK=1` | AWS 凭证链 |
| GCP Vertex AI | `CLAUDE_CODE_USE_VERTEX=1` | GCP 凭证 |
| Azure Foundry | `CLAUDE_CODE_USE_FOUNDRY=1` | Azure 凭证 |
| 直接 API key | `ANTHROPIC_API_KEY=sk-...` | 直接传递 |
| API Key Helper | `apiKeyHelper` 配置 | 自定义命令 |

```typescript
// restored-src/src/utils/auth.ts:208-212
type ApiKeySource =
  | 'ANTHROPIC_API_KEY'     // 环境变量
  | 'apiKeyHelper'          // 自定义命令
  | '/login managed key'    // OAuth 生成的 key
  | 'none'                  // 无认证
```

## G.2 订阅层级与速率限制

### G.2.1 四级订阅

源码中的订阅判断函数（`restored-src/src/utils/auth.ts:1662-1711`）揭示了完整的层级体系：

| 层级 | 组织类型 | 速率倍数 | 价格（月） |
|------|---------|---------|-----------|
| **Pro** | `claude_pro` | 1x | $20 |
| **Max** | `claude_max` | 5x 或 20x | $100 / $200 |
| **Team** | `claude_team` | 5x（Premium） | 按席位 |
| **Enterprise** | `claude_enterprise` | 自定义 | 按合同 |

```typescript
// restored-src/src/utils/auth.ts:1662-1711
function getSubscriptionType(): 'max' | 'pro' | 'team' | 'enterprise' | null
function isMaxSubscriber(): boolean
function isTeamPremiumSubscriber(): boolean  // Team with 5x rate limit
function getRateLimitTier(): string  // e.g., 'default_claude_max_20x'
```

### G.2.2 速率限制层

`getRateLimitTier()` 返回的值直接影响 API 调用频率上限：

- `default_claude_max_20x` — Max 最高档，20 倍默认速率
- `default_claude_max_5x` — Max 标准档 / Team Premium
- 默认 — Pro 和普通 Team

### G.2.3 Extra Usage（额外用量）

某些操作会触发额外计费（`restored-src/src/utils/extraUsage.ts:4-24`）：

```typescript
function isBilledAsExtraUsage(): boolean {
  // 以下情况触发 Extra Usage 计费：
  // 1. Claude.ai 订阅用户使用 Fast Mode
  // 2. 使用 1M 上下文窗口模型（Opus 4.6, Sonnet 4.6）
}
```

支持的计费类型：
- `stripe_subscription` — 标准 Stripe 订阅
- `stripe_subscription_contracted` — 合同制
- `apple_subscription` — Apple IAP
- `google_play_subscription` — Google Play

## G.3 Token 管理与安全存储

### G.3.1 Token 生命周期

```
获取 token → 存储到 macOS Keychain → 使用时从 Keychain 读取
  → 过期前 5 分钟自动刷新 → 刷新失败重试（最多 3 次）
  → 全部失败 → 提示用户重新登录
```

关键实现（`restored-src/src/utils/auth.ts`）：

```typescript
// 过期检查：5 分钟缓冲
function isOAuthTokenExpired(token): boolean {
  return token.expires_at < Date.now() + 5 * 60 * 1000
}

// 自动刷新
async function checkAndRefreshOAuthTokenIfNeeded() {
  // 带重试逻辑的 token 刷新
  // 失败时清除缓存，下次调用重新获取
}
```

### G.3.2 安全存储

- **macOS**：Keychain Services（加密存储）
- **Linux**：libsecret / 文件系统回退
- **子进程传递**：通过 File Descriptor（`CLAUDE_CODE_API_KEY_FILE_DESCRIPTOR`），避免环境变量泄露
- **API Key Helper**：支持自定义命令获取 key，默认 5 分钟缓存 TTL

### G.3.3 登出清理

`performLogout()`（`restored-src/src/commands/logout/logout.tsx:16-48`）执行完整清理：

1. 刷新遥测数据（确保不丢失）
2. 移除 API key
3. 擦除 Keychain 中的所有凭证
4. 清除配置中的 OAuth 账户信息
5. 可选：清除 onboarding 状态
6. 失效所有缓存：OAuth token、用户数据、beta feature、GrowthBook、policy limits

## G.4 权限与角色

OAuth profile 返回的组织角色决定了用户的能力边界：

```typescript
// restored-src/src/utils/billing.ts
// Console 计费访问
function hasConsoleBillingAccess(): boolean {
  // 需要：非订阅用户 + admin 或 billing 角色
}

// Claude.ai 计费访问
function hasClaudeAiBillingAccess(): boolean {
  // Max/Pro 自动有
  // Team/Enterprise 需要 admin, billing, owner, 或 primary_owner
}
```

| 能力 | 需要的角色 |
|------|-----------|
| 访问 Console 计费 | admin 或 billing（非订阅用户） |
| 访问 Claude.ai 计费 | Max/Pro 自动有；Team/Enterprise 需 admin/billing/owner |
| 超额用量开关 | Claude.ai 订阅 + 支持的 billingType |
| `/upgrade` 命令 | 非 Max 20x 用户 |

## G.5 遥测与账户追踪

认证系统与遥测深度集成（`restored-src/src/services/analytics/metadata.ts`）：

- `isClaudeAiAuth` — 是否使用 Claude.ai 认证
- `subscriptionType` — 用于 DAU-by-tier 分析
- `accountUuid` / `emailAddress` — 遥测 header 中传递

关键分析事件：
```
tengu_oauth_flow_start          → 开始 OAuth 流程
tengu_oauth_success             → OAuth 成功
tengu_oauth_token_refresh_success/failure → token 刷新结果
tengu_oauth_profile_fetch_success → 获取 profile 成功
```

## G.6 合规边界分析

### G.6.1 背景：2026 年 4 月 OpenClaw 事件

Anthropic 于 2026 年 4 月正式封禁第三方工具通过 OAuth 使用订阅额度。核心原因：

1. **成本不可承受**：OpenClaw 等工具 7×24 运行自动化 Agent，每天消耗 $1,000-5,000 API 成本，$200/月的 Max 订阅无法覆盖
2. **绕过缓存优化**：Claude Code 的四层 prompt cache（详见第 13-14 章）能降低 90% 成本，第三方工具直接调 API 全是 cache miss
3. **条款修改**：OAuth `user:inference` scope 限定为官方产品使用

### G.6.2 行为分类

| 行为 | 技术实现 | 风险等级 |
|------|---------|---------|
| 手动使用 Claude Code CLI | `claude` 命令行交互 | **安全** — 官方产品的设计用途 |
| 脚本调用 `claude -p` | Shell 脚本自动化 | **安全** — 官方支持的非交互模式 |
| cc-sdk 启动 claude 子进程 | `cc_sdk::query()` / `cc_sdk::llm::query()` | **低风险** — 走 CLI 完整管线（含缓存） |
| MCP Server 被 Claude Code 调用 | rmcp / MCP 协议 | **安全** — 官方扩展机制 |
| Agent SDK 构建个人工具 | `@anthropic-ai/claude-code` SDK | **安全** — 官方 SDK 的设计用途 |
| 提取 OAuth token 直调 API | 绕过 Claude Code CLI | **高风险** — 这是被封的行为 |
| CI/CD 中自动化运行 | `claude -p` 在 CI 中 | **灰色** — 取决于频率和用途 |
| 分发依赖 claude 的开源工具 | 用户自行认证 | **灰色** — 取决于使用方式 |
| 7×24 自动化 daemon | 持续消耗订阅额度 | **高风险** — OpenClaw 模式 |

### G.6.3 关键区分：走不走 Claude Code 的基础设施

这是最核心的判断标准：

```
安全路径:
  你的代码 → cc-sdk → claude CLI 子进程 → CC 基础设施（含缓存）→ API
  ↑ 走了 prompt cache，Anthropic 的成本可控

危险路径:
  你的代码 → 提取 OAuth token → 直接调 Anthropic API
  ↑ 绕过 prompt cache，每次请求都是 full price
```

Claude Code 的 `getCacheControl()` 函数（`restored-src/src/services/api/claude.ts:358-374`）精心设计了全局/组织/会话三级缓存断点。通过 CLI 发送的请求自动享受这套缓存优化。直接调 API 的第三方工具无法复用这些缓存——这正是成本问题的根源。

**一键判断：是否 spawn `claude` 子进程？**

这是最简洁的合规判断标准。所有通过 `claude` CLI 子进程通信的方式都走了 CC 的完整基础设施（prompt cache + 遥测 + 权限检查），Anthropic 的成本可控；直接调 API 则绕过了一切。

| 方式 | spawn process? | 合规 |
|------|:---:|------|
| cc-sdk `query()` | 是 — `Command::new("claude")` | 合规 |
| cc-sdk `llm::query()` | 是 — 同上，加 `--tools ""` | 合规 |
| Agent SDK (`@anthropic-ai/claude-code`) | 是 — 官方 SDK spawn claude | 合规 |
| `claude -p "..."` Shell 脚本 | 是 | 合规 |
| MCP Server 被 CC 调用 | 是 — CC 发起的 | 合规 |
| 提取 OAuth token → `fetch("api.anthropic.com")` | **否** — 绕过 CLI | **违规** |
| OpenClaw 等第三方 Agent | **否** — 直接调 API | **违规** |

### G.6.4 本书示例代码的合规性

本书第 30 章的 Code Review Agent 使用以下方式：

| 后端 | 实现方式 | 合规性 |
|------|---------|--------|
| `CcSdkBackend` | cc-sdk 启动 claude CLI 子进程 | **合规** — 走官方 CLI |
| `CcSdkWsBackend` | WebSocket 连接 CC 实例 | **合规** — 走官方协议 |
| `CodexBackend` | Codex 订阅（OpenAI，非 Anthropic） | **无关** — 不涉及 Anthropic |
| MCP Server 模式 | Claude Code 通过 MCP 调用 | **合规** — 官方扩展机制 |

**建议**：
1. 不要从 `~/.claude/` 提取 OAuth token 用于其他用途
2. 不要构建 7×24 运行的自动化 daemon
3. 保留 `CodexBackend` 作为不依赖 Anthropic 订阅的替代方案
4. 如果需要高频自动化，使用 API key 按量付费而非订阅

## G.7 关键环境变量索引

| 变量 | 用途 | 来源 |
|------|------|------|
| `ANTHROPIC_API_KEY` | 直接 API key | 用户设置 |
| `CLAUDE_CODE_OAUTH_REFRESH_TOKEN` | 预认证 refresh token | 自动化部署 |
| `CLAUDE_CODE_OAUTH_SCOPES` | Refresh token 的 scope | 配合上条使用 |
| `CLAUDE_CODE_ACCOUNT_UUID` | 账户 UUID（SDK 调用者） | SDK 集成 |
| `CLAUDE_CODE_USER_EMAIL` | 用户邮箱（SDK 调用者） | SDK 集成 |
| `CLAUDE_CODE_ORGANIZATION_UUID` | 组织 UUID | SDK 集成 |
| `CLAUDE_CODE_USE_BEDROCK` | 启用 AWS Bedrock | 第三方集成 |
| `CLAUDE_CODE_USE_VERTEX` | 启用 GCP Vertex AI | 第三方集成 |
| `CLAUDE_CODE_USE_FOUNDRY` | 启用 Azure Foundry | 第三方集成 |
| `CLAUDE_CODE_API_KEY_FILE_DESCRIPTOR` | API key 的文件描述符 | 安全传递 |
| `CLAUDE_CODE_CUSTOM_OAUTH_URL` | 自定义 OAuth 端点 | FedStart 部署 |
