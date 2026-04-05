spec: task
name: "第17b章：提示注入防御 — 从 Unicode 清洗到纵深防御的七层体系"
inherits: project
tags: [ch17b, part5, security, prompt-injection, defense-in-depth]
depends: [part5-safety-permissions]
estimate: 1d
---

## 意图

Claude Code 的安全分析在 ch16（Bash 安全）和 ch17（权限系统）中已覆盖了工具执行层的安全机制，但提示注入（Prompt Injection）防御——AI Agent 面临的最独特的安全威胁——尚未系统分析。

源码中存在一套完整的七层纵深防御体系，分散在 `utils/sanitization.ts`、`utils/xml.ts`、`constants/prompts.ts`、`constants/cyberRiskInstruction.ts`、`tools/SendMessageTool`、`services/mcp/client.ts`、`services/mcp/channelNotification.ts` 等文件中。本章将这些分散的防御机制梳理为一个连贯的安全架构。

与 ch16（Bash 安全）的关系：ch16 分析的是"Agent 执行了什么命令"的安全性，本章分析的是"Agent 被输入了什么"的安全性——两者分别守卫输出和输入两端。

与 ch17（权限系统）的关系：ch17 分析的是"谁被允许做什么"的授权模型，本章分析的是"谁在说话"的认证/信任模型——权限假设身份已知，防注入解决身份伪造。

## 约束

- 章节正文 5000-7000 字
- 每层防御必须有 `restored-src/src/` 源码证据
- 至少 2 个 Mermaid 图：七层架构总览 + 攻击/防御流程图
- 引用 HackerOne #3086545 报告作为实际漏洞案例
- 不在书中暴露可直接利用的攻击方法（描述防御机制，不描述绕过方法）
- 与 ch16、ch17 交叉引用，不重复已有分析
- 不对 Anthropic 的安全决策做道德评判

## 已定决策

### 章节

- 文件：`book/src/part5/ch17b.md`
- 在 SUMMARY.md 中置于 ch17 之后、ch18 之前
- 核心论点：AI Agent 的防注入不是单一技术，而是纵深防御——从字符级清洗到架构级信任边界

### 七层防御结构

| 层 | 名称 | 防御对象 | 关键源文件 |
|----|------|---------|-----------|
| L1 | Unicode 清洗 | 隐藏字符/ASCII 走私 | `utils/sanitization.ts` |
| L2 | 提示词免疫训练 | 工具结果中的注入尝试 | `constants/prompts.ts:191` |
| L3 | XML 转义 | 结构标签伪造 | `utils/xml.ts` |
| L4 | system-reminder 信任模型 | 伪造系统消息 | `constants/prompts.ts:132` |
| L5 | 跨机器硬阻断 | 远程注入 | `tools/SendMessageTool:592` |
| L6 | 内容来源标签 | 来源混淆 | `constants/xml.ts` |
| L7 | 行为约束指令 | 越权操作 | `constants/cyberRiskInstruction.ts` |

### 章节结构

1. 为什么这很重要（AI Agent 的提示注入 vs 传统 Web 注入的本质区别）
2. 源码分析
   - 17b.1 真实漏洞：HackerOne #3086545 与 Unicode 隐形攻击
   - 17b.2 第一道防线：Unicode 清洗（sanitization.ts 的迭代式清洗）
   - 17b.3 结构防御：XML 转义与来源标签
   - 17b.4 模型层防御：提示词免疫训练与 system-reminder 信任模型
   - 17b.5 架构级防御：跨机器硬阻断与 classifierApprovable
   - 17b.6 行为边界：CYBER_RISK_INSTRUCTION
   - 17b.7 MCP 作为最大攻击面：外部工具返回的完整清洗链
3. 模式提炼
4. 用户能做什么

## 边界

### 允许修改
- `book/src/part5/ch17b.md`（新建）
- `book/src/SUMMARY.md`（添加条目）

### 禁止
- 不修改 `restored-src/` 下的任何文件
- 不修改已有章节（ch16、ch17、ch18）
- 不描述具体的攻击绕过方法
- 不在书中包含可直接利用的攻击代码

## 排除范围

- Anthropic API 层的安全机制（那是模型侧，不是客户端侧）
- 具体的 Unicode 攻击 payload 构造
- 与其他 AI Agent 框架的安全对比
- OWASP LLM Top 10 的完整覆盖（只聚焦提示注入这一项）

## 验收标准

场景: 章节文件存在且在预算内
  测试: verify_ch17b_exists
  当 检查 book/src/part5/ch17b.md
  那么 文件存在
  并且 字数在 5000-7000 范围

场景: 七层防御全部有源码证据
  测试: verify_seven_layers
  当 检查章节内容
  那么 引用了 sanitization.ts
  并且 引用了 xml.ts
  并且 引用了 prompts.ts
  并且 引用了 cyberRiskInstruction.ts
  并且 引用了 SendMessageTool
  并且 引用了 mcp/client.ts
  并且 引用了 constants/xml.ts

场景: 不包含攻击代码
  测试: verify_no_attack_code
  当 检查章节内容
  那么 不包含可直接利用的 Unicode 攻击 payload
  并且 不包含 XML 标签注入示例
