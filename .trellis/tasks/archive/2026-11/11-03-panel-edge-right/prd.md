# 右侧栏贴屏幕右缘（内容位置不变）

## Goal

左侧栏贴窗口左缘，右侧栏在超宽屏下却距右缘 318px。把右侧栏推到距屏幕右缘 56px，同时**保持内容块左侧留白与位置不变**。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 `@media (min-width: 1650px)`：
  - `#main-wrapper > .container`：`width: calc(100% - var(--layout-gap) - 56px)`、`margin-left: var(--layout-gap)`（保持与 1200+ 块一致）、`margin-right: 56px`；
  - `#main-wrapper .row.flex-grow-1 { justify-content: space-between; }`；
  - `#panel-wrapper { margin-left: 56px; }`（最小间隔）。
- R3 主栏仍受 1060 上限；左侧留白仍为 `max(56px, (100vw-1600)/3)`。
- R4 1200–1649px 行为不变。
- R5 验证：2555px 下主栏左缘仍在 618（300 + 318），右栏右缘 2499（2555−56）。

## Acceptance Criteria

- [ ] 2555px：容器左缘 618（不变）、右缘 2499、主栏 1060、右栏 [2259, 2499]。
- [ ] 1920px：容器左缘 407、右缘 1864、主栏 1060。
- [ ] 1650px：左右各 56，与现状一致。
- [ ] 1264px 行为不变。
- [ ] 无横向溢出；`git diff --stat` 仅 `vendor/patches/patch.css`。
- [ ] `fmt` / `clippy` / `test` / `build --locked` 全绿。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +19 行（新增 `@media (min-width: 1650px)` 块）。

```css
#main-wrapper > .container {
  width: calc(100% - var(--layout-gap) - 56px) !important;
  margin-left: var(--layout-gap) !important;   /* 左留白不变 */
  margin-right: 56px !important;               /* 右边界收到屏宽−56 */
}
#main-wrapper .row.flex-grow-1 { justify-content: space-between; }
#panel-wrapper { margin-left: 56px !important; }
```

实测：

| 视口 | 容器 [左, 宽, 右] | 主栏 [左, 宽] | 右栏 [左, 右] | 左留白 | 右缘留白 |
| --- | --- | --- | --- | --- | --- |
| 2555 | 618, 1881, 2499 | 618, 1060 | 2259, 2499 | 318（不变） | **56** |
| 1920 | 407, 1457, 1864 | 407, 1060 | 1624, 1864 | 107（不变） | **56** |
| 1650 | 356, 1238, 1594 | 356, 942 | 1354, 1594 | 56 | 56 |
| 1264（回归） | 316, 892 | 316, 596 | 968, 1208 | — | — |

- 内容块左侧留白与主栏位置与改动前完全一致；仅右侧栏从「距右缘 318」变为「距右缘 56」。
- 各档位 `scrollWidth === clientWidth`，无横向溢出。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
