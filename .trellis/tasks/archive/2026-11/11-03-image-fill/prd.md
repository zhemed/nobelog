# 正文图片撑满内容列

## Goal

居中版观感不自然。改为图片宽度撑满内容列（含外壳 `a.img-link`），用户看过 1.14×/1.5× 放大效果后确认采用。

## 改动

```css
.markdown-body img, .post-content img { display: block; width: 100%; height: auto; }
.markdown-body a.img-link, .post-content a.img-link { display: block; width: 100%; }
```

## 验证结果（2026-11-03）

| 视口 | 内容列 | 图片渲染宽 | 原图 | 倍数 |
| --- | --- | --- | --- | --- |
| 1920 | 1094 | 1094 | 960 | 1.14× |
| 2560 | 1450 | 1442 | 960 | 1.5× |
| 390 | 334 | 334 | 960 | 0.35×（缩小） |

无横向溢出；`fmt` / `clippy` / `test` 98 全绿。
