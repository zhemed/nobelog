# Markdown 管线

`rust/src/markdown.rs` 把 Markdown 正文转成 HTML + TOC。文件头写明：本文件是 `src/markdown.ts` 的逐字节移植，
唯一例外是代码块 token 层（syntect + `highlight_map.rs` 映射到 hljs 类名），因此代码块内部高亮不保证与 Node 版逐字节一致。

## 两个入口

```rust
pub fn render(body: &str) -> (String, Vec<TocEntry>)         // 独立 renderer（单页用，可并行）
pub fn render_shared(body: &str) -> (String, Vec<TocEntry>)  // 共享 renderer（文章用，必须串行）
pub fn reset_shared();                                        // 新一次全量渲染前重置
```

- 标题 id 的 `slugCount` 是**跨文章累积**的（与 JS 版一致）。所以 `content.rs` 先并行做 IO/解析，
  再在 `load()` 的 for 循环里串行调用 `render_shared` 填 `html`/`toc`。
  **不要**把这一步改成 `par_iter`，否则同名标题的 `-1` 后缀会错位。
- 单页之间互相独立，`read_pages` 里用 `render` 并行是安全的。

## 标题与 TOC

- TOC 只收 **h2–h4**（`is_toc_level`）。
- id 生成（`slugify`）：转小写 → 空白压成单个 `-` → 只保留字母/数字/`-`/`_` → 去掉首尾 `-`
  → 按 UTF-16 截到 80 → 空则 `section` → 重名追加 `-1`、`-2`（首次出现不加后缀）。
- 中文标题保留中文（浏览器会自己做百分号编码），所以文章里锚点形如 `#二级标题文字`。
- 每个标题后面追加锚点图标：
  `<a href="#id" class="anchor text-muted"><i class="fas fa-hashtag"></i></a>`（`anchor_html`）。

## 提示块（GitHub alert 风格）

```markdown
::: tip 小技巧
正文
:::
```

- 只认 `tip` / `info` / `warning` / `danger`（大小写不敏感，`PROMPT_KINDS`）。
- 起始 `:::` 必须**位于块首**（行首或前一行为空），` ``` `/`~~~` 围栏内部不解析。
- 结束标记是"只有 `:::` 的行"（可带空白）。
- 允许 `::: tip 自定义标题`；有标题时输出 `<p><strong>标题</strong></p>`，没有标题就不输出标题块
  （没有"默认文案"）。
- 输出形态：`<blockquote type="{kind}" class="prompt-{kind}">` + 正文 + `</blockquote>`，样式由
  `patch.css` 的 `.prompt-*` 规则承担。
- 未闭合的 `:::` 会被当成普通 Markdown 行，不会吞掉后续内容。

## 代码块

- 语言名经 `safe_lang_name` 过滤（仅字母数字与 `_ + # . -`），类名形如 `language-rust`；
  行内代码固定 `language-plaintext highlighter-rouge`。
- 代码块结构固定为 `<div class="highlight"><pre class="code-block"><code class="language-xxx" data-highlighted="yes">…
  </code></pre><button class="copy-button" data-copy>…</button></div>`，内嵌 `octicon-copy` / `octicon-check` 两个 SVG。
  `rust/client/main.js` 用 `document.querySelectorAll("[data-copy]")` + `button.closest(".highlight").querySelector("code")`
  取代码，复制成功给按钮加 `.is-copied`（CSS 负责切图标）。改结构必须同步改这两处选择器。
- 原始代码块先用 `protect_raw_code_blocks` 占位保护，再做标题/表格/图片升级，最后 `restore_raw_code_blocks` 还原，
  避免把代码里的 `<h2>`、`<table>` 当 HTML 处理。

## 其它升级（`upgrade_html` 系列）

| 处理 | 函数 | 结果 |
| --- | --- | --- |
| 表格 | `wrap_tables` | 外包响应式容器，避免窄屏溢出 |
| 图片 | `wrap_images` | 外包可点击容器，配合图片浮层（lightbox） |
| 标题 | `upgrade_headings` | 加 id + 锚点，同时收集 TOC |
| 行内代码 | `upgrade_code` | 套 `highlighter-rouge` 结构 |

## 链接与安全

- `normalize_link` 处理站内绝对化等链接归一并保证与 TS 版一致。
- Markdown 里的原始 HTML 会被放行（作者即站主），因此**不要在模板层再二次信任正文 HTML**；
  正文来自 `content/` 仓库内容，不是用户提交。

## 验证

- 单文件/全量对照：`cargo run --manifest-path rust/Cargo.toml --example render`（输出 JSONL）。
- 全站级：`nobelog build` + `nobelog diff`，Markdown 改动必须让 `diff` 只剩预期差异。
- 改 `highlight.rs` / `highlight_map.rs` 时只要求 hljs 类名与结构一致，不要求 token 字节一致。

## 反模式

- 用 `str::len()`/字节切片做截断或判断（JS 语义是按 UTF-16 单元）。
- 在 `render_shared` 之外复用同一个 `Ctx` 跨文章渲染 → 标题计数错乱。
- 为了新样式改 Markdown 输出结构却不改 `patch.css` 与 `main.js` 的对应选择器。
