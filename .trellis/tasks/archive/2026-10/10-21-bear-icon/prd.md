# 换用熊脸图做博客图标（可切换源图）

## Goal

用用户新给的奶白熊脸图试用站点图标；把图标文件名改成**通用名**（`site-icon-*`），生成脚本支持传入源图，
以后在猫/熊/其它图之间切换只需重跑脚本，不再改模板。

## 输入

- `tools/assets/bear-icon-source.jpg`（2048x2048，白底、彩色描边插画，sha256 da7fbd01…）。
- 已有 `tools/assets/cat-icon-source.jpg` 与 `tools/make-icons.py`（上一版猫咪图标脚本）。

## 交付

1. `tools/make-icons.py` 改造：
   - 接受源图路径参数（默认 `tools/assets/bear-icon-source.jpg`），输出前缀 `site-icon`；
   - 阈值/bbox 裁剪 + 约 5% 内边距 + 居中方形画布；
   - 输出 `content/images/site-icon-32.png`、`-192.png`、`-180.png`、`-512.png`（**白底**，彩色插画保持原色，
     不做反色——反色会让奶白变深蓝、粉变绿，不适合彩色图）；
   - 另出 `content/images/site-icon-transparent-512.png`（白底转透明，备用/预览对比）。
2. 删除上一版 `content/images/cat-icon*`（设计保留在 `tools/assets/`，可一键切回）。
3. `rust/src/templates.rs` head 改指通用名（32 / 192 / apple-touch 180 / og:image 512；本版无 SVG，彩色图不适合明暗反色）；
   测试断言同步。
4. 预览（`.run/review/bear-icon/`）：16/32/48/180/512 在明/暗底效果 + 透明版对比 + 标签页模拟；
   截图 `preview.png`；另出首页明/暗截图。
5. 本机上线（Lead 已获本机部署授权）：重建 release + 重启一次；验证端点 200、head 就位、回归通过。

## 约束

- 源图放 `tools/assets/`，不进 `content/images`；不访问远程；不改其它视觉；不引入依赖。

## Acceptance

- [ ] 32px 下熊脸可辨认（相对原图留白大幅减少）。
- [ ] 模板 head 指向 `site-icon-*`；切换源图只需重跑脚本（模板不动）。
- [ ] 门禁 fmt / clippy / test 全绿；静态产物 `images/` 只含新图标与原有图。
- [ ] 本机上线后端点 200、head 正确、9 路由回归通过。
- [ ] 提交、journal、归档。
