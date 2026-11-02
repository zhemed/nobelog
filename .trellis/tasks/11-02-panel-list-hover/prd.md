> 一句话：把右侧面板的列表链接改成和归档行同一套 hover 手感。

## Goal

右侧「最近更新」链接与归档行不统一：面板是 400 字重、次要灰、无 hover 变化。改为与归档行一套：常态 500 + 次墨色、hover 700 + 墨色，并确保没有静态下划线；作用域仅 `#panel-wrapper` 列表。

## Requirements

- R1 改动范围：只改 `vendor/patches/patch.css`；不动模板、内容、JS、上游 CSS。
- R2 作用域：仅 `#panel-wrapper .post-content a`；面板里的标签胶囊 `.post-tag` 不受影响。
- R3 常态：`font-weight: 500 !important`、`color: var(--ink-body) !important`、无 `text-decoration`、无边框下划线、`transition: color .2s`。
- R4 hover：`font-weight: 700 !important`、`color: var(--ink) !important`。
- R5 不得影响文章正文、归档、分类、标签页与页脚链接。
- R6 暗色沿用既有令牌（`--ink-body #c7ccd3`、`--ink #e6e9ec`）。
- R7 验证：面板链接常态/悬停计算样式命中预期；其它页面抽样无变化；静态构建 HTML 不变。

## Acceptance Criteria

- [x] 面板链接常态 `font-weight 500` / `rgb(51,55,61)`，`text-decoration: none`、`border-bottom: 0`。
- [x] 面板链接 hover `font-weight 700` / `rgb(17,20,24)`。
- [x] 面板标签胶囊 `.post-tag` 样式不变。
- [x] 正文/归档/分类/页脚链接抽样与改前一致。
- [x] 暗色下常态 `#c7ccd3`、悬停 `#e6e9ec`。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 非目标

- 不改面板标题、标签胶囊、面板宽度。
- 不改归档/分类/标签页已有规则。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +14 / −1 行。把原来合并的
`#panel-wrapper .post-content a, #panel-wrapper .post-tag { 400 / ink-muted }`
拆开：胶囊保持原样，列表链接单独一套。

```css
#panel-wrapper .post-content a       { font-weight: 500 !important; color: var(--ink-body) !important;
                                       text-decoration: none; border-bottom: none; transition: color .2s; }
#panel-wrapper .post-content a:hover { font-weight: 700 !important; color: var(--ink) !important; }
```

实测：

| 场景 | 字重 | 颜色 | 下划线 |
| --- | --- | --- | --- |
| 面板链接常态 | 500 | `rgb(51,55,61)` | 无（`border-bottom: 0`） |
| 面板链接 hover | 700 | `rgb(17,20,24)` | 无 |
| 面板标签胶囊 | 400 | `rgb(107,114,128)` | 不变 |
| 归档行链接（回归） | 500 | `rgb(51,55,61)` | 不变 |
| 暗色面板链接常态 / hover | 500 / 700 | `#c7ccd3` / `#e6e9ec` | 无 |

- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
- 备注：改动前的面板链接实测本来就无下划线（`border-bottom: 0`），用户截图中的下划线可能来自旧 CSS 缓存；本次按要求补齐了常态字重与 hover 加粗。
