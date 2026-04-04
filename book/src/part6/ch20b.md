# 第20b章：Teams 与多进程协作

> **定位**：本章分析 Claude Code 的 Swarm 团队协作机制——平面结构的多 Agent 协作模型。前置依赖：第20章。适用场景：想深入了解 CC 的 Swarm 团队协作机制——包括 TaskList 调度、DAG 依赖、Mailbox 通信的读者。

## 为什么单独讨论 Teams

第20章介绍了 Claude Code 的三种 Agent 派生模式——子 Agent、Fork 和协调者——它们的共同点是"父派生子"的层级关系。Teams（队友系统）是一个不同的维度：它创建一个**平面结构的团队**，团队中的 Agent 通过消息传递协作，而非层级调用。这种差异不仅体现在架构上，更体现在通信协议、权限同步和生命周期管理等工程实现中。

---

## 20b.1 队友 Agent（Agent Swarms）

队友系统是 Agent 编排的另一个维度。与子 Agent 的"父派生子"模型不同，队友系统创建一个**平面结构的团队**，团队中的 Agent 通过消息传递协作。

### TeamCreateTool：团队创建

`TeamCreateTool`（`tools/TeamCreateTool/TeamCreateTool.ts`）用于创建新团队：

```typescript
// tools/TeamCreateTool/TeamCreateTool.ts:37-49
const inputSchema = lazySchema(() =>
  z.strictObject({
    team_name: z.string().describe('Name for the new team to create.'),
    description: z.string().optional(),
    agent_type: z.string().optional()
      .describe('Type/role of the team lead'),
  }),
)
```

团队信息持久化到 `TeamFile` 中，包含团队名称、成员列表、Leader 信息等。团队名称需要唯一——如果冲突则自动生成一个 word slug（第 64-72 行）。

### TeammateAgentContext：队友上下文

队友使用 `TeammateAgentContext` 类型（`agentContext.ts` 第 60-85 行），包含丰富的团队协调信息：

```typescript
// utils/agentContext.ts:60-85
export type TeammateAgentContext = {
  agentId: string          // 完整 ID，如 "researcher@my-team"
  agentName: string        // 显示名称，如 "researcher"
  teamName: string         // 所属团队
  agentColor?: string      // UI 颜色
  planModeRequired: boolean // 是否需要计划审批
  parentSessionId: string  // Leader 的会话 ID
  isTeamLead: boolean      // 是否是 Leader
  agentType: 'teammate'
}
```

队友的 ID 格式是 `name@team-name`，这种格式使得在日志和通信中可以一眼看出 Agent 的身份和归属。

### 平面结构约束

队友系统有一个重要的架构约束：**队友不能派生其他队友**（第 272-274 行）：

```typescript
// tools/AgentTool/AgentTool.tsx:272-274
if (isTeammate() && teamName && name) {
  throw new Error('Teammates cannot spawn other teammates — the team roster is flat.');
}
```

这是刻意的设计——团队名册是一个扁平数组，嵌套的队友会导致名册中出现没有来源信息的条目，混淆 Leader 的协调逻辑。

同样，进程内队友（in-process teammate）不能派生后台 Agent（第 278-280 行），因为它们的生命周期绑定在 Leader 的进程上。

---

## 20b.2 Agent 间通信

### SendMessageTool：消息路由

`SendMessageTool`（`tools/SendMessageTool/SendMessageTool.ts`）是 Agent 间通信的核心。它的 `to` 字段支持多种寻址方式：

```typescript
// tools/SendMessageTool/SendMessageTool.ts:69-76
to: z.string().describe(
  feature('UDS_INBOX')
    ? 'Recipient: teammate name, "*" for broadcast, "uds:<socket-path>" for a local peer, or "bridge:<session-id>" for a Remote Control peer'
    : 'Recipient: teammate name, or "*" for broadcast to all teammates',
),
```

消息类型是一个判别联合（第 47-65 行），支持：
- 纯文本消息
- 关闭请求（`shutdown_request`）
- 关闭响应（`shutdown_response`）
- 计划审批响应（`plan_approval_response`）

### 广播机制

当 `to` 为 `"*"` 时触发广播（`handleBroadcast`，第 191-266 行）：遍历团队文件中的所有成员（排除发送者自己），逐一写入邮箱。广播结果包含接收者列表，方便协调者跟踪。

### 邮箱系统

消息实际通过 `writeToMailbox()` 函数写入文件系统邮箱。每条消息包含：发送者名称、文本内容、摘要、时间戳和发送者颜色。这种基于文件系统的邮箱设计使得跨进程的队友（tmux 模式）可以通过共享文件系统通信。

### UDS_INBOX：Unix Domain Socket 扩展

当 `UDS_INBOX` Feature Flag 启用时，`SendMessageTool` 的寻址能力扩展到 Unix Domain Socket：`"uds:<socket-path>"` 可以向同一机器上的其他 Claude Code 实例发送消息，`"bridge:<session-id>"` 可以向 Remote Control 对等端发送消息。

这创建了一个超越单一团队边界的通信拓扑：

```
┌─────────────────────────────────────────────────────────────────┐
│                    Agent 间通信架构                              │
│                                                                 │
│  ┌──────────────────────────────────┐                          │
│  │        Team "my-team"            │                          │
│  │                                  │                          │
│  │  ┌─────────┐    MailBox    ┌─────────┐                     │
│  │  │ Leader  │◄────────────►│Teammate │                     │
│  │  │ (lead)  │   (文件系统)  │  (dev)  │                     │
│  │  └────┬────┘              └─────────┘                     │
│  │       │                                                    │
│  │       │ SendMessage(to: "tester")                         │
│  │       │                                                    │
│  │       ▼                                                    │
│  │  ┌─────────┐                                              │
│  │  │Teammate │                                              │
│  │  │ (tester)│                                              │
│  │  └─────────┘                                              │
│  └──────────────────────────────────┘                          │
│         │                                                      │
│         │ SendMessage(to: "uds:/tmp/other.sock")              │
│         ▼                                                      │
│  ┌──────────────┐                                              │
│  │ 其他 Claude  │    SendMessage(to: "bridge:<session>")       │
│  │ Code 实例    │──────────────────────────►  Remote Control   │
│  └──────────────┘                                              │
└─────────────────────────────────────────────────────────────────┘
```

### 协调者模式下的 Worker 结果回传

在协调者模式中，Worker 完成任务后的结果以 `<task-notification>` XML 格式作为**用户角色消息**注入协调者的对话中（`coordinatorMode.ts` 第 148-159 行）：

```xml
<task-notification>
  <task-id>{agentId}</task-id>
  <status>completed|failed|killed</status>
  <summary>{人类可读的状态摘要}</summary>
  <result>{Agent 的最终文本响应}</result>
  <usage>
    <total_tokens>N</total_tokens>
    <tool_uses>N</tool_uses>
    <duration_ms>N</duration_ms>
  </usage>
</task-notification>
```

协调者提示词明确要求（第 144 行）："它们看起来像用户消息但不是。通过 `<task-notification>` 开始标签区分它们。"这种设计避免了协调者把 Worker 结果当作用户输入来回应。

---

## 20b.3 真正的调度内核：TaskList、Claim Loop 与 Idle Hooks

如果只看到 `TeamCreateTool`、`SendMessageTool` 和 Mailbox，很容易把 Teams 理解成"一组能互发消息的 Agent"。但 Claude Code 的 Swarm 真正有价值的地方，不是聊天，而是**共享任务图**。`TeamCreate` 的提示词直接写明了这一点：`Teams have a 1:1 correspondence with task lists (Team = TaskList)`。创建团队时，`TeamCreateTool` 不只写 `TeamFile`，还会重置并创建对应的任务目录，然后把 Leader 的 `taskListId` 绑定到团队名上。这意味着 Teams 从一开始就不是"先有团队，任务只是附属品"，而是**团队和任务表是同一个运行时对象的两个视图**。

### Task 不是 Todo，而是 DAG 节点

`utils/tasks.ts` 中的 `Task` 结构包含：

```typescript
{
  id: string,
  owner?: string,
  status: 'pending' | 'in_progress' | 'completed',
  blocks: string[],
  blockedBy: string[],
}
```

这里最关键的不是 `status`，而是 `blocks` 和 `blockedBy`。它们把任务列表从普通的 Todo 清单提升成一个**显式依赖图**：某个任务只有在所有 blocker 都完成后才算可执行。这种设计让 Leader 可以先创建整批有依赖关系的工作项，再把"什么时候可以并行"交给运行时，而不必在提示词里反复口头协调。

这也是为什么 `TeamCreate` 的提示词会强调："teammates should check TaskList periodically, especially after completing each task, to find available work or see newly unblocked tasks"。Claude Code 并不要求每个队友都拥有一份完整的全局计划推理能力；它要求队友**回到共享任务图上读状态**。

### 自动 Claim：Swarm 的最小调度器

真正把这张任务图驱动起来的是 `useTaskListWatcher.ts`。这个 watcher 会在任务目录变化或 Agent 重新空闲时触发一次检查，自动挑选一个可工作的任务：

- `status === 'pending'`
- `owner` 为空
- `blockedBy` 中的任务都已完成

源码中的 `findAvailableTask()` 正是按这个条件筛选。找到任务后，运行时先 `claimTask()` 抢占 owner，再把任务格式化成 prompt 交给 Agent 执行；如果提交失败，还会释放 claim。这里有两个很重要的工程含义：

1. **调度和推理分离**。模型不需要自己在自然语言里判断"哪个任务现在没被别人做、而且依赖已经解开"；运行时先把候选工作缩到一个明确任务。
2. **并行来自共享状态，而不是消息协商**。多个 Agent 能同时推进，不是因为它们彼此足够聪明，而是因为 claim + blocker 检查把冲突显式编码进了状态机。

从这个角度看，Claude Code 的 Swarm 其实已经具备一个很小但完整的调度器：**任务图 + 原子 claim + 状态转移**。Mailbox 只是协作补充，不是主调度面。

### 回合结束后的事件面：TaskCompleted 与 TeammateIdle

Swarm 的另一个关键点是：队友在一轮执行结束后，不是简单"停下"，而是进入事件驱动的收尾阶段。`query/stopHooks.ts` 里，当当前执行者是 teammate 时，Claude Code 会在普通 Stop hooks 之后继续运行两类专用事件：

- `TaskCompleted`：对当前队友拥有的 `in_progress` 任务触发完成钩子
- `TeammateIdle`：队友进入空闲状态时触发钩子

这使得 Teams 不只是一个 pull-based 系统，也不是纯 push-based 系统，而是两者叠加：

- **pull**：空闲队友回到 TaskList，继续 claim 新任务
- **push**：任务完成和队友空闲会触发事件，通知 Leader 或驱动后续自动化

换句话说，Claude Code 的 Swarm 不是"一群会发消息的 agent"，而是**共享任务图 + durable mailbox + 回合结束事件**共同构成的协作内核。

### 这不是共享内存，而是共享状态

这里有一个措辞要非常小心。Teams 看起来像"多个 Agent 共享一个工作区"，但按源码更准确的说法不是"共享内存"，而是三层共享状态：

- **共享任务状态**：`~/.claude/tasks/{team-name}/`
- **共享通信状态**：`~/.claude/teams/{team}/inboxes/*.json`
- **共享团队配置**：`~/.claude/teams/{team}/config.json`

In-Process teammate 只是在物理运行位置上变成同进程，并通过 `AsyncLocalStorage` 保存自己的身份上下文；它没有把整个系统提升成一个通用 blackboard shared-memory runtime。这个区分很重要，因为它决定了 Claude Code Swarm 的真正可迁移模式：**先把协作状态外化，再让不同执行单元围绕它协作**。

---

## 20b.4 异步 Agent 的生命周期

当 `shouldRunAsync` 为 `true` 时（由 `run_in_background`、`background: true`、协调者模式、Fork 模式、助手模式等任一条件触发，第 567 行），Agent 进入异步生命周期：

1. **注册**：`registerAsyncAgent()` 创建后台任务记录，分配 `agentId`
2. **执行**：在 `runWithAgentContext()` 包裹下运行 `runAgent()`
3. **进度上报**：通过 `updateAsyncAgentProgress()` 和 `onProgress` 回调更新状态
4. **完成/失败**：调用 `completeAsyncAgent()` 或 `failAsyncAgent()`
5. **通知**：`enqueueAgentNotification()` 将结果注入调用者的消息流

关键的设计选择：后台 Agent 不与父 Agent 的 `abortController` 关联（第 694-696 行注释）——当用户按 ESC 取消主线程时，后台 Agent 继续运行。它们只能通过 `chat:killAgents` 显式终止。

### Worktree 隔离

当 `isolation: 'worktree'` 时，Agent 在临时 git worktree 中运行（第 590-593 行）：

```typescript
const slug = `agent-${earlyAgentId.slice(0, 8)}`;
worktreeInfo = await createAgentWorktree(slug);
```

Agent 完成后，如果 worktree 没有变更（与创建时的 HEAD commit 比较），则自动清理（第 666-679 行）。有变更的 worktree 会被保留，其路径和分支名返回给调用者。

---

## 20b.5 Teams 实现细节：后端、通信、权限与记忆

> 本节是 20b.1（队友概述）的实现层深入。20b.1 回答"Teams 是什么"——平面结构团队、TeamCreateTool、TeammateAgentContext 类型；本节回答"Teams 怎么跑起来"——进程管理、通信协议、权限同步、共享记忆的具体工程实现。
>
> 源码中 "Swarm" 和 "Team" 是同义词：目录叫 `utils/swarm/`，工具叫 `TeamCreateTool`，Feature Flag 叫 `ENABLE_AGENT_SWARMS`，常量叫 `SWARM_SESSION_NAME = 'claude-swarm'`。

### 三种后端、一个接口

Teams 支持三种物理后端，统一在 `PaneBackend` + `TeammateExecutor` 接口之后（`utils/swarm/backends/types.ts`）：

| 后端 | 进程模型 | 通信机制 | 适用场景 |
|------|---------|---------|---------|
| **Tmux** | 独立 CLI 进程，tmux 分屏显示 | 文件系统 Mailbox | 默认后端，适用于 Linux/macOS |
| **iTerm2** | 独立 CLI 进程，iTerm2 分屏 | 文件系统 Mailbox | macOS 原生终端用户 |
| **In-Process** | 同进程 `AsyncLocalStorage` 隔离 | AppState 内存队列 | 无 tmux/iTerm2 环境 |

后端检测优先级链（`backends/registry.ts`）：

```
1. 在 tmux 内运行？          → Tmux（原生）
2. 在 iTerm2 内且 it2 可用？  → iTerm2（原生）
3. 在 iTerm2 但无 it2？       → 提示安装 it2
4. 系统有 tmux？              → Tmux（外部会话）
5. 都没有？                   → In-Process 回退
```

这种策略模式的好处：Leader 的 `TeamCreateTool` 和 `SendMessageTool` 不需要知道队友运行在哪种后端——`spawnTeammate()` 自动选择最佳方案。

### 团队生命周期

```typescript
// utils/swarm/teamHelpers.ts — TeamFile 结构
{
  name: string,                    // 唯一团队名
  description?: string,
  createdAt: number,
  leadAgentId: string,             // 格式：team-lead@{teamName}
  members: [{
    agentId: string,               // 格式：{name}@{teamName}
    name: string,
    agentType?: string,
    model?: string,
    prompt: string,
    color: string,                 // 自动分配的终端颜色
    planModeRequired: boolean,
    tmuxPaneId?: string,
    sessionId?: string,
    backendType: BackendType,
    isActive: boolean,
    mode: PermissionMode,
  }]
}
```

存储位置：`~/.claude/teams/{teamName}/config.json`

**队友生成流程**（`spawnMultiAgent.ts:305-539`）：

1. 检测后端 → 生成唯一名称 → 格式化 agent ID（`{name}@{teamName}`）
2. 分配终端颜色 → 创建 tmux/iTerm2 分屏
3. 构建继承的 CLI 参数：`--agent-id`、`--agent-name`、`--team-name`、`--agent-color`、`--parent-session-id`、`--permission-mode`
4. 构建继承的环境变量 → 发送启动命令到分屏
5. 更新 TeamFile → 通过 Mailbox 发送初始指令
6. 注册进程外任务追踪

**平面结构约束**：队友不能生成子队友（`AgentTool.tsx:266-300`）。这不是技术限制——是有意的组织原则：协调集中在 Leader，避免形成无限深度的委托链。

### Mailbox 通信协议

队友间通过文件系统邮箱异步通信（`teammateMailbox.ts`）：

```
~/.claude/teams/{teamName}/inboxes/{agentName}.json
```

**并发控制**：async lockfile + 指数退避（10 次重试，5-100ms 延迟窗口）

**消息结构**：

```typescript
type TeammateMessage = {
  from: string,      // 发送者名称
  text: string,      // 消息内容或 JSON 控制消息
  timestamp: string,
  read: boolean,      // 标记已读
  color?: string,     // 发送者终端颜色
  summary?: string,   // 5-10 词摘要
}
```

**控制消息类型**（嵌套在 `text` 字段中的结构化 JSON）：

| 类型 | 方向 | 用途 |
|------|------|------|
| `idle` 通知 | Teammate → Leader | 队友完成工作，报告原因（available/error/shutdown/completed） |
| `shutdown_request` | Leader → Teammate | 请求队友优雅关闭 |
| `shutdown_response` | Teammate → Leader | 批准或拒绝关闭请求 |
| `plan_approval_response` | Leader → Teammate | 审批或拒绝队友提交的计划 |

**Idle 通知结构**（`teammateMailbox.ts`）：

```typescript
type IdleNotificationMessage = {
  type: 'idle',
  teamName: string,
  agentName: string,
  agentId: string,
  idleReason: 'available' | 'error' | 'shutdown' | 'completed',
  summary?: string,           // 工作摘要
  peerDmSummary?: string,     // 最近收到的私信摘要
  errorDetails?: string,
}
```

### 权限同步：Leader 代理审批

队友不能自行审批危险工具调用——必须通过 Leader 代理（`utils/swarm/permissionSync.ts`）：

```
~/.claude/teams/{teamName}/permissions/
  ├── pending/     # 等待审批的请求
  └── resolved/    # 已处理的请求
```

**请求流程**：

```
Worker 遇到权限检查
  ↓
创建 SwarmPermissionRequest（含 toolName, input, suggestions）
  ↓
写入 pending/{requestId}.json + 发送到 Leader Mailbox
  ↓
Leader 轮询 Mailbox → 检测到权限请求 → 展示给用户
  ↓
用户在 Leader 终端审批/拒绝
  ↓
写入 resolved/{requestId}.json
  ↓
Worker 轮询 resolved/ → 获取结果 → 继续执行
```

这种设计确保了即使队友运行在独立进程中，所有危险操作仍然经过人类审批。

### 团队记忆（Team Memory）

Feature gate `TENGU_HERRING_CLOCK` 控制。位于：

```
~/.claude/projects/{project}/memory/team/MEMORY.md
```

与个人记忆（`~/.claude/projects/{project}/memory/`）独立，团队所有成员共享。使用与个人记忆相同的两步写入流程：先写 `.md` 文件，再更新 `MEMORY.md` 索引。

**路径安全验证**（`memdir/teamMemPaths.ts`，PSR M22186 安全补丁）：

| 攻击类型 | 防护 |
|---------|------|
| Null byte 注入 | 拒绝含 `\0` 的路径 |
| URL 编码遍历 | 拒绝 `%2e%2e%2f` 等模式 |
| Unicode 正规化攻击 | 拒绝全角 `．．／` 等变体 |
| 反斜杠遍历 | 拒绝含 `\` 的路径 |
| 符号链接循环 | 检测 ELOOP + 悬空链接 |
| 路径逃逸 | 解析 realpath 验证最深存在祖先的包含关系 |

### In-Process Teammates：无 tmux 的团队协作

当环境无 tmux/iTerm2 时，队友在同一进程内以 `AsyncLocalStorage` 隔离运行（`utils/swarm/spawnInProcess.ts`）：

```typescript
// AsyncLocalStorage 上下文隔离
type TeammateContext = {
  agentId: string,
  agentName: string,
  teamName: string,
  parentSessionId: string,
  isInProcess: true,
  abortController: AbortController,  // 独立取消控制
}

runWithTeammateContext<T>(context, fn: () => T): T  // 隔离执行
```

In-Process 队友的任务状态（`InProcessTeammateTaskState`）包含：

- `pendingUserMessages: string[]` — 消息队列（替代文件 Mailbox）
- `awaitingPlanApproval: boolean` — Plan 模式下等待 Leader 审批
- `isIdle: boolean` — 空闲状态
- `onIdleCallbacks: Array<() => void>` — 空闲时回调（通知 Leader）
- `messages: Message[]` — UI 显示缓冲（上限 `TEAMMATE_MESSAGES_UI_CAP = 50`）

与 tmux 队友的关键区别：通信通过内存队列而非文件 Mailbox，但 API 完全一致。

### 模式提炼：基于文件系统的进程间协作

Teams 的通信设计做了一个反直觉但务实的选择：**用文件系统而非 IPC/RPC 做跨进程通信**。

| 维度 | 文件 Mailbox | 传统 IPC/RPC |
|------|-------------|-------------|
| 持久性 | 进程崩溃后消息不丢失 | 连接断开即丢失 |
| 调试性 | 直接 `cat` 查看 | 需要专用调试工具 |
| 并发控制 | lockfile | 内置于协议 |
| 延迟 | 轮询间隔（毫秒级） | 即时 |
| 跨机器 | 需要共享文件系统 | 原生支持 |

对于 Agent Teams 的场景（秒级交互、进程可能崩溃、需要人类调试），文件 Mailbox 的权衡是合理的——UDS 作为补充方案覆盖低延迟场景。

---

## 用户能做什么

**利用 Teams 系统提升多 Agent 协作效率：**

1. **注意 Agent 间通信的寻址方式**。`SendMessageTool` 支持名称寻址（`"tester"`）、广播（`"*"`）和 UDS 寻址（`"uds:<path>"`）。理解这些寻址方式有助于设计更高效的多 Agent 工作流。

2. **理解 Teams 的后端选择**。如果你使用 tmux 或 iTerm2，队友会以独立终端分屏运行，通过文件 Mailbox 通信；无终端复用器时则回退到进程内模式。了解这一点有助于调试队友间的通信问题。

3. **利用 Idle 检测判断队友状态**。Leader 通过轮询 Mailbox 中的 idle 通知来感知队友状态。如果队友似乎"卡住了"，检查 `~/.claude/teams/{teamName}/inboxes/` 下的邮箱文件可以帮助定位问题。

4. **权限审批集中在 Leader**。所有队友的危险操作都需要通过 Leader 终端审批。确保 Leader 终端保持活跃，否则队友会因等待审批而阻塞。
