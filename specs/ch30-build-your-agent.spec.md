spec: task
name: "第30章：构建你自己的 AI Agent — Rust 代码审查 Agent 实战"
inherits: project
tags: [ch30, part7, practical, rust, code-review-agent, agent-building]
depends: [part7-lessons]
estimate: 1.5d
---

## 意图

本书前 29 章从分析视角剖析了 Claude Code 的每个子系统，提炼出 22 个命名模式/原则（ch25 6条 + ch26 6条 + ch27 8个 + 2个来自 ch6b）。本章从**应用视角**出发，用一个真正可运行的 Rust 代码审查 Agent 项目（基于 `cc-sdk` 社区 SDK），演示如何将这些模式组合应用。

章节采用"CC 源码模式分析（TypeScript）→ Rust 实现"的对照结构，确保每层都有源码证据支撑。配套代码在 `examples/code-review-agent/` 中，是可编译的 Rust 项目，不是伪代码。

## 约束

- 章节正文 5000-7000 字
- 每层（共 6 层）必须先展示 CC 源码中的模式（TypeScript，`restored-src/src/path:行号`），再展示 Rust 实现
- Rust 代码可编译（`cargo check` 通过），edition 2024
- 22 个命名模式中至少覆盖 12 个（>50%），每个引用注明来源章节
- 代码审查 Agent 只读（不修改被审查代码）
- 不重复 ch25-ch27 的模式分析，只引用并应用
- `cc-sdk` crate（crates.io）作为 Claude Code CLI 的 Rust SDK

## 已定决策

### 章节

- 文件：`book/src/part7/ch30.md`
- 风格：CC 源码模式分析（TypeScript）→ Rust 实现对照
- Mermaid 图至少 2 个：Agent 架构总览图 + 六层叠加图
- 模式引用格式："**模式名**（详见第N章）"

### 代码

- 语言：Rust（edition 2024）
- SDK：`cc-sdk`（crates.io 依赖）
- 位置：`examples/code-review-agent/`
- 六层结构：提示词 → 上下文 → 工具 → 安全 → 韧性 → 可观测性
- 核心 crate：`cc-sdk` + `clap` + `tracing` + `serde` + `tokio`

### 六层 ↔ 模式映射

| 层 | CC 子系统 | 应用的模式（来源） |
|----|-----------|-------------------|
| L1 提示词 | systemPrompt | 提示词即控制面(ch25)、带外控制信道(ch25)、工具级提示词(ch27)、范围匹配响应(ch27) |
| L2 上下文 | compaction | 为一切设定预算(ch26)、上下文卫生(ch26)、告知而非隐藏(ch26)、保守估算(ch26) |
| L3 工具 | toolSystem | 编辑前先读取(ch27)、结构化搜索(ch27) |
| L4 安全 | permissions | 失败关闭(ch25)、渐进式自主(ch27) |
| L5 韧性 | retry/degrade | 有限重试预算(ch6b)、熔断失控循环(ch26)、局部选模与能力降维(ch27) |
| L6 可观测 | telemetry | 先观察再修复(ch25)、结构化验证(ch27) |

**覆盖 16/22 = 73%**

## 边界

### 允许修改
- `book/src/part7/ch30.md`（新建）
- `book/src/SUMMARY.md`（添加 ch30 条目）
- `examples/code-review-agent/**`（新建整个目录）

### 禁止
- 不修改 `restored-src/` 下的任何文件
- 不修改已有的 ch25-ch29 章节
- 不在 examples/ 代码中包含真实 API key 或凭证
- 不引入 `unsafe` Rust 代码
- 不修改 `cc-sdk` 的源码（只通过公开 API 使用）

## 排除范围

- 完整 Agent 框架设计（只是 mini-Agent 演示）
- `cc-sdk` 的源码分析
- Agent 部署、CI/CD、Docker 化
- 与其他 AI Agent 框架（LangChain、CrewAI 等）的对比
- 性能基准测试

## 验收标准

场景: 章节文件存在且在预算内
  测试: verify_ch30_exists_and_budget
  假设 `book/src/part7/ch30.md` 已生成
  当 统计章节字数（中文按字符计）
  那么 字数在 5000-7000 范围内
  并且 章节标题包含"第30章"

场景: 章节结构完整
  测试: verify_ch30_structure
  假设 `book/src/part7/ch30.md` 已生成
  当 审阅章节结构
  那么 包含"为什么需要这一章"或等效导言部分
  并且 包含 6 个层级小节（提示词/上下文/工具/安全/韧性/可观测性）
  并且 包含"完整架构回顾"或等效总结部分
  并且 包含"模式提炼"部分（至少 2 个命名模式）
  并且 包含"用户能做什么"部分（至少 6 条建议）

场景: CC 源码引用真实有效（critical）
  标签: critical
  测试: verify_ch30_source_references
  假设 `book/src/part7/ch30.md` 已生成
  当 提取所有 `restored-src/src/` 引用
  那么 每个引用指向 `restored-src/src/` 下真实存在的文件
  并且 引用的行号范围内包含相关代码
  并且 6 个层级小节中每层至少有 1 个 CC TypeScript 源码引用

场景: 模式覆盖率达标
  测试: verify_ch30_pattern_coverage
  假设 `book/src/part7/ch30.md` 已生成
  当 统计被引用的命名模式
  那么 至少覆盖 12 个命名模式（全书 22 个中的 >50%）
  并且 每个被引用的模式注明来源章节编号（ch25/ch26/ch27/ch6b）

场景: Mermaid 图表要求
  测试: verify_ch30_diagrams
  假设 `book/src/part7/ch30.md` 已生成
  当 统计 Mermaid 代码块
  那么 至少包含 2 个 Mermaid 图
  并且 包含 Agent 架构流程图（输入→分析→输出）
  并且 包含六层叠加关系图

场景: Rust 代码可编译（critical）
  标签: critical
  测试: verify_code_compiles
  假设 `examples/code-review-agent/` 目录存在
  当 在该目录执行 `cargo check`
  那么 编译通过，无 error
  并且 `Cargo.toml` 中 `edition = "2024"`
  并且 依赖 `cc-sdk` 通过 crates.io 引用

场景: 代码结构合理
  测试: verify_code_structure
  假设 `examples/code-review-agent/` 目录存在
  当 检查目录内容
  那么 包含 `src/main.rs` 入口文件
  并且 包含至少 3 个模块文件（prompts/context/review 或等效）
  并且 包含 `Cargo.toml`
  并且 包含 `README.md`

场景: 章节代码引用与 examples/ 一致（critical）
  标签: critical
  测试: verify_ch30_code_consistency
  假设 章节和代码文件均已生成
  当 对比章节中的 Rust 代码片段与 `examples/code-review-agent/src/` 文件
  那么 章节中的代码片段来自实际 examples/ 文件
  并且 代码片段标注了源文件路径

场景: SUMMARY.md 已更新
  测试: verify_summary_updated
  假设 `book/src/SUMMARY.md` 已修改
  当 检查第七篇部分
  那么 包含第30章的条目
  并且 链接指向 `./part7/ch30.md`

场景: 代码不包含安全敏感信息
  测试: verify_no_secrets
  假设 `examples/code-review-agent/` 目录存在
  当 扫描所有源文件
  那么 不包含硬编码的 API key 或认证 token
  并且 敏感配置通过环境变量读取

场景: mdbook 构建通过
  测试: verify_mdbook_build
  假设 所有文件修改已完成
  当 在 `book/` 目录执行 `mdbook build`
  那么 构建成功，无 error

场景: 不重复已有章节分析
  测试: verify_no_redundancy
  假设 `book/src/part7/ch30.md` 已生成
  当 检查章节内容
  那么 不包含超过 3 行与 ch25/ch26/ch27 相同的源码片段
  并且 对已有模式使用交叉引用（"详见第N章"）而非重新分析
