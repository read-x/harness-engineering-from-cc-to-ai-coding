spec: task
name: "交互式 D3.js 可视化 — 全书关键章节动画"
inherits: project
tags: [visualization, d3js, interactive, mdbook]
depends: [ch30-build-your-agent]
estimate: 2d
---

## 意图

为本书的关键章节创建 D3.js 交互式可视化动画，嵌入 mdbook 构建输出中。ch30 的 Agent Loop 动画已完成并验证（`book/src/part7/agent-viz.html`），本任务为其他 4 个高优先级章节创建同样质量的可视化。

目标是让读者通过交互动画理解 Claude Code 的核心机制——比静态 Mermaid 图更直观，比纯文字更具操作感。

## 已定决策

### 技术栈

- D3.js v7（CDN 加载，无本地构建）
- 自包含 HTML 文件（无外部依赖除 D3 CDN）
- 暗色主题（与 mdbook ayu 主题一致）
- 每个可视化一个独立 HTML 文件，放在对应 `partN/` 目录
- 通过章节中的超链接引用：`[点击查看交互式动画](xxx-viz.html)`

### 文件命名

- `book/src/part1/agent-loop-viz.html` — ch03 Agent Loop
- `book/src/part3/compaction-viz.html` — ch09 自动压缩
- `book/src/part4/cache-viz.html` — ch13-14 缓存命中
- `book/src/part5/permission-viz.html` — ch16-17 权限决策

### 交互规范

- 每个可视化必须有：Play / Pause / Reset / Step 控制
- 3 档速度切换（Slow / Normal / Fast）
- 点击节点/步骤展开详细信息面板
- 鼠标悬浮显示 tooltip
- 自动播放（页面加载 500ms 后开始）
- 响应式布局（viewBox 适配）

### 设计参考

- 已完成的 `book/src/part7/agent-viz.html` 作为样式和交互基准
- 颜色编码：L1 蓝 `#58a6ff`、L2 绿 `#3fb950`、L3 黄 `#d29922`、L4 红 `#f47067`、L5 紫 `#bc8cff`、L6 青 `#39d353`
- 背景 `#0d1117`，面板 `#161b22`，边框 `#30363d`

## 约束

- 每个 HTML 文件必须自包含（除 D3 CDN 外不依赖外部资源）
- 文件大小不超过 30KB（含内联 CSS/JS）
- 不使用 `document.write`、`eval` 或其他不安全的 JS 模式
- mdbook build 必须将 HTML 文件复制到输出目录
- 动画步骤必须与章节源码分析内容一致（引用的文件路径和行号匹配）

## 边界

### 允许修改
- `book/src/part1/agent-loop-viz.html`（新建）
- `book/src/part3/compaction-viz.html`（新建）
- `book/src/part4/cache-viz.html`（新建）
- `book/src/part5/permission-viz.html`（新建）
- `book/src/part1/ch03.md`（添加链接）
- `book/src/part3/ch09.md`（添加链接）
- `book/src/part4/ch13.md`（添加链接）
- `book/src/part5/ch16.md`（添加链接）

### 禁止
- 不修改已有章节的正文内容（只添加可视化链接）
- 不引入本地 JS 构建工具（webpack/vite 等）
- 不修改 book.toml 的 additional-js 配置

## 排除范围

- P2/P3 级别的可视化（ch05, ch20, ch29, ch06b, ch11）留待后续
- 移动端触控交互优化
- 无障碍访问（aria 标签等）
- 国际化（只做中文版）

## 验收标准

场景: ch03 Agent Loop 状态机动画（critical）
  测试: verify_agent_loop_viz
  假设 `book/src/part1/agent-loop-viz.html` 已创建
  当 在浏览器中打开该文件
  那么 显示 5 个状态节点（Idle/Thinking/ToolUse/ToolResult/Responding）
  并且 点击 Play 后状态按顺序高亮并转换
  并且 点击任意状态节点显示该阶段的源码引用和数据流描述
  并且 右侧或底部面板显示 token 消耗和工具调用统计

场景: ch09 自动压缩 Token 仪表盘（critical）
  测试: verify_compaction_viz
  假设 `book/src/part3/compaction-viz.html` 已创建
  当 在浏览器中打开该文件
  那么 显示 200K token 进度条
  并且 动画展示对话消息逐条添加、进度条增长
  并且 达到阈值时触发压缩动画（消息合并/消失、进度条回退）
  并且 显示压缩前后的 token 对比

场景: ch13-14 缓存命中动画
  测试: verify_cache_viz
  假设 `book/src/part4/cache-viz.html` 已创建
  当 在浏览器中打开该文件
  那么 左侧显示系统提示词的各 section（彩色块）
  并且 动画逐段比较并标记缓存命中（绿）或 miss（红）
  并且 显示 4 个 cache_control 断点位置
  并且 底部显示命中率和成本节省计算

场景: ch16-17 权限决策树
  测试: verify_permission_viz
  假设 `book/src/part5/permission-viz.html` 已创建
  当 在浏览器中打开该文件
  那么 显示权限决策流程（工具调用 → 权限检查 → YOLO 分类 → 结果）
  并且 用户可以选择不同的工具调用场景（如 Bash rm、Read file、Edit）
  并且 每个决策节点可点击查看判断逻辑
  并且 最终显示 allow/deny 结果及原因

场景: 所有 HTML 文件自包含
  测试: verify_self_contained
  假设 4 个 HTML 文件均已创建
  当 检查文件内容
  那么 每个文件除 D3 CDN 外不引用其他外部资源
  并且 每个文件可独立在浏览器中打开并正常运行
  并且 文件大小均不超过 30KB

场景: 章节链接已添加
  测试: verify_chapter_links
  假设 对应章节的 .md 文件已修改
  当 检查修改内容
  那么 ch03.md 包含指向 `agent-loop-viz.html` 的链接
  并且 ch09.md 包含指向 `compaction-viz.html` 的链接
  并且 ch13.md 包含指向 `cache-viz.html` 的链接
  并且 ch16.md 包含指向 `permission-viz.html` 的链接

场景: mdbook 构建通过
  测试: verify_mdbook_build
  假设 所有文件已就位
  当 执行 `mdbook build`
  那么 构建成功
  并且 输出目录中包含 4 个 HTML 可视化文件

场景: 可视化数据与章节一致
  测试: verify_data_consistency
  假设 4 个可视化已创建
  当 审查动画内容
  那么 Agent Loop 的状态转换顺序与 ch03 描述一致
  并且 压缩阈值（13K buffer tokens）与 ch09 源码引用一致
  并且 缓存断点设计与 ch13 的 4 断点分析一致
  并且 权限判断流程与 ch16-17 的决策逻辑一致
