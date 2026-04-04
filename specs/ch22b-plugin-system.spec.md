spec: task
name: "第22b章：插件系统 — 从打包到市场的扩展工程"
inherits: project
tags: [ch22b, part6, plugin, marketplace, trust, manifest]
depends: [part6-advanced-subsystems]
estimate: 1d
---

## 意图

ch22 分析了技能系统（Skills），但 Claude Code 的扩展机制远不止 Markdown 技能文件。插件（Plugin）是更上层的打包单元——它把 skills、hooks、commands、agents、MCP/LSP servers、output styles 捆绑成一个可发现、可安装、可信任、可更新的包。本章从源码出发，分析插件系统的完整架构：清单验证、生命周期管理、信任模型、市场发现、选项存储、自动更新和推荐机制。

与 ch22（技能系统）的关系：ch22 分析的是插件内部的一种组件（skill），本章分析的是容器本身和围绕容器的工程基础设施。

## 约束

- 章节正文 6000-8000 字
- 每个子系统（清单、生命周期、信任、市场、选项、更新、推荐）至少有 1 段源码分析
- 源码引用格式：`restored-src/src/path:行号`
- 至少 2 个 Mermaid 图：插件生命周期流程图 + 插件组件组成图
- 与 ch18（Hooks）、ch22（Skills）的交叉引用，不重复已有分析
- 不对 Anthropic 做道德评判

## 已定决策

### 章节

- 文件：`book/src/part6/ch22b.md`
- 在 SUMMARY.md 中置于 ch22 之后
- 核心论点：插件不是"技能的容器"，而是一套完整的扩展治理系统

### 源码关键文件

| 子系统 | 关键源文件 |
|--------|-----------|
| 类型定义 | `types/plugin.ts` |
| 清单 Schema | `utils/plugins/schemas.ts` |
| 加载器 | `utils/plugins/pluginLoader.ts` |
| Hook 加载 | `utils/plugins/loadPluginHooks.ts` |
| 命令加载 | `utils/plugins/loadPluginCommands.ts` |
| 内置插件 | `plugins/builtinPlugins.ts` |
| 安装管理 | `utils/plugins/installedPluginsManager.ts` |
| 操作层 | `services/plugins/pluginOperations.ts` |
| 市场管理 | `utils/plugins/marketplaceManager.ts` |
| 选项存储 | `utils/plugins/pluginOptionsStorage.ts` |
| 自动更新 | `utils/plugins/pluginAutoupdate.ts` |
| 信任对话框 | `components/TrustDialog/TrustDialog.tsx` |
| 推荐系统 | `hooks/usePluginRecommendationBase.tsx` |
| 命令迁移 | `commands/createMovedToPluginCommand.ts` |
| 验证工具 | `commands/plugin/ValidatePlugin.tsx` |
| 错误类型 | `types/plugin.ts`（PluginError，22+ 变体） |

### 章节结构

1. 为什么这很重要（插件 vs 技能 vs 独立工具的定位）
2. 源码分析
   - 2.1 插件清单：1682 行 Zod Schema 的设计
   - 2.2 生命周期：从发现到组件加载的 5 阶段
   - 2.3 信任模型：分层信任与安装前审计
   - 2.4 市场系统：发现、安装和依赖解析
   - 2.5 选项存储：敏感值分流到安全存储
   - 2.6 自动更新与推荐：三种推荐来源
   - 2.7 错误治理：22 种错误变体的类型安全处理
   - 2.8 命令迁移模式：从内置到插件的渐进演化
3. 模式提炼
4. 用户能做什么

## 边界

### 允许修改
- `book/src/part6/ch22b.md`（新建）
- `book/src/SUMMARY.md`（添加条目）

### 禁止
- 不修改 `restored-src/` 下的任何文件
- 不修改已有章节（ch18、ch22 等）
- 不在书中包含 API key 或凭证

## 排除范围

- 插件 UI 组件的 React 实现细节（只分析业务逻辑）
- 具体市场中有哪些插件（只分析市场机制）
- 与其他扩展系统（VS Code extensions、npm packages）的对比

## 验收标准

场景: 章节文件存在且在预算内
  测试: book/src/part6/ch22b.md 存在
  测试: 字数在 6000-8000 范围
  测试: 至少 2 个 Mermaid 图
  测试: 至少引用 10 个不同的源文件
  测试: 与 ch18、ch22 有交叉引用且不重复 >3 行代码
