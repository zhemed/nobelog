# 列表行 hover 复刻侧栏加粗变深

## Goal

把左侧栏导航的 hover 手感（常态 500 字重、悬停 700 字重并加深为墨色）复刻到列表类页面：归档行、分类卡片、标签索引胶囊、标签详情列表。

## 背景

侧栏导航的样式是：

```css
#sidebar .nav-link        { font-weight: 500; color: var(--ink); }
#sidebar .nav-link:hover  { font-weight: 700; color: var(--ink); background: var(--brand-hover); }
```

用户感受到的「平常浅、移上去变黑」来自 500 → 700 的字重跃迁叠加浅灰底。

归档行、标签详情行目前只有整行背景高亮，文字不变化；分类卡片只有 header 背景变化；标签胶囊只有整块高亮。

## Requirements

- R1 改动范围：只改 `vendor/patches/patch.css`；不动模板、内容、JS、上游 CSS。
- R2 列表链接常态：`font-weight: 500`、`color: var(--ink-body)`（次墨色，比墨色浅一档）。
- R3 列表链接 hover：`font-weight: 700`、`color: var(--ink)`；过渡只作用于颜色（`transition: color .2s`）。
- R4 作用对象：`#archives ul li a`、`#page-tag li a`、`#page-category li a`、`.categories .card-header a`。
- R5 标签索引胶囊 `#tags .tag`：加粗会引起 flex 换行抖动，改用「颜色加深 + 边框加深」，不加粗。
- R6 标签详情行补齐与归档一致的整行背景高亮（`var(--brand-hover)`、圆角 `.5rem`、`.3s` 过渡）。
- R7 暗色沿用既有令牌，不新增硬编码颜色。
- R8 不改侧栏自身样式，不改链接 hover 的颜色以外属性（不再引入下划线）。

## Acceptance Criteria

- [x] 归档行链接常态 `font-weight 500` / `color rgb(51,55,61)`；hover 为 `700` / `rgb(17,20,24)`。
- [x] 标签详情行与分类详情行同样行为，且 hover 有整行浅灰底。
- [x] 分类卡片 header 链接同样行为。
- [x] 标签索引胶囊 hover 颜色加深但不加粗、不发生换行跳动。
- [x] 暗色下常态/悬停分别使用 `#c7ccd3` 与 `#e6e9ec`。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 非目标

- 不改字号、行高、间距。
- 不给链接加下划线。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +42 行。

- 列表链接常态：`font-weight: 500`、`color: var(--ink-body)`；hover：`700` + `var(--ink)`。
- 作用对象：`#archives ul li a`、`#page-tag li a`、`#page-category li a`、`.categories .card-header a`。
- 标签/分类详情行补齐整行高亮：圆角 `.5rem` + `var(--brand-hover)` + `.3s` 过渡。
- 标签索引胶囊：只加深颜色，不加粗（实测宽度与换行高度不变，无抖动）。

实测（1264px 视口）：

| 场景 | 常态 | hover |
| --- | --- | --- |
| 归档行链接 | 500 / `rgb(51,55,61)` | 700 / `rgb(17,20,24)` |
| 分类卡片链接 | 500 / `rgb(51,55,61)` | 700 / `rgb(17,20,24)` |
| 标签详情行 | 500 / `rgb(51,55,61)`，无底 | 700 / `rgb(17,20,24)` + 行底 `#eef1f5` |
| 标签索引胶囊 | 400 / `rgb(107,114,128)` | 400 / `rgb(17,20,24)`，宽度仍 56px、换行高度 39px 不变 |
| 暗色归档行 | 500 / `#c7ccd3` | 700 / `#e6e9ec` |

- 修正一处继承问题：标签详情链接原本继承 `.post-content` 的橙色 hover（`#d2603a !important` + 橙色下边框），已用同权 `!important` 改回墨色并去掉边框。
- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化，HTML 无改动。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
