# 标签索引页胶囊对齐侧栏热门标签观感

## Goal

让 `/tags/` 标签索引页的胶囊与侧栏「热门标签」用同一套观感：小字号、紧凑行高、无外阴影、灰色文字、统一圆角与边框，并让计数与中文字形落在同一条基线上。

## 背景

`/tags/` 的胶囊是主题默认 `.tag`：`font-size` 继承 1.03rem（实测 16.48px）、`line-height: 3rem`（48px）、`padding: 6px 8px 7px`、`border-radius: .7em`、`box-shadow: 0 0 3px`、文字为链接蓝 `rgb(59,91,122)`，计数 `<span>` 用 Oswald 字体、11.5px。

侧栏「热门标签」是 `.post-tag`：13.6px 字号、16.8px 行高、4.8/8px 内边距、12.8px 圆角、无阴影、灰色 `rgb(107,114,128)`。

两套样式并存导致标签页偏重、发虚、计数垂直位置飘（计数继承 48px 行高 + Oswald 度量）。

## Requirements

- R1 改动范围：只改 `vendor/patches/patch.css`；不动模板、内容、客户端 JS、上游 CSS。
- R2 作用域：只作用于 `#tags .tag` 与其内部 `span`；不得影响标签详情页的 `#page-tag`、分类页的 `.categories`、以及侧栏/文章页的 `.post-tag`。
- R3 胶囊外观对齐侧栏：`font-size: .85rem`、`line-height: 1.05rem`、`padding: .3rem .5rem`、`border-radius: .8rem`、`border: 1px solid var(--btn-border-color)`、`box-shadow: none`、`color: var(--text-muted-color)`、`font-weight: 400`。
- R4 计数保留：字体继承正文、字号约为标签文字的 .85em、颜色沿用 muted 灰，与标签文字**基线对齐**（容器改 inline-flex + `align-items: baseline`，或等效方案）。
- R5 不新增硬编码颜色，亮色/暗色均使用既有令牌。
- R6 验证：`/tags/` 胶囊实测字号 13.6px、高度约 28px、无阴影；计数与中文文字底线高度差 ≤1px；其它页面 HTML/视觉不变；`cargo build --locked` 通过。

## Acceptance Criteria

- [x] `/tags/ .tag` 计算样式：`font-size 13.6px`、`box-shadow: none`、`border-radius 12.8px`、`height ≈ 28px`。
- [x] 计数字形不再使用 Oswald，与标签文字基线差 ≤1px。
- [x] 胶囊观感与侧栏「热门标签」一致（截图并排比对）。
- [x] 标签详情页 `/tags/<tag>/`、分类页 `/categories/`、文章页标签区 `.post-tag` 不受影响。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 与改动前一致。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 非目标

- 不改侧栏 `.post-tag` 的既有样式。
- 不改标签页的结构与信息（计数保留）。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +25 行，新增 `#tags .tag` 与 `#tags .tag span` 两段规则（放在「文章底部标签」分区之后）。

计算样式对比（同一页面 `/tags/`，1264px 视口）：

| 属性 | 改动前 `.tag` | 改动后 `#tags .tag` | 侧栏 `.post-tag` |
| --- | --- | --- | --- |
| font-size | 16.48px | 13.6px | 13.6px |
| line-height | 48px | 16.8px | 16.8px |
| 高度 | 33px | 28px | 28px |
| 圆角 | 11.54px | 12.8px | 12.8px |
| 阴影 | `0 0 3px` | none | none |
| 文字色 | `rgb(59,91,122)`（链接蓝） | `rgb(107,114,128)` | `rgb(107,114,128)` |
| 边框 | `#dee2e6` | `#e9ecef` | `#e9ecef` |
| 计数 | Oswald 11.5px / 行高 48px | 继承字体 11.56px / 行高 16.8px | — |

- 基线：canvas 字体度量计算，标签文字基线 185.76px、计数基线 185.26px，差 **0.5px**（亚像素级）。
- 主题 `.tag` 的边框在 `main.css` 里带 `!important`，本次用同权 `!important` 覆盖，否则边框色停留在 `#dee2e6`（首次验证时即如此，已修正）。
- 影响面：`/tags/` 上仅 4 个胶囊变化；同页侧栏 `.post-tag` 计算样式与改前完全一致；`/tags/排版/`、`/categories/`、文章页标签区结构不含 `.tag`，不受影响。
- 静态构建：与改动前归一化资源哈希后比对，42 个文件仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
