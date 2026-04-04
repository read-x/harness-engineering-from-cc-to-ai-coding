spec: project
name: "驾驭工程 — 从 Claude Code 源码到 AI 编码最佳实践"
tags: [book, harness-engineering, claude-code]
---

## 意图

基于 Claude Code v2.1.88 的逆向工程源码（1,902 个 TypeScript 文件），编写一本系统性的技术书籍。书的核心价值在于：从真实的生产级 AI 编码产品源码中提取可复用的驾驭工程（Harness Engineering）模式，为 AI Agent 构建者提供实战指南。

全书 7 篇 25 章 + 3 个附录，总计约 15-20 万字。每章以源码中的真实代码片段为证据，避免空泛的理论。

## 约束

- 所有技术论断必须有源码证据支撑，引用格式为 `文件路径:行号`
- 代码片段使用实际源码，不使用伪代码（除非对比假想的 Rust 实现）
- 每章包含：导言（为什么这很重要）、源码分析（实际怎么做的）、模式提炼（你能学到什么）
- 中文写作，技术术语保留英文原文并在首次出现时附中文注释
- 章节间不重复相同的源码分析，但可交叉引用
- 不对 Anthropic 做道德评判，聚焦于工程实践本身

## 已定决策

- 输出格式：每章一个 Markdown 文件，放在 `book/src/partN/` 目录（mdbook 结构）
- 文件命名：`chNN.md`（如 `book/src/part1/ch01.md`）
- 源码引用：使用 `restored-src/src/` 下的实际文件路径
- 写作顺序：按篇分批，每批并行写作同一篇内的各章
- 图表：使用 Mermaid 语法内嵌，不使用外部图片
- 代码块：标注语言（typescript）和源文件路径注释

## 边界

### 允许修改
- book/src/**
- docs/book-outline.md
- specs/**

### 禁止
- 不修改 restored-src/ 下的任何源码文件
- 不在书中包含可能的安全凭证、API key、内部 Slack channel ID
- 不发明源码中不存在的功能或行为

## 排除范围

- Claude Code 的 UI/UX 设计分析（仅关注工程实现）
- Bun 运行时的 Zig 源码分析（仅分析 TypeScript 层）
- 与竞品（Cursor、Copilot 等）的功能对比
- 商业策略分析
