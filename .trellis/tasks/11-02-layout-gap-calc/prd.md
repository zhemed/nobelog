# 版式留白改为显式 calc 并统一宽屏间距下限

## Goal

把 `#main-wrapper > .container` 的宽度从「`width: 100%` + 左右外边距、靠 flex 收缩吸收」改为显式 `calc(100% - 2 × gap)`，让版式不再依赖收缩行为；同时把 ≥1650px 断点的间距下限统一为 56px，修掉 1650–1768px 区间间距塌陷到 16.7px 的不一致。

## 背景

`vendor/patches/patch.css` 经过三轮追加，同一组选择器被四段规则先后覆盖：

- `1200–1649.98px`：`width:100%`，`margin: max(56px, calc((100vw - 1560px)/3))` —— 该区间恒为 56px。
- `≥1650px`：`width:100%`，`margin: calc((100vw - 1600px)/3)` —— 无下限，1650px 时只有 16.7px，1768px 才回到 56px。

容器宽度超出父容器的部分由 flex 收缩吸收；上一任务 `11-02-fix-post-overflow` 已用 `min-width:0` 修复收缩失效导致的溢出，但「宽度靠收缩兜底」的写法仍在。

## Requirements

- R1 改动范围：只改 `vendor/patches/patch.css`；不动模板、内容、客户端 JS、上游 CSS。
- R2 统一留白：`gap = max(56px, calc((100vw - 1600px) / 3))`，1200px 起全部断点共用这一个表达式。
- R3 显式宽度：容器 `width: calc(100% - gap * 2)`，不再出现 `width: 100%` 搭配外边距的写法。
- R4 右栏对齐：`#panel-wrapper` 的 `margin-left` 使用同一 `gap` 表达式。
- R5 保留上一任务的 `min-width: 0` 作为防御，不删除。
- R6 值不变区间：1200–1649.98px 计算结果保持 56px 与现有容器宽度；≥1768px 保持现有 `(100vw-1600)/3`。
- R7 行为修正区间：1650–1767px 的间距由 16.7–55.9px 提升为 56px（本次唯一有意的视觉变化）。
- R8 不改 1200px 以下断点。
- R9 验证：三个视口（1264、1440、1920）实测无横向溢出、右栏可见、间距符合表达式；`cargo build --locked` 与静态构建 HTML 产物不变化。

## Acceptance Criteria

- [x] 1264px 视口：容器 877px、间距 56px、无横向溢出，与改动前一致。
- [x] 1440px 视口：间距 56px，容器 = 100% − 112px（实测 1068px）。
- [x] 1650px 视口：间距 56px（改动前为 16.7px）。
- [x] 1920px 视口：间距 106.67px，容器 = 100% − 213.33px（实测 1407px）。
- [x] 三个视口下 `#panel-wrapper` 右缘都不超出视口，页面 `scrollWidth === clientWidth`。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [x] `fmt` / `clippy` / `test` / `build --locked` 全绿；静态构建 HTML 与改动前一致。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 非目标

- 不改主栏 1060 上限、右栏 240 宽度、侧栏 260 宽度。
- 不处理 1200px 以下断点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` −63/+10 行。删除四段互相覆盖的旧留白规则，合并为一段：

```css
@media all and (min-width: 1200px) {
  #main-wrapper > .container {
    --layout-gap: max(56px, calc((100vw - 1600px) / 3));
    width: calc(100% - var(--layout-gap) * 2) !important;
    margin-left/right: var(--layout-gap) !important;
  }
  #panel-wrapper { margin-left: var(--layout-gap, …) !important; }
}
```

浏览器实测（Playwright 调视口，文章页 `/posts/typeset-test/`）：

| 视口 | 间距 | 容器宽 | 横向溢出 | 右栏可见 |
| --- | --- | --- | --- | --- |
| 1200 | 56 | 828 | 0 | 是 |
| 1264（ego 实际窗口） | 56 | 877 | 0 | 是 |
| 1440 | 56 | 1068 | 0 | 是 |
| 1650 | **56**（改前 16.7） | 1238 | 0 | 是 |
| 1800 | 66.67 | 1367 | 0 | 是 |
| 1920 | 106.67 | 1407 | 0 | 是 |

- 三处留白在 1650 实测均为 56（左间隔 56 / 面板 margin-left 56 / 面板右缘到屏幕 56）。
- 首页在 1920 下容器 1407、右缘留白 107，无溢出。
- 静态构建：15 页 153219 字节，与改动前归一化资源哈希逐文件比对，仅 `feed.xml` 的 `lastBuildDate` 变化。
- 质量：`cargo fmt --check` 通过、`clippy --all-targets` 无警告、`test` 98 passed、`build --release --locked` 通过。
