# 超宽屏内容贴左、右栏贴右

## Goal

超宽屏（≥1650px，实测 2555px）下当前「三段留白均分」把内容挤在中间，左右各留约 318px。改为：内容紧贴侧栏（左距 56px）、主栏保持 1060px 上限、右栏贴屏幕右缘（右距 56px），多余空间留在主栏与右栏之间。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 `@media (min-width: 1650px)`：
  - `#main-wrapper > .container`：`width: calc(100% - 112px)`、左右 `margin: 56px`；
  - `#main-wrapper .row.flex-grow-1`：`justify-content: space-between`；
  - `#panel-wrapper`：`margin-left: 56px`（保证最小间隔）。
- R3 主栏仍受 `max-width: 1060px` 约束。
- R4 1200–1649px 现有行为不变。
- R5 改后：主栏左缘距侧栏 56px；右栏右缘距屏幕右缘 56px。

## Acceptance Criteria

- [ ] 2555px 视口：容器左缘 356（侧栏 300 + 56）；右栏右缘 2499（2555−56）。
- [ ] 1920px 视口：主栏 1060、右栏贴右缘 56。
- [ ] 1650px 视口：主栏与右栏之间仍至少 56px。
- [ ] 主栏宽度不超过 1060px。
- [ ] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [ ] `fmt` / `clippy` / `test` / `build --locked` 全绿。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +19 行。

```css
@media all and (min-width: 1650px) {
  #main-wrapper > .container {
    --layout-gap: 56px;
    width: calc(100% - 112px) !important;
    margin-left: 56px !important;
    margin-right: 56px !important;
  }
  #main-wrapper .row.flex-grow-1 { justify-content: space-between; }
  #panel-wrapper { margin-left: 56px !important; }
}
```

实测（Playwright 调视口）：

| 视口 | 容器 [左, 宽] | 主栏 [左, 宽] | 右栏 [左, 右] | 主/右间隔 | 右缘留白 |
| --- | --- | --- | --- | --- | --- |
| 2555 | 356, 2143 | 356, 1060 | 2259, 2499 | 843 | 56 |
| 1920 | 356, 1508 | 356, 1060 | 1624, 1864 | 208 | 56 |
| 1650 | 356, 1238 | 356, 942 | 1354, 1594 | 56 | 56 |
| 1264（回归） | 316, 892 | 316, 596 | 968, 1208 | 56 | — |

- 2555 下：容器左缘 356 = 侧栏 300 + 56；右栏右缘 2499 = 2555 − 56。
- 主栏始终不超过 1060；多余空间成为主栏与右栏之间的留白（`space-between`）。
- 所有档位 `scrollWidth === clientWidth`，无横向溢出。
- 静态构建：仅 CSS 哈希变化。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
