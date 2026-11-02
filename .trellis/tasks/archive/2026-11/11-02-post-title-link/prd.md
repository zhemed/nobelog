# 文章标题与面包屑可点击回干净地址

## Goal

读者点击目录锚点后，地址栏会保留 `#锚点` 后缀，只有浏览器后退能去掉。把「面包屑里的当前文章名」与「文章大标题」变成指向文章自身干净地址（`/posts/<slug>/`）的链接，点一下即可清除锚点。

## Requirements

- R1 模板：`rust/src/templates.rs` 中文章页面包屑末项由纯文本改为链接（`href` 指向 `post_url(post)`）；`breadcrumb()` 在「末项且有 href 且面包屑项数 > 1」时渲染为 `<a>`。
- R2 文章大标题 `<h1 data-toc-skip>` 内包 `<a href="{post_url}">`。
- R3 样式：大标题链接继承标题颜色与字重，无下划线；面包屑链接沿用既有样式。
- R4 其它页面（归档、分类、标签、404、关于）面包屑保持纯文本。
- R5 不改 Markdown 正文渲染与 TOC 生成。
- R6 验证：点击大标题或面包屑当前项后 `location.hash` 变空；静态产物仅两个文章页有结构变化（外加 CSS 哈希与 feed 时间）。

## Acceptance Criteria

- [ ] 文章页面包屑末项渲染为 `<a href="/posts/<slug>/">`。
- [ ] 文章大标题渲染为 `<h1><a href="/posts/<slug>/">标题</a></h1>`，视觉与纯文本一致。
- [ ] 从带锚点的地址点击任一处后，URL 不含 `#`。
- [ ] 其它页面面包屑仍为纯文本。
- [ ] `fmt` / `clippy` / `test` 全绿；构建通过。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`rust/src/templates.rs`（面包屑末项支持链接、文章页面包屑带 href、`<h1>` 内包链接）+ `vendor/patches/patch.css`（标题自链接样式）。

- 面包屑渲染：`breadcrumb()` 在「末项有 href 且项数 > 1」时输出 `<a>`；空 href 的其它页面（归档/分类/标签/404/关于）仍为纯文本，实测 `/archives/` 面包屑仍是 `<span>归档</span>`。
- 文章页产物：
  - 面包屑：`<span><a href="/posts/typeset-test/">全要素排版测试</a></span>`
  - 标题：`<h1 data-toc-skip><a href="/posts/typeset-test/">全要素排版测试</a></h1>`
- 交互实测（浏览器）：先点目录项得到 `#二级标题代码块`，再点大标题或面包屑当前项，`location.url` 变回 `http://10.0.0.91:10086/posts/typeset-test/`（无 `#`）。
- 样式：大标题链接与纯标题一致（`rgb(17,20,24)`、无下划线）；面包屑链接沿用既有样式。
- 静态构建：归一化 CSS 哈希后，42 个文件中仅 2 个文章页有结构变化（外加 `feed.xml` 的 `lastBuildDate`）。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
