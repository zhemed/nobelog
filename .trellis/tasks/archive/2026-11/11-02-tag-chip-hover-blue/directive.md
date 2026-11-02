# 用户当轮原话（逐字记录）

> 你看一下这个哈哈哈哈哈哈

随附截图：右侧「热门标签」里「排版」胶囊整块变成亮蓝色，其余胶囊正常。

## 当轮上下文与边界

- 现象：胶囊 hover 被 Bootstrap 的 `.btn-outline-primary:hover` 涂蓝。
- 授权范围：只调整 `vendor/patches/patch.css` 中 `.post-tag` 的交互态。
- 明确保留：胶囊默认外观、尺寸、布局；文章尾部既有的橙色 hover 规则。
