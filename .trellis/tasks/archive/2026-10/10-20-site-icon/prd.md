# 把粉笔环形涂鸦做成站点图标并预览

## Goal

用用户提供的粉笔环形涂鸦图生成站点图标资源，接入页面 `<head>`，在本机做视觉预览（含多尺寸、明暗背景），
让用户判断效果；本阶段不部署 systemd、不发版。

## 输入

- 原图：`content/images/site-icon-source.png`（由附件复制，1254x1254，米白背景，彩色粉笔环形涂鸦）。
- 站点当前没有 favicon（历史提交曾加过又 revert）。

## 交付

1. 图标资源（放 `content/images/`，构建时复制到产物 `images/`、动态模式直接服务）：
   - `site-icon-32.png`（浏览器 favicon）
   - `site-icon-180.png`（apple-touch-icon，保持不透明米白底）
   - `site-icon-192.png`、`site-icon-512.png`（PWA/分享用）
   - `site-icon-512-transparent.png`（去掉近白底，供预览对比）
   - 源图保留 `site-icon-source.png`
2. `rust/src/templates.rs::render_layout` 的 `<head>` 增加：
   ```html
   <link rel="icon" type="image/png" sizes="32x32" href="/images/site-icon-32.png">
   <link rel="icon" type="image/png" sizes="192x192" href="/images/site-icon-192.png">
   <link rel="apple-touch-icon" sizes="180x180" href="/images/site-icon-180.png">
   <meta property="og:image" content="{base_url}/images/site-icon-512.png">
   ```
   （`base_url` 需转义，与其它插值一致。）
3. 预览：临时实例（高位端口，`--dynamic`）截图
   - 站点首页/文章页截图（确认页面正常）
   - 自制预览页：同一图标在 16/32/48/180/512 尺寸、明/暗背景下的效果，以及不透明 vs 透明版本对比
   - 截图存 `.run/review/icon/`，回复中用 Markdown 内联展示。
4. 校验：`/images/site-icon-32.png` 等返回 200 且 `content-type: image/png`；HTML 里链接存在；
   `cargo fmt/clippy/test` 全绿（模板改动可能影响既有测试，若有需同步）。

## 约束

- 不改站点其它视觉；不重启 systemd 服务（用临时实例预览）；不提交、不推送、不发版（另行确认）。
- 安装图像工具（Pillow/ImageMagick）属本机装包，用户已给长期授权。

## Acceptance

- [ ] 5 个图标文件生成，尺寸正确、视觉无裁切（环形完整）。
- [ ] 模板 head 链接就位，本地临时实例 HTML 可见且资源 200。
- [ ] 明/暗背景与多尺寸预览图产出，可读性判断有依据。
- [ ] 测试门禁全绿；临时实例按 PID 关闭。
