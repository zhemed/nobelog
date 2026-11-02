# 标签胶囊 hover 不再变蓝

## Goal

标签胶囊（`.post-tag`）在模板里带 Bootstrap 的 `btn btn-outline-primary` 类，hover 时被 Bootstrap 涂成整块亮蓝 `#0d6efd`，文字还是灰色，观感突兀。把它统一到站点的浅灰 hover 语言。

## 背景

实测面板「热门标签」的胶囊 hover：`background-color: rgb(13,110,253)`、`color: rgb(107,114,128)`、`border-color: rgb(0,123,255)`。文章页标题下的胶囊同样带这两个类，存在同一问题。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 覆盖 `.post-tag:hover`、`.post-tag:focus`、`.post-tag:active`：背景 `var(--brand-hover)`、文字 `var(--ink)`、边框 `var(--btn-border-color)`、`box-shadow: none`，均带 `!important`。
- R3 不改胶囊的默认外观（透明底、次要灰字、浅边框）。
- R4 文章尾部已有的橙色 hover 规则（更高特异度）保持不变。
- R5 亮/暗模式都使用既有令牌。
- R6 验证：面板与文章页胶囊 hover 均不再出现蓝色；默认外观与尺寸不变。

## Acceptance Criteria

- [x] 面板胶囊 hover 背景 `rgb(238,241,245)`、文字 `rgb(17,20,24)`，无蓝色。
- [x] 文章页标题下胶囊 hover 同样不蓝。
- [x] 胶囊默认样式（透明底/灰字/浅边框）不变。
- [x] 暗色 hover 为 `#1d232a`。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 不变。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +14 行。

根因：胶囊元素带 `btn btn-outline-primary` 类，命中主题自带的
`.btn.btn-outline-primary:not(.disabled):hover { border-color:#007bff !important }`
与 Bootstrap 的 `.btn-outline-primary:hover`（背景 `#0d6efd`），而站点的
`.post-tag` 只覆盖了静态颜色，没有覆盖交互态。

覆盖规则（选择器刻意压过 `:not(.disabled)` 那条 4 类特异度）：

```css
#panel-wrapper .post-tag:hover, :focus, :active,
.post-tag.btn.btn-outline-primary:hover, :focus, :active {
  background-color: var(--brand-hover) !important;
  color: var(--ink) !important;
  border-color: var(--btn-border-color) !important;
  box-shadow: none !important;
}
```

实测（面板「排版」胶囊）：

| 状态 | 背景 | 文字 | 边框 |
| --- | --- | --- | --- |
| 默认 | 透明 | `rgb(107,114,128)` | `rgb(233,236,239)` |
| hover（改前） | **`rgb(13,110,253)` 亮蓝** | 灰 | **`rgb(0,123,255)` 蓝** |
| hover（改后） | `rgb(238,241,245)` | `rgb(17,20,24)` | `rgb(233,236,239)` |

- 文章页的标题下标签与尾部标签不带 `btn` 类，无此问题；规则对它们不生效。
- 静态构建：42 个文件归一化后仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
