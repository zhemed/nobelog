# 用猫咪线稿做博客图标（明暗自适应）

## Goal

把用户新指定的黑线稿趴猫做成博客图标：裁到笔画外框（原图白底占比极大，直接缩到 32px 会看不见），
生成明暗自适应 favicon（SVG）+ PNG 兜底 + apple-touch + og:image，接入模板并在本机上线。

## 输入

- `tools/assets/cat-icon-source.jpg`（1080x1080，白底黑线稿，sha256 855c3d3a…）。
- 上一版图标已完全回退（commit 5cbad23），当前站点无图标。

## 交付

1. `tools/make-icons.py`（重新入库，可复跑，输入指向 `tools/assets/cat-icon-source.jpg`）：
   - 自动裁到暗像素 bbox + 约 6% 内边距，再居中放进方形画布（保持 1:1，猫占满宽度）。
2. `content/images/` 图标（构建后进 `images/`）：
   - `cat-icon.svg`：内嵌透明 PNG（base64）+ `@media (prefers-color-scheme: dark)` 反色，使暗色标签页里黑线变白线。
   - `cat-icon-32.png`：透明底黑线（PNG 兜底 favicon）。
   - `cat-icon-192.png`：白底（PWA / 高分屏兜底）。
   - `cat-icon-180.png`：白底 apple-touch-icon。
   - `cat-icon-512.png`：白底 og:image。
3. `rust/src/templates.rs` head：
   ```html
   <link rel="icon" type="image/svg+xml" href="/images/cat-icon.svg">
   <link rel="icon" type="image/png" sizes="32x32" href="/images/cat-icon-32.png">
   <link rel="icon" type="image/png" sizes="192x192" href="/images/cat-icon-192.png">
   <link rel="apple-touch-icon" sizes="180x180" href="/images/cat-icon-180.png">
   <meta property="og:image" content="{base_url}/images/cat-icon-512.png">
   ```
   base_url 走现有 `escape_html` 转义；测试断言同步。
4. 预览（`.run/review/cat-icon/`）：16/32/48/180/512 在明/暗底上的效果 + SVG 明暗自适应对比 + 浏览器标签页模拟；
   截图 `preview.png`，另出首页明/暗截图。
5. 本机上线：重建 release + 重启 `nobelog-local` 一次（用户已把本机调试与部署交给 Lead），
   验证图标端点 200、head 链接就位、`/images/cat-icon-source.jpg` 不进产物（源图在 tools/，天然不进）、回归通过。

## 约束

- 不访问远程；不改站点其它视觉；不引入依赖（Pillow 已装）。
- 源图放 `tools/assets/`，不进 `content/images`（避免进发布包）。

## Acceptance

- [ ] 32px 下猫形可辨认（相对原图显著放大，留白大幅减少）。
- [ ] SVG favicon 在暗色下自动反色（预览可见）。
- [ ] 模板 head 五条链接就位；图标端点 200；静态构建产物 `images/` 只含 5 个 cat-icon 文件 + 原有图。
- [ ] 门禁 fmt / clippy / test 全绿；本机重启后回归通过。
- [ ] 提交、journal、归档。
