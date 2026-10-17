# highlight-golden.json：冻结语料的兼容 fixture

## 它是什么

`highlight-golden.json` 是 **highlight.js token 层** 的 golden 快照，不是通用高亮实现：

- 键：`sha256(lang_or_empty + "\0" + code)`（hex），`lang` 保留作者写法（`SQL` / `shell` / `sh` / …）；
  未标语言 / `language-auto` 的块用**空字符串**作为 lang，生成时走 `hljs.highlightAuto`。
- 值：highlight.js 11.12 对该段代码输出的 token HTML，且与 `src/markdown.ts` 一致地去掉内联 `style`。

markdown 的正文结构（fence、表格、blockquote、标题锚点、TOC、`upgradeHtml`）全部由
`src/markdown.rs` 真移植；只有「一段代码 → hljs token HTML」这一层用 fixture，
因为 highlight.js 的 193 种语言文法依赖 JS 正则的 lookaround，Rust 侧逐字节复刻不现实
（详见 `src/highlight.rs` 顶部说明）。

## 生成方式（作者期，构建期不依赖 Node）

```bash
cd /root/博客/nobelog
node rust/tools/gen-highlight-golden.mjs
```

脚本用 Node 24 直接 import `src/markdown.ts`，拦截 `hljs.highlight` / `hljs.highlightAuto`
把冻结语料（`content/posts/*.md` + `content/pages/*.md`）的 80 个代码块固化成表。
当前语料命中率 **80/80**。

## 未命中行为

新文章/新代码块未命中时，`src/highlight.rs` 退回 highlight.js 的 `escapeHTML`
（`& < > " '` 转义，语言 class 仍保留在代码块壳上），并只打一条 warning；构建不失败，
也不做模糊/近似匹配。新增内容后重跑生成脚本即可补表。
