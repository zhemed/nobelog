# 目录激活项加 2px 竖条提示

## Goal

激活项取消特殊化后，读者无法判断当前读到哪一节。在激活项左缘加一条 2px 墨色圆角短竖条作为定位提示；文字保持次要灰 400，不加粗不变色，不与 hover 语言冲突。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 `#toc-wrapper .toc-link.is-active-link::before`：`display: inline-block !important`、`width: 2px`、`height: 1rem`、`left: -1px`、`border-radius: 1px`、`background-color: var(--ink) !important`。
- R3 激活项文字仍为 `var(--ink-muted)` / 400（沿用上一任务）。
- R4 hover 行为不变（浅灰底 + 墨色 + 700）。
- R5 验证：激活项左侧出现墨色 2px 竖条；文字样式不变；hover 不变。

## Acceptance Criteria

- [x] 激活项 `::before` 可见，宽 2px、高 1rem、背景 `rgb(17,20,24)`、圆角 1px。
- [x] 激活项文字仍 `rgb(107,114,128)` / 400。
- [x] hover 仍是 `#eef1f5` 底 + 墨色 + 700。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +11 / −1 行。

```css
#toc-wrapper .toc-link { position: relative; }   /* ::before 以条目为定位参照 */
#toc-wrapper .toc-link.is-active-link::before {
  display: inline-block !important;
  top: 50%; transform: translateY(-50%);
  width: 2px; height: 1rem; left: .25rem; border-radius: 1px;
  background-color: var(--ink) !important;
}
```

第一版未加 `position: relative`，`::before` 以 `#toc-wrapper` 为参照，竖条落到整列左缘、
离文字很远；改为以条目自身为参照后，竖条贴在条目文字左侧并与行垂直居中。

实测：

| 检查 | 结果 |
| --- | --- |
| 激活项竖条 | 2px × 16px，`rgb(17,20,24)`，圆角 1px，垂直居中 |
| 激活项文字 | `rgb(107,114,128)` / 400（与普通项一致） |
| 激活项 hover | 背景 `rgb(238,241,245)`、墨色、700，竖条保留 |
| 普通项 | 无竖条 |

- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
