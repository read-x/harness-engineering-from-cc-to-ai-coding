spec: task
name: "书籍整体改进 v2 — 从分析走向综合"
inherits: project
tags: [book, structure, content, quality, v2-improvements]
depends: [book-improvements, version-evolution, interactive-viz, ch30-build-your-agent]
estimate: 5d
---

## 意图

基于对全书 30 章 + 5 附录（18,898 行）和 CC 源码（1,884 文件、512K LOC）的全面交叉分析，执行一组尚未被现有 spec 覆盖的改进。

核心判断：书的分析深度已达 8.5/10，最大的提升杠杆在于从模块化分析走向系统性综合——让读者不仅理解每个子系统，还能看到它们协同工作的全貌。本 spec 覆盖 5 个批次共 10 项改进，预计新增约 10,000 字内容。

## 约束

- 不重复 `book-improvements.spec.md` 中已定义的改进（ch20 拆分、ch06b、ch13-15/ch23/ch24 深化等）
- 不重复 `interactive-viz.spec.md` 中的 D3.js 可视化工作
- 不重复 `ch30-build-your-agent.spec.md` 中的实战章节
- 所有新增内容遵循 `project.spec.md` 的写作规范（源码引用格式、Mermaid 图表、中文写作等）
- 每个批次完成后执行 `mdbook build` 验证
- 新增内容字数预算：前言 ~800 字、定位段 ~90 字（30×3行）、附录 F ~5500 字、Bridge ~1500 字、Settings ~800 字、State ~500 字

## 已定决策

### 批次划分与优先级

| 批次 | 内容 | 优先级 | 工作量 |
|------|------|--------|--------|
| 1 | 前言扩展 + 每章定位锚点 | P0 | 0.5d |
| 2 | 附录 F：3 个端到端案例追踪 | P0 | 1.5d |
| 3 | 源码盲区填补（Bridge/Settings/State） | P1 | 1d |
| 4 | 早期章节版本演进标注 | P1 | 0.5d |
| 5 | CI 质量检查 + 术语表双语化 + 可视化静态 fallback | P2 | 1.5d |

### 批次 1：前言扩展

- 新增前置知识清单（TypeScript 阅读能力、CLI 概念、LLM API 基础）
- 新增 3 条推荐阅读路径：Agent 构建者 / 安全工程师 / 性能优化
- 新增 1 个 Mermaid 全书知识地图（7 篇依赖关系 + ch03 锚点标注）
- 新增章节标记说明（源码引用格式、证据分级、可视化说明）
- 说明 `ch20b`/`ch20c`/`ch22b` 是深化章节的编号惯例

### 批次 2：附录 F 端到端案例

- 3 个案例放在 `book/src/appendix/f-e2e-traces.md`，不新增正式章节
- 案例 1：`/commit` 命令全链路（ch03→ch05→ch04→ch16→ch17→ch13），含 Mermaid 序列图
- 案例 2：触发 auto-compaction 的长对话（ch09→ch10→ch11→ch12→ch13→ch26），含 token 时间线图
- 案例 3：多 Agent 协作（ch20→ch20b→ch05→ch25），含 Agent 通信序列图
- 每个案例追踪实际源码路径，不使用伪代码

### 批次 3：源码盲区

- Bridge 架构：在 ch20 末尾新增 "远程执行：Bridge 架构" 小节（~1500 字），覆盖 `bridge/` 目录
- Settings 层级：在附录 B 末尾新增 "配置优先级体系" 小节（~800 字），覆盖 `utils/settings/`
- State Management：在 ch01 的三层架构小节新增 AppStateStore 说明（~500 字），覆盖 `state/AppStateStore.tsx`

### 批次 4：版本演进标注

- 为 ch01、ch02、ch03、ch07、ch08 添加版本演进声明（每章 3 行）
- ch04、ch05、ch06 已有版本演化小节，不重复

### 批次 5：基础设施

- CI 质量检查脚本 `scripts/check-chapter-quality.sh`：检查 Mermaid 图数量、源码引用数量、路径有效性
- 术语表双语化：附录 C 升级为中英对照格式
- 可视化静态 fallback：确保 ch03/ch09/ch13/ch16 有 Mermaid 替代图

### 不改编号

保持 `ch20b`/`ch20c`/`ch22b` 编号不变——已有 spec 和交叉引用广泛使用，改动代价大于收益。在前言中说明即可。

## 边界

### 允许修改
- `book/src/preface.md`
- `book/src/part*/ch*.md`（所有章节，仅追加定位段和版本声明）
- `book/src/appendix/*.md`（所有附录）
- `book/src/SUMMARY.md`（添加附录 F 条目）
- `scripts/check-chapter-quality.sh`（新建）
- `.github/workflows/pages.yml`（添加检查步骤）

### 禁止
- 不修改 `restored-src/` 下的任何文件
- 不重新编号已有章节
- 不修改已有章节的核心源码分析内容
- 不重复 `book-improvements.spec.md` 已覆盖的工作

## 验收标准

场景: 前言包含前置知识清单（critical）
  标签: critical
  测试: verify_preface_prerequisites
  假设 `book/src/preface.md` 已修改
  当 检查前言内容
  那么 包含"前置知识"或"阅读准备"标题
  并且 提及 TypeScript、CLI、LLM API 三个领域
  并且 前言总行数 >= 100

场景: 前言包含 3 条阅读路径
  测试: verify_preface_reading_paths
  假设 `book/src/preface.md` 已修改
  当 检查前言内容
  那么 包含"Agent 构建者"阅读路径
  并且 包含"安全工程师"阅读路径
  并且 包含"性能优化"阅读路径
  并且 每条路径列出至少 5 个章节编号

场景: 前言包含全书知识地图
  测试: verify_preface_knowledge_map
  假设 `book/src/preface.md` 已修改
  当 检查前言内容
  那么 包含至少 1 个 Mermaid 代码块
  并且 Mermaid 图引用 7 个篇章

场景: 至少 25 章有定位锚点段
  测试: verify_chapter_anchors
  假设 所有章节文件已修改
  当 统计包含"定位"blockquote 的章节数
  那么 数量 >= 25

场景: 每个定位段包含前置依赖信息
  测试: verify_anchor_content
  假设 所有章节文件已修改
  当 检查定位 blockquote 内容
  那么 每个定位段提及"前置依赖"或"适用场景"

场景: 附录 F 包含 3 个端到端案例（critical）
  标签: critical
  测试: verify_appendix_f_cases
  假设 `book/src/appendix/f-e2e-traces.md` 已创建
  当 检查内容
  那么 包含"git commit"或"/commit"案例
  并且 包含"自动压缩"或"auto-compaction"案例
  并且 包含"多 Agent"或"多代理"案例

场景: 每个案例串联至少 3 章引用
  测试: verify_e2e_cross_references
  假设 `book/src/appendix/f-e2e-traces.md` 已创建
  当 统计每个案例中的"第N章"引用
  那么 案例 1 引用至少 3 个不同章节
  并且 案例 2 引用至少 3 个不同章节
  并且 案例 3 引用至少 3 个不同章节

场景: 每个案例包含 Mermaid 图
  测试: verify_e2e_diagrams
  假设 `book/src/appendix/f-e2e-traces.md` 已创建
  当 统计 Mermaid 代码块
  那么 至少包含 3 个 Mermaid 图

场景: 每个案例包含源码路径引用
  测试: verify_e2e_source_refs
  假设 `book/src/appendix/f-e2e-traces.md` 已创建
  当 统计 `restored-src/src/` 引用
  那么 至少 6 个不同的源码路径

场景: SUMMARY.md 包含附录 F 条目
  测试: verify_summary_appendix_f
  假设 `book/src/SUMMARY.md` 已修改
  当 检查附录部分
  那么 包含"附录 F"条目
  并且 链接指向 `./appendix/f-e2e-traces.md`

场景: Bridge 架构小节存在
  测试: verify_bridge_section
  假设 `book/src/part6/ch20.md` 已修改
  当 检查新增内容
  那么 包含"Bridge"或"远程执行"标题
  并且 包含 `bridge/` 目录下的源码引用
  并且 包含至少 1 个 Mermaid 图

场景: Bridge 小节引用至少 3 个 bridge/ 源文件
  测试: verify_bridge_source_depth
  假设 `book/src/part6/ch20.md` 已修改
  当 统计 `bridge/` 路径引用
  那么 引用至少 3 个不同的 `bridge/` 下文件

场景: Settings 层级小节存在
  测试: verify_settings_hierarchy
  假设 `book/src/appendix/b-env-vars.md` 已修改
  当 检查新增内容
  那么 包含"配置优先级"或"Settings 层级"标题
  并且 包含 5 层优先级表（env/MDM/user/project/defaults）
  并且 包含 `utils/settings/` 源码引用

场景: State Management 说明存在
  测试: verify_state_management
  假设 `book/src/part1/ch01.md` 已修改
  当 检查新增内容
  那么 包含"AppStateStore"或"状态管理"标题
  并且 包含 `state/AppStateStore.tsx` 源码引用

场景: 早期章节有版本演进标注
  测试: verify_early_chapter_evolution
  假设 ch01, ch02, ch03, ch07, ch08 已修改
  当 检查各文件末尾
  那么 ch01 包含"版本演化"或"版本演进"文字
  并且 ch02 包含"版本演化"或"版本演进"文字
  并且 ch03 包含"版本演化"或"版本演进"文字
  并且 ch07 包含"版本演化"或"版本演进"文字
  并且 ch08 包含"版本演化"或"版本演进"文字

场景: CI 质量检查脚本可执行
  测试: verify_quality_script_executable
  假设 `scripts/check-chapter-quality.sh` 已创建
  当 检查文件权限
  那么 文件具有可执行权限
  并且 脚本包含 Mermaid 图计数逻辑
  并且 脚本包含 `restored-src/` 路径验证逻辑

场景: CI 质量检查脚本对所有章节报告结果
  测试: verify_quality_script_coverage
  假设 `scripts/check-chapter-quality.sh` 已创建
  当 执行该脚本
  那么 输出覆盖至少 30 个章节文件
  并且 报告每章的 Mermaid 图数量
  并且 报告每章的源码引用数量

场景: 术语表包含双语格式
  测试: verify_glossary_bilingual
  假设 `book/src/appendix/c-glossary.md` 已修改
  当 检查表格结构
  那么 表头包含"中文"和"英文"列（或等效双语标识）
  并且 至少 30 个术语有双语条目

场景: 可视化章节有静态 Mermaid 替代
  测试: verify_viz_static_fallback
  假设 ch03, ch09, ch13, ch16 已检查
  当 检查每个包含交互式动画链接的章节
  那么 每个动画链接附近存在 Mermaid 代码块作为静态替代

场景: 前言包含章节标记说明
  测试: verify_preface_notation
  假设 `book/src/preface.md` 已修改
  当 检查前言内容
  那么 包含源码引用格式说明（提及 `restored-src/src/`）
  并且 包含证据分级说明（提及 v2.1.88 或 bundle 逆向）

场景: 附录 F 案例使用实际源码路径而非伪代码
  测试: verify_e2e_real_source_paths
  假设 `book/src/appendix/f-e2e-traces.md` 已创建
  当 检查所有代码块
  那么 代码块标注 `typescript` 语言
  并且 代码块包含 `restored-src/src/` 路径注释
  但是 不包含 `// pseudocode` 或 `// 伪代码` 标记

场景: ch04/ch05/ch06 不重复添加版本演进标注
  测试: verify_no_duplicate_evolution
  假设 批次 4 版本演进标注已完成
  当 检查 ch04, ch05, ch06
  那么 不新增额外的"版本演化说明"标题（它们已有版本演化小节）

场景: 术语表双语格式完整
  测试: verify_glossary_bilingual_format
  假设 `book/src/appendix/c-glossary.md` 已修改
  当 检查表格第一行
  那么 表头同时包含中文术语列和英文术语列
  并且 每行术语条目包含中文定义和英文对应词

场景: mdbook 构建通过（critical）
  标签: critical
  测试: verify_mdbook_build
  假设 所有文件修改已完成
  当 在 `book/` 目录执行 `mdbook build`
  那么 构建成功，无 error

## 排除范围

- 英文版内容翻译（仅做术语表双语化准备）
- Pagefind 中文分词配置（需实际测试，另开 spec）
- 版本追踪 CI 自动化（`version-evolution.spec.md` 已覆盖）
- `ch20b`/`ch20c`/`ch22b` 重新编号（代价大于收益）
- 已被其他 spec 覆盖的改进项（ch20 拆分、ch06b、ch30 等）
