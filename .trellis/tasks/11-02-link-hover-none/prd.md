# 移除链接 hover 下划线反馈

## Goal

去掉上一任务为链接新增的 hover 下划线，让链接在默认与悬停两种状态下保持同一颜色与装饰；可点击感交给已有的块级反馈（归档行高亮、分类卡片高亮、侧栏导航高亮）与指针形状。

## 背景

`11-02-link-color-ink` 把链接色改为墨色后，正文与链接同色，于是补了 `text-decoration: underline` 作为 hover 提示。用户看过后认为不需要：下划线在归档列表与右侧面板里显得多余，且与整行/整卡片的 hover 高亮重复。

## Requirements

- R1 改动范围：只改 `vendor/patches/patch.css`；不动模板、内容、JS、上游 CSS。
- R2 删除 `text-decoration: underline` 与 `text-underline-offset` 的 hover 规则（正文、归档、标签详情、面板、页脚）。
- R3 保留 `.markdown-body a { color: var(--link-color); }`：否则正文链接会回落到 `github-markdown` 写死的 GitHub 蓝 `#0969da`。
- R4 保留块级反馈：归档行 hover 高亮、分类卡片 hover 高亮、侧栏导航 hover 高亮。
- R5 不新增任何 hover 颜色变化；链接 hover 前后 `color` 应一致。
- R6 验证：hover 归档链接 / 面板链接 / 正文链接，`text-decoration` 为 `none`、`color` 不变；正文链接仍为 `rgb(17,20,24)`；静态构建 HTML 不变。

## Acceptance Criteria

- [x] hover 归档标题链接：`text-decoration: none`，颜色仍 `rgb(17,20,24)`。
- [x] hover 右侧面板链接：无下划线、颜色不变。
- [x] 正文内链接默认仍为 `rgb(17,20,24)`（未回落到 GitHub 蓝）。
- [x] 归档行、分类卡片、侧栏导航的整块 hover 高亮保留。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 非目标

- 不改链接颜色、字号、字重。
- 不处理标签胶囊与分类卡片自带的 hover 颜色。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +5 / −13 行。

1. 删除上一任务新增的链接 hover 下划线规则（正文、归档、标签详情、面板、页脚五处选择器）。
2. 保留 `.markdown-body a { color: var(--link-color); }`，避免正文链接回落到 `github-markdown` 的 GitHub 蓝。
3. 追加 `.markdown-body a:hover { text-decoration: none; }`：`github-markdown.css` 自带 `.markdown-body a:hover { text-decoration: underline; }`，需同权覆盖。

实测（1264px 视口）：

| 场景 | text-decoration | color |
| --- | --- | --- |
| 归档链接 hover | none | `rgb(17,20,24)` 不变 |
| 面板链接 hover | none | `rgb(107,114,128)` 不变 |
| 正文链接 hover | none | `rgb(17,20,24)` 不变 |

- 块级反馈保留：归档行 hover 背景 `#eef1f5`、分类卡片 header 高亮、侧栏导航高亮均未受影响。
- 正文链接仍有主题原有的 1px 底部细线（`#dee2e6`，非 hover 触发），这是默认链接样式；若要求「连静态下划线也去掉」，正文链接将完全无法与普通文字区分，需单独确认。
- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化，HTML 无改动。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
