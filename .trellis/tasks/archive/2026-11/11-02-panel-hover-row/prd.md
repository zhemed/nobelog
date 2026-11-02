# 面板 hover 改由整行驱动加粗

## Goal

面板列表的浅灰底由 `li:hover` 触发，但加粗/变深挂在 `a:hover` 上。链接是行内元素，鼠标落在行内空白（文字右侧）时只有底色、文字不变粗，与归档行的整行手感不一致。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 把 `#panel-wrapper .post-content a:hover` 的选择器改为 `#panel-wrapper .post-content li:hover a`，让整行任意位置都能触发加粗与变深。
- R3 保留 `!important`（原有面板规则带 `!important`）。
- R4 不改底色规则、不改变文字位置、不影响标签胶囊。
- R5 验证：鼠标停在行内文字右侧空白处时，链接字重 700、颜色 `#111418`，与停在文字上一致。

## Acceptance Criteria

- [x] 悬停行内空白处时链接 `font-weight 700` / `rgb(17,20,24)`。
- [x] 悬停文字上效果相同。
- [x] 底色与圆角不变；标签胶囊不受影响。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +2 / −1 行。

```css
/* 之前 */
#panel-wrapper .post-content a:hover { … }
/* 之后 */
#panel-wrapper .post-content li:hover a { … }
```

原因：`<a>` 是行内元素，宽度只覆盖文字；鼠标在行内空白处时 `a:hover` 不命中，
只有 `li:hover` 的底色生效，于是出现「底色有、字不加粗」的半套效果。

实测（鼠标位于文字右缘外侧 40px 的空白处，`x=1126`）：

| 检查 | 结果 |
| --- | --- |
| `li:hover` | true |
| `a:hover` | **false** |
| 行底色 | `rgb(238,241,245)` |
| 链接字重 / 颜色 | **700** / `rgb(17,20,24)` |

- 悬停文字本身时效果相同（`li:hover` 同样成立）。
- 其余列表本来就是行驱动（`#archives ul li:hover a`、`#page-tag li:hover a`、`.categories.card:hover …`），无需改动。
- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
