# 小尺寸 favicon 清晰化（16/32/48 专用简化版）

## Goal

用户反馈浏览器标签页里的图标「非常糊」。根因：现有 favicon 是 32/192 的**高细节**插画
（粉笔笔触、问号、星星、爱心），浏览器把它缩到 16px 渲染，细节糊成一团。

做法：为小尺寸单独出一套**简化版**（脸部特写 + 加粗描边 + 提高对比 + 去掉装饰），
并让 head 按尺寸列出 16/32/48，浏览器可直接取对应尺寸，不再靠缩放。

## 方案

1. `tools/make-icons.py` 增加小尺寸模式（保留现有 lineart / bleed）：
   - 脸部特写裁切（比现在更紧，眼睛+嘴占据主要面积；`--crop` 约 0.45–0.55 区间试验后取最佳）
   - 描边加粗：对 alpha 做 1–2 像素膨胀（16px 用更粗），保证缩小后线仍在
   - 提高对比：加深主体、去掉半透明噪点（例如 alpha 阈值化）
   - 去掉装饰（星星/爱心/问号若在裁切框外自然消失；若仍在框内则按需清理）
   - 输出 `site-icon-16.png`、`site-icon-32.png`、`site-icon-48.png`（透明底）与对应暗色变体
     `site-icon-16-dark.png`、`-32-dark.png`、`-48-dark.png`
   - 现有 192 透明/暗色、180 白底、512 白底保持不变
2. `site-icon.svg` 重建：内嵌**简化版 48px** 的亮/暗两张（而不是 192 细节版），
   这样 SVG 缩到 16px 时仍是粗线条、不会糊。
3. `rust/src/templates.rs` head：
   ```html
   <link rel="icon" type="image/svg+xml" href="/images/site-icon.svg">
   <link rel="icon" type="image/png" sizes="16x16" href="/images/site-icon-16.png">
   <link rel="icon" type="image/png" sizes="32x32" href="/images/site-icon-32.png">
   <link rel="icon" type="image/png" sizes="48x48" href="/images/site-icon-48.png">
   <link rel="icon" type="image/png" sizes="192x192" href="/images/site-icon-192-transparent.png">
   <link rel="apple-touch-icon" sizes="180x180" href="/images/site-icon-180.png">
   <meta property="og:image" content="{base_url}/images/site-icon-512.png">
   ```
   测试断言同步。
4. 预览（`.run/review/favicon-crisp/`）：
   - **前后对比**：旧 32px 缩到 16px vs 新 16px 专用版，1x 与 8x 放大并排
   - 16/32/48 三档在明/暗标签页模拟下的效果
   - 截图 `preview.png`
5. 上线本机（Lead 已获本机部署授权）：重建 release + 重启一次；验证端点与 head、回归通过。

## Acceptance

- [ ] 16px 专用版在 1x 下能看出「黑发 + 白脸 + 两只眼」且线条不糊（与旧版对比明显改善）。
- [ ] head 含 16/32/48/192 四档 PNG + SVG + apple-touch + og。
- [ ] 透明版角 alpha=0；暗色变体保留彩色（蓝眼）。
- [ ] 门禁 fmt / clippy / test 全绿；本机上线后端点与回归通过。
- [ ] 提交、journal、归档。
