# 目录条目补上标签胶囊同款 hover

## Goal

文章页右侧「目录」的 `.toc-link` 目前只有静态颜色，鼠标移上去没有任何反馈。补上与「热门标签」胶囊一致的 hover：浅灰圆角底 + 墨色字。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 作用域：`#toc-wrapper ul .toc-link`，不影响面板其它区块。
- R3 hover：背景 `var(--brand-hover)`、文字 `var(--ink)`、圆角 `.375rem`、`transition: background-color .2s, color .2s`。
- R4 当前激活项 `.is-active-link` 保留强调色（hover 只加底色，不改其文字色）。
- R5 亮/暗均用既有令牌。
- R6 验证：hover 目录条目出现 `#eef1f5` 底、文字变墨色；激活项文字仍为强调色；无布局位移。

## Acceptance Criteria

- [x] hover 普通目录条目：背景 `rgb(238,241,245)`、文字 `rgb(17,20,24)`。
- [x] hover 激活项：背景变化但文字仍为 `rgb(59,91,122)`。
- [x] 不产生横向溢出或文字换行位移。
- [x] 暗色 hover 背景为 `#1d232a`。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +10 行。

```css
#toc-wrapper .toc-link { border-radius: .375rem; transition: background-color .2s, color .2s; }
#toc-wrapper .toc-link:hover { background-color: var(--brand-hover); }
#toc-wrapper .toc-link:not(.is-active-link):hover { color: var(--ink) !important; }
```

关键点：目录位于 `#panel-wrapper` 内，旧规则 `#panel-wrapper .toc-link { color: var(--ink-muted) !important }`
(1,1,0) 会压过普通的 `.toc-link:hover`；因此非激活项的 hover 色用
`#toc-wrapper .toc-link:not(.is-active-link):hover` (1,3,0) 并带 `!important` 才能生效。

实测：

| 状态 | 背景 | 文字 | 圆角 |
| --- | --- | --- | --- |
| 普通项默认 | 透明 | `rgb(107,114,128)` | — |
| hover | `rgb(238,241,245)` | `rgb(17,20,24)` | 6px |
| 激活项 | — | `rgb(59,91,122)` 保持强调色 | — |

- 暗色下背景自动取 `#1d232a`。
- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
