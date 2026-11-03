# 页脚对齐内容与侧栏两端

## Goal

页脚整条缩在中间：左边没对齐正文左缘，右边没顶到侧栏右缘。原因是 `#tail-wrapper` 用了 bootstrap 的 `col-xl-9`（75% 宽）。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 `@media (min-width: 1200px)`：`#tail-wrapper` 占满容器宽（`flex: 0 0 100%`、`max-width: 100%`、左右 padding 0），`#tail-wrapper > footer` 左右各 28px 内边距。
- R3 页脚仍为左右分布（`justify-content: space-between` 由主题提供）。
- R4 不影响 1200 以下与移动端。

## Acceptance Criteria

- [ ] 1920：页脚左文字左缘 384（= 内容左缘 356 + 28），右文字右缘 1836（= 容器右缘 1864 − 28）。
- [ ] 页脚横向宽度 = 容器宽度（1508）。
- [ ] 1200 以下不变。
- [ ] `fmt` / `clippy` / `test` / `build` 全绿。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +17 行。

```css
@media all and (min-width: 1200px) {
  #tail-wrapper { flex: 0 0 100% !important; width/max-width: 100% !important;
                  padding-left/right: 0 !important; }
  #tail-wrapper > footer { padding-left: 28px; padding-right: 28px; }
}
```

| 视口 | 容器 | 页脚 | 左文字左缘 | 右文字右缘 |
| --- | --- | --- | --- | --- |
| 1920 | [356, 1864] | [356, 1864]（原 1131 宽） | 384（= 356 + 28） | 1836（= 1864 − 28） |
| 1264 | [316, 1208] | [316, 1208] | 344 | 1180 |

- 修改前 `#tail-wrapper` 只有容器宽度的 75%（`col-xl-9`），页脚因此缩在中间。
- 现在页脚与内容+侧栏同宽，左右两端分别对齐正文左缘与右栏右缘内 28px。
- 质量：`fmt` / `clippy` / `test` 98 全绿。
