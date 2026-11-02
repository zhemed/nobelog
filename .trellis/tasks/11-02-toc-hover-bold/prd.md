# 目录 hover 补加粗加深，激活项改墨色

## Goal

1. 目录条目 hover 时补上「加深 + 加粗」，与列表行手感一致（当前只有浅灰底）。
2. 当前激活项是强调蓝 + 蓝色细条，与全站已统一的墨色语言不一致；改为墨色文字 + 加粗 + 墨色指示条。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 hover：背景 `var(--brand-hover)`、文字 `var(--ink)`、`font-weight: 700`。
- R3 激活项：文字 `var(--ink)`、`font-weight: 700`、`::before` 指示条背景 `var(--ink)`；需带 `!important` 压过 chirpy 的 `--toc-highlight` 规则。
- R4 非激活项默认保持次要灰、400 字重。
- R5 暗色沿用令牌。
- R6 验证：hover 与激活项均为墨色 700；指示条为墨色；无换行溢出。

## Acceptance Criteria

- [x] hover 条目：背景 `#eef1f5`、文字 `rgb(17,20,24)`、字重 700。
- [x] 激活项：文字 `rgb(17,20,24)`、字重 700，不再是蓝色。
- [x] 激活项左侧指示条为墨色。
- [x] 非激活项默认仍是 `rgb(107,114,128)` / 400。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +9 / −4 行。

- hover：`background-color: var(--brand-hover)` + `font-weight: 700 !important`，文字 `var(--ink)`。
- 激活项：`color: var(--ink) !important`、`font-weight: 700 !important`，`::before` 指示条 `var(--ink) !important`。
- 之所以要 `!important`：面板区存在 `#panel-wrapper .toc-link { color: var(--ink-muted) !important; font-weight: 400 !important }`，
  未加权的规则会被它压住（第一版实测激活项字重仍是 400）。

实测：

| 状态 | 背景 | 文字 | 字重 | 指示条 |
| --- | --- | --- | --- | --- |
| 非激活默认 | 透明 | `rgb(107,114,128)` | 400 | — |
| hover | `rgb(238,241,245)` | `rgb(17,20,24)` | **700** | — |
| 激活项 | — | `rgb(17,20,24)`（原为强调蓝） | **700** | 墨色（原为蓝） |

- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
