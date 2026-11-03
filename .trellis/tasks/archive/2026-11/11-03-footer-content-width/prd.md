# 页脚改为与正文列同宽

## Goal

页脚左端对齐正文左缘、右端止于正文右缘（不是侧栏右缘）。上一版把页脚拉到容器全宽，包含了右侧栏那一列，过头了。

## Requirements

- R1 只改 `vendor/patches/patch.css` 中上一版新增的页脚对齐块。
- R2 `#tail-wrapper`：`width: calc(100% - 296px)`（296 = 右栏 240 + 间隔 56），`flex: 0 0 auto`，左右 padding 0。
- R3 `#tail-wrapper > footer`：左右 padding 28px。
- R4 页脚仍左右分布。

## Acceptance Criteria

- [ ] 1920：页脚 [356, 1568]，左文字 384，右文字止于 1540。
- [ ] 1264：页脚 [316, 912]，宽度与正文一致。
- [ ] 1200 以下不变。
- [ ] `fmt` / `clippy` / `test` / `build` 全绿。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +4 / −3（只调整上一版的页脚宽度）。

```css
#tail-wrapper {
  flex: 0 0 auto !important;
  width: calc(100% - 296px) !important;   /* 296 = 右栏 240 + 间隔 56 */
  max-width: calc(100% - 296px) !important;
}
```

| 视口 | 正文列 | 页脚 | 左文字左缘 | 右文字右缘 |
| --- | --- | --- | --- | --- |
| 1920 | [356, 1568] | **[356, 1568]** | 384 | 1540 |
| 1264 | [316, 912] | **[316, 912]** | 344 | 884 |

页脚现在与正文列严格同宽：左端对齐正文左缘，右端止于正文右缘（内缩 28px 与正文文字对齐）。

`fmt` / `clippy` / `test` 98 全绿。
