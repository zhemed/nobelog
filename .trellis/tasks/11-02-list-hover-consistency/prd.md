# 归档与分类列表的悬停高亮与字体统一

## Goal

把归档行与分类卡片上的「静态灰底」改成「随鼠标出现的浅灰高亮」，并统一归档页的字体与字距，让这两页的交互语言与左侧栏导航一致。

## 背景

- 归档页奇数行是 Chirpy 的斑马纹：`#archives ul li:nth-child(odd){background-image:linear-gradient(to left,#fff,#fbfbfb,#fbfbfb,#fbfbfb,#fff)}`，静态铺满整行，在白底上像一块横向污渍。
- 归档日期 `.date.day` 单独指定 `font-family: Lato`，与「10月」的 system 字体不同；`#archives` 还有 `letter-spacing: .48px`。
- 分类卡片头 `background-color` 是 Bootstrap 的 `rgba(33,37,41,.03)`，静态浅灰横条。
- 侧栏导航的交互是 `#sidebar .nav-link:hover{background:#eef1f5}` + `transition .3s`，用户希望把这种「鼠标移上去浮起」的感觉搬到归档与分类。

## Requirements

- R1 改动范围：只改 `vendor/patches/patch.css`；不动模板、内容、JS、上游 CSS。
- R2 归档去静态灰底：`#archives ul li` 与 `li:nth-child(odd)` 的 `background-image` 置 `none`。
- R3 归档悬停：`#archives ul li:hover` 背景 `var(--brand-hover)`，圆角 `.5rem`，`transition: background-color .3s ease-in-out`。
- R4 归档字体：`.date.day` 改 `font-family: inherit`（去掉 Lato）；`#archives` 字距改 `normal`。
- R5 分类去静态灰底：`.categories .card-header` 背景改透明（默认白卡片）。
- R6 分类悬停：卡片 hover 时 `card-header` 背景 `var(--brand-hover)`，过渡与侧栏一致。
- R7 暗色模式全部使用既有令牌（`--brand-hover` 在暗色为 `#1d232a`），不新增硬编码颜色。
- R8 不影响首页、文章页、标签页；不改任何 HTML 结构。

## Acceptance Criteria

- [x] 归档行 `background-image` 为 `none`，hover 时背景为 `#eef1f5` 且圆角生效。
- [x] `.date.day` 计算字体不再是 Lato；`#archives` 字距为 `normal`。
- [x] 分类卡片头默认背景透明；hover 时背景为 `#eef1f5`。
- [x] 暗色模式下两处 hover 均使用 `#1d232a`，无硬编码颜色。
- [x] 其它页面计算样式与改动前一致；静态构建 HTML 不变。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 非目标

- 不改侧栏导航自身样式。
- 不改归档页时间线（竖线、圆点）的结构与位置。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +31 行。

- 归档：`#archives ul li` 与 `:nth-child(odd)` 的 `background-image` 置 `none`（主题暗色规则优先级更高，用 `!important` 确保覆盖）；行圆角 `.5rem`、`transition: background-color .3s`。
- 归档 hover：`#archives ul li:hover` 背景 `var(--brand-hover)`。
- 字体：`#archives` 字距 `normal`；`.date.day` 的 `font-family` 改 `inherit`，不再用 Lato。
- 分类：`.categories .card-header` 背景 `transparent !important`（覆盖 Bootstrap 的 3% 灰与主题暗色的 `#292929`）；`.categories.card:hover .card-header` 背景 `var(--brand-hover) !important`。

实测（1264px 视口）：

| 检查 | 结果 |
| --- | --- |
| 归档行 background-image | `none`（改前为渐变） |
| 归档行 hover 背景 | `rgb(238,241,245)` = #eef1f5 |
| `.date.day` 字体 | `-apple-system`（改前 Lato） |
| `#archives` 字距 | `normal`（改前 0.48px） |
| 分类卡片头默认 | `transparent`（改前 `rgba(33,37,41,.03)`） |
| 分类卡片 hover | header `rgb(238,241,245)` |
| 暗色归档 hover | `rgb(29,35,42)` = #1d232a |
| 暗色分类 hover | `rgb(29,35,42)` |

- 静态构建：42 个文件归一化资源哈希后比对，仅 `feed.xml` 的 `lastBuildDate` 变化，HTML 无改动。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
- 备注：首次在同一脚本里设置 `data-mode` 后立即读取计算样式会拿到旧值，需间隔一次求值再读（暗色验证按此方式复测）。
