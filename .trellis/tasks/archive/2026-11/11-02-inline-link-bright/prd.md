# 正文链接改用更醒目的蓝色

## Goal

正文行内链接从偏灰的 `--accent #3b5b7a` 换成对比更强的链接蓝：亮色 `#0969da`、暗色 `#58a6ff`，确保一眼可辨。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 新增令牌 `--link-body`：亮色 `#0969da`、暗色 `#58a6ff`（放在两套纸墨令牌块内）。
- R3 `.markdown-body a { color: var(--link-body) }`。
- R4 不改导航类链接（归档、目录、面包屑、面板）与 `--accent` 的其它用途（图钉、复选框、面包屑 hover）。
- R5 不加下划线。
- R6 验证：正文链接亮色 `rgb(9,105,218)`、暗色 `rgb(88,166,255)`；导航链接不变。

## Acceptance Criteria

- [ ] 正文行内链接亮色 `rgb(9,105,218)`、暗色 `rgb(88,166,255)`。
- [ ] 面包屑/目录/归档/面板链接颜色不变。
- [ ] `--accent` 的其它用途（图钉、复选框、面包屑 hover）不受影响。
- [ ] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [ ] `fmt` / `clippy` / `test` / `build --locked` 全绿。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +3 / −1 行。

- 新增令牌：亮色块 `--link-body: #0969da`、暗色块 `--link-body: #58a6ff`。
- `.markdown-body a` 改用 `var(--link-body)`。

实测：

| 位置 | 亮色 | 暗色 |
| --- | --- | --- |
| 正文行内链接 | `rgb(9,105,218)` | `rgb(88,166,255)` |
| 面包屑链接 | `rgb(17,20,24)` 不变 | `rgb(230,233,236)` 不变 |
| 目录条目 | 不变 | 不变 |

- `--accent` 未改动，图钉、复选框、面包屑 hover 等用途不受影响。
- 静态构建：仅 CSS 哈希变化，无新增结构性差异。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
