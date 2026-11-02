# 面板列表补齐 hover 浅灰底

## Goal

上一任务只把面板链接改成加粗变深，漏了归档行同款的整行浅灰底；本次补上，让「最近更新」的 hover 观感与归档行完全一致。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 作用域：`#panel-wrapper .post-content li` 及其 `:hover`；不影响面板标题与标签胶囊。
- R3 hover 背景 `var(--brand-hover)`，圆角 `.5rem`，`transition: background-color .3s ease-in-out`。
- R4 不改变文字位置：如需内边距，用等量负外边距抵消。
- R5 亮/暗均使用既有令牌。
- R6 验证：面板列表行 hover 背景为 `rgb(238,241,245)`（暗色 `#1d232a`）；文字基线位置与改前一致；其它页面不变。

## Acceptance Criteria

- [x] 面板列表行 hover 出现浅灰圆角底，颜色 `#eef1f5`。
- [x] 文字水平位置与改前一致。
- [x] 面板标题、标签胶囊不受影响。
- [x] 暗色 hover 为 `#1d232a`。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +13 行。

```css
#panel-wrapper .post-content li {
  padding: 0 .4rem; margin: 0 -.4rem;   /* 内边距与负外边距抵消，文字位置不变 */
  border-radius: .5rem; transition: background-color .3s ease-in-out;
}
#panel-wrapper .post-content li:hover { background-color: var(--brand-hover); }
```

实测：

| 检查 | 结果 |
| --- | --- |
| 列表行 hover 背景 | `rgb(238,241,245)` = #eef1f5 |
| 圆角 | 8px（.5rem） |
| 文字水平位置 | 锚点左缘 974px，与加 padding 前一致 |
| 行宽 | 232px（padding 6.4px × 2，负外边距抵消） |
| 暗色 hover | `rgb(29,35,42)` = #1d232a |
| 标签胶囊 | 未受影响 |

- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
