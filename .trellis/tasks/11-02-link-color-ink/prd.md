# 全站链接默认色统一为墨色

## Goal

把全站链接的默认颜色从 `--accent`（亮色 `#3b5b7a` 蓝灰 / 暗色 `#8fb3d9`）改为 `--ink`（亮色 `#111418` 近黑 / 暗色 `#e6e9ec`），与左侧栏导航文字一致，形成单色阅读风格。

## 背景

链接色来自品牌令牌 `--link-color: var(--accent)`（`vendor/patches/patch.css` 的亮/暗两组纸墨令牌）。归档页的标题、文章内链接、面包屑、面板链接等都取这一个值，实测为 `rgb(59,91,122)`。

用户希望链接文字与侧栏导航同色（`#111418`），全站统一。

## Requirements

- R1 改动范围：只改 `vendor/patches/patch.css` 中的令牌定义；不动模板、内容、JS、上游 CSS。
- R2 亮色：`--link-color: var(--ink)`（`#111418`）。
- R3 暗色：`--link-color: var(--ink)`（`#e6e9ec`）。
- R4 保留 hover 反馈：链接 hover 的下划线/变色规则不变，确保单色下仍能识别可点击。
- R5 强调用途不跟随：`--accent` 本身、`--toc-highlight`、置顶图钉、面包屑 hover、复选框着色继续用强调色。
- R6 不改侧栏、标签胶囊、分类卡片等已单独定色的组件。
- R7 验证：抽查归档、首页、文章页、标签页、分类页、面板的链接计算色均为墨色；hover 有可感知反馈；暗色为 `#e6e9ec`；静态构建 HTML 不变。

## Acceptance Criteria

- [x] 归档页标题链接计算色为 `rgb(17,20,24)`（改前 `rgb(59,91,122)`）。
- [x] 首页、文章正文、标签页、分类页、右侧面板的链接同色。
- [x] 链接 hover 仍有下划线/变色反馈。
- [x] TOC 激活项、置顶图钉、面包屑 hover 仍为强调色。
- [x] 暗色模式链接为 `#e6e9ec`，无硬编码颜色。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 非目标

- 不调整字体、字重、字号。
- 不改 hover 的具体颜色（沿用主题既有的橙色/下划线行为）。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +20 / −2 行。

1. 令牌：亮/暗两组纸墨块的 `--link-color` 由 `var(--accent)` 改为 `var(--ink)`。
2. 正文链接：`github-markdown.css` 写死了 GitHub 蓝（`--fgColor-accent: #0969da`），新增 `.markdown-body a { color: var(--link-color); }` 覆盖。
3. 悬停反馈：链接与正文同色后失去颜色提示，新增 `text-decoration: underline` + `text-underline-offset: .18em`（仅正文/归档/标签/面板/页脚）。

实测：

| 位置 | 改前 | 改后 |
| --- | --- | --- |
| 归档标题链接 | `rgb(59,91,122)` | `rgb(17,20,24)` |
| 正文链接（含 GitHub 蓝那条） | `rgb(9,105,218)` / `rgb(59,91,122)` | `rgb(17,20,24)` |
| 分类卡片链接 | `rgb(59,91,122)` | `rgb(17,20,24)` |
| 标签详情链接 | `rgb(59,91,122)` | `rgb(17,20,24)` |
| 面包屑链接 | — | `rgb(17,20,24)` |
| 暗色链接 | `rgb(143,179,217)` | `rgb(230,233,236)` |
| TOC 激活项 | `rgb(59,91,122)` | 不变（强调色） |
| 结论 | 链接 hover 出现下划线（`text-decoration: underline`） | |

- 面板与页脚链接保持次要灰 `#6b7280`：它们原本就不是蓝色链接，且带 `!important` 的次要文字规则；如需一并改墨色需要另行确认（会削弱侧栏层次）。
- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化，HTML 无改动。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
