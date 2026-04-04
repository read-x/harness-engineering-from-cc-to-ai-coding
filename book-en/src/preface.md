# Preface

<p align="center">
  <img src="./assets/cover-en.jpeg" alt="Harness Engineering cover" width="420">
</p>

<p align="center">
  <a href="../">Read the Chinese edition</a>
</p>

*Harness Engineering* — known informally in Chinese as "The Horse Book" (because the Chinese title sounds like "harness" as in horse harness).

I believe the best way to "consume" the Claude Code source code is to transform it into a book for systematic learning. For me, learning from a book is more comfortable than reading raw source code, and it makes it easier to form a complete cognitive framework.

So I had Claude Code extract a book from the leaked TypeScript source code. The book is now open-sourced, and everyone can read it online:

- Repository: <https://github.com/ZhangHanDong/harness-engineering-from-cc-to-ai-coding>
- Read online: <https://zhanghandong.github.io/harness-engineering-from-cc-to-ai-coding/>

If you want to read the book while gaining a more intuitive understanding of Claude Code's internal mechanisms, pairing it with this visualization site is highly recommended:

- Visualization site: <https://ccunpacked.dev>

To ensure the best possible AI writing quality, the extraction process was not as simple as "throw the source code at the model and let it generate." Instead, it followed a fairly rigorous engineering workflow:

1. First, discuss and clarify `DESIGN.md` based on the source code — that is, establish the outline and design of the entire book.
2. Then write specs for each chapter, using my open-source `agent-spec` to constrain chapter objectives, boundaries, and acceptance criteria.
3. Next, create a plan, breaking down the specific execution steps.
4. Finally, layer on my own technical writing skill before having the AI begin formal writing.

This book is not intended for publication — it's meant to help me learn Claude Code more systematically. My basic judgment is: AI certainly won't write a perfect book, but as long as the initial version is open-sourced, everyone can read, discuss, and gradually improve it together, co-building it into a truly valuable public-domain book.

That said, objectively speaking, this initial version is already quite well-written. Contributions and discussions are welcome. Rather than creating a separate discussion group, all related conversations are hosted on GitHub Discussions:

- Discussions: <https://github.com/ZhangHanDong/harness-engineering-from-cc-to-ai-coding/discussions>
