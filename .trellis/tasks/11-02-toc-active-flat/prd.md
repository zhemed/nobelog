# 目录激活项取消特殊处理

## Goal

目录当前激活项仍与其它条目不同（墨色文字 + 700 字重 + 左侧墨色指示条）。用户认为这样与其它条目不一致、观感怪；取消激活项的特殊视觉，使其与普通条目完全相同，hover 反馈保留。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 激活项：文字 `var(--ink-muted)`、`font-weight: 400`、`::before` 指示条 `display: none`。
- R3 hover 对所有条目（含激活项）一致：浅灰底 + 墨色 + 700。
- R4 非激活项默认保持次要灰 400。
- R5 不改 DOM，不新增类名。
- R6 验证：默认时激活项与其它条目计算样式完全相同；hover 激活项仍有底色/墨色/700。

## Acceptance Criteria

- [x] 激活项默认：`color rgb(107,114,128)`、`font-weight 400`、指示条不显示。
- [x] 激活项 hover：背景 `#eef1f5`、文字 `rgb(17,20,24)`、字重 700。
- [x] 非激活项行为不变。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +8 / −12 行。

```css
/* 激活项不做特殊处理 */
#toc-wrapper .toc-link.is-active-link { color: var(--ink-muted) !important; font-weight: 400 !important; }
#toc-wrapper .toc-link.is-active-link::before { display: none !important; }
/* hover 对所有条目生效（放在后面，等特异度时后写者胜） */
#toc-wrapper .toc-link:hover { background-color: var(--brand-hover); color: var(--ink) !important; font-weight: 700 !important; }
```

实测：

| 状态 | 颜色 | 字重 | 指示条 |
| --- | --- | --- | --- |
| 激活项默认 | `rgb(107,114,128)` | 400 | 隐藏 |
| 普通项默认 | `rgb(107,114,128)` | 400 | — |
| 激活项 hover | `rgb(17,20,24)` | 700 | 隐藏，带 `#eef1f5` 底 |

激活项与普通项的计算样式已完全一致（`same = true`）。

- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
