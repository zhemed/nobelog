# 去掉 favicon 白框（透明版 + 暗色变体 SVG）

## Goal

用户反馈浏览器标签页图标带白色方块。根因是 favicon 用了白底不透明 PNG。改为：
favicon 用**透明底**，并给暗色标签页准备**深色变体**（否则透明版在暗色下黑描边/眼睛会看不见）。

## 方案

1. `tools/make-icons.py` 增加输出：
   - `site-icon-32-transparent.png`、`site-icon-192-transparent.png`（透明底，保留原色）
   - `site-icon-32-dark.png`、`site-icon-192-dark.png`（暗色变体：奶白身体→深灰、黑描边/五官→近白、粉耳保持）
   - 保留 `site-icon-180.png`（白底，apple-touch 需要不透明）与 `site-icon-512.png`（白底，og:image）
   - 原白底 32/192 可删除（不再被引用）
2. `site-icon.svg`：内嵌透明亮色 192 与暗色 192 两张 PNG，用
   `@media (prefers-color-scheme: dark)` 切换显示，作为首选 favicon（支持该特性的浏览器自动明暗自适应）。
3. `rust/src/templates.rs` head：
   ```html
   <link rel="icon" type="image/svg+xml" href="/images/site-icon.svg">
   <link rel="icon" type="image/png" sizes="32x32" href="/images/site-icon-32-transparent.png">
   <link rel="icon" type="image/png" sizes="192x192" href="/images/site-icon-192-transparent.png">
   <link rel="apple-touch-icon" sizes="180x180" href="/images/site-icon-180.png">
   <meta property="og:image" content="{base_url}/images/site-icon-512.png">
   ```
   测试断言同步。
4. 预览：明/暗两种标签页模拟（确认无白框、暗色下可见）+ 多尺寸；截图 `preview.png`。
5. 上线本机（Lead 已获本机部署授权）：重建 release + 重启一次，验证端点与 head，回归通过。

## Acceptance

- [ ] 透明 favicon 无白框；暗色变体在深色底上轮廓/五官可见。
- [ ] SVG 在支持 `prefers-color-scheme` 的浏览器里自动切换；不支持时回退透明 PNG。
- [ ] apple-touch / og:image 仍白底（不透明）。
- [ ] 门禁全绿；本机上线后端点 200、head 就位、9 路由回归通过。
- [ ] 提交、journal、归档。
