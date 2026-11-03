# 正文图片居中

## Goal

正文图片按自然宽度渲染、默认靠左，内容列变宽后右侧空白失衡。让图片在内容列里居中，不改尺寸。

## 改动

```css
.markdown-body img, .post-content img { display: block; margin-inline: auto; }
.markdown-body a.img-link, .post-content a.img-link { display: block; width: fit-content; margin-inline: auto; }
```

图片外层是 `a.img-link`（inline-flex，与图片同宽），只居中 `img` 不生效，因此连 `a` 一起居中。

## 验证结果（2026-11-03）

| 视口 | 图片宽 | 左空隙 | 右空隙 |
| --- | --- | --- | --- |
| 1920 | 960 | 67 | 67 |
| 390 | 334（满宽） | 0 | 0 |

无横向溢出；`fmt` / `clippy` / `test` 98 全绿。
