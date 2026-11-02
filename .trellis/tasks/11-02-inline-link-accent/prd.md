# 正文行内链接恢复强调蓝

## Goal

把文章正文里的行内链接恢复为强调蓝（`--accent #3b5b7a`），让它与正文可区分；导航类链接（归档标题、目录、面包屑、面板列表）继续保持墨色，不受影响。

## 背景

`11-02-link-color-ink` 把全站链接令牌改为 `var(--ink)`，正文行内链接因此与正文同色，读者无法识别。导航场景同色是刻意设计，正文场景则需要可辨识。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 `.markdown-body a` 颜色改为 `var(--accent)`（替换当前的 `var(--link-color)`）。
- R3 不改变 hover 行为（无下划线）。
- R4 导航类链接不变：`#archives`、`#page-tag`、`#panel-wrapper`、`#breadcrumb`、`#toc-wrapper`。
- R5 暗色下用暗色 `--accent #8fb3d9`。
- R6 验证：正文行内链接计算色为 `rgb(59,91,122)`（暗色 `rgb(143,179,217)`）；归档/目录/面包屑仍为墨色。

## Acceptance Criteria

- [ ] 文章正文行内链接为 `rgb(59,91,122)`。
- [ ] 归档标题、目录、面包屑、面板仍为 `rgb(17,20,24)`。
- [ ] 暗色下正文链接为 `rgb(143,179,217)`。
- [ ] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [ ] `fmt` / `clippy` / `test` / `build --locked` 全绿。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +3 / −3 行（`.markdown-body a` 的 `color` 由 `var(--link-color)` 改为 `var(--accent)`）。

实测：

| 位置 | 亮色 | 暗色 |
| --- | --- | --- |
| 正文行内链接（「普通链接」「吱吱」） | `rgb(59,91,122)` | `rgb(143,179,217)` |
| 面包屑链接 | `rgb(17,20,24)` 不变 | `rgb(230,233,236)` 不变 |
| 目录条目 | 次要灰（默认）/ 墨色（hover）不变 | 不变 |
| 归档标题 | 墨色不变 | 不变 |

- 装饰保持 `text-decoration: none`，未引入下划线。
- 静态构建：归一化后差异仍只有 2 个文章页（上一任务的模板改动）与 `feed.xml` 时间，本次 CSS 改动无结构影响。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
