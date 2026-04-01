# 前言

<p align="center">
  <img src="./assets/cover-zh.jpeg" alt="《马书》封面" width="420">
</p>

《驾驭工程》，中文别名《马书》。

我认为，Claude Code 源码最佳的“食用”姿势应该是转化为一本书，供自己系统学习。对我来说，看书学习比直接看源码更舒服，也更容易形成完整的认知框架。

所以，我让 Claude Code 从泄露出来的 TypeScript 源码里提取出一本书。现在这本书已经开源，大家可以在线阅读：

- 仓库地址：<https://github.com/ZhangHanDong/harness-engineering-from-cc-to-ai-coding>
- 在线阅读：<https://zhanghandong.github.io/harness-engineering-from-cc-to-ai-coding/>

为了尽可能保证 AI 写作质量，这本书的提取过程并不是“把源码丢给模型直接生成”那么简单，而是按一条比较严格的工程流程推进的：

1. 先根据源码把 `DESIGN.md` 聊清楚，也就是先把整本书的大纲和设计定下来。
2. 然后为每一章编写 spec，基于我开源的 `agent-spec` 来约束章节目标、边界和验收标准。
3. 接着再做 plan，把具体执行步骤拆开。
4. 最后再叠加我自己的技术写作 skill，才让 AI 开始正式写作。

这本书并不是为了出版，而是为了让我能更系统地学习 Claude Code。我对它的基本判断是：AI 肯定写不得十全十美，但只要把初始版本开源出来，大家就可以一边阅读、一边讨论、一边逐步完善它，把它共建成一本真正有价值的公版书。

不过，客观地说，现在这个初始版本其实已经写得还不错了。欢迎大家交流和贡献。这里不单独建交流群，相关讨论就放在 GitHub Discussions：

- Discussions：<https://github.com/ZhangHanDong/harness-engineering-from-cc-to-ai-coding/discussions>
