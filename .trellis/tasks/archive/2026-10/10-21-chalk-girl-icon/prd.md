# 换用粉笔风少女涂鸦做图标（透明 + 暗色变体）

## Goal

用户指定新图（粉笔风少女涂鸦，米白底）作为站点图标。这张与熊脸同类（浅底 + 深色主体），
走既有 `lineart` 路径：bbox 裁切 → 白底透明化 → 暗色变体 → SVG 明暗自适应，
从而既没有白框（用户上次明确不要），暗色标签页也看得见。

## 输入

- `tools/assets/chalk-girl-icon-source.png`（1254x1254，米白底，黑发 + 蓝眼 + 蓝问号 + 星星/爱心，
  sha256 5b03d7a4…）。

## 交付

1. 用 `tools/make-icons.py` 的 lineart 模式（源图传参）重新生成全套：
   - `site-icon-32-transparent.png`、`site-icon-192-transparent.png`（透明底原色）
   - `site-icon-32-dark.png`、`site-icon-192-dark.png`（暗色变体）
   - `site-icon-180.png`（白底 apple-touch）、`site-icon-512.png`（白底 og:image）
   - `site-icon.svg`（内嵌亮/暗两版，`prefers-color-scheme` 切换）
2. 暗色变体映射需要**保留彩色元素**（蓝眼、蓝问号、粉腮红、黄星星、红爱心），不能把它们变灰：
   - 近中性深色（黑发/描边）→ 近白（约 #f2f2f2）
   - 近中性浅色（米白底/脸部）→ 深灰（约 #2b2b2b）
   - 高饱和彩色（蓝/粉/黄/红）→ 保留原色（可略提亮）
3. `rust/src/templates.rs` head 恢复为五处：svg 首选 + 32/192 transparent 回退 + apple-touch 180 + og:image 512；
   测试断言同步。
4. 预览（`.run/review/chalk-girl-icon/`）：16/32/48/180/512 多尺寸 + 透明 vs 白底对比 +
   明/暗标签页模拟（确认明色无白框、暗色下黑发变亮可辨）；截图 `preview.png`。
5. 上线本机（Lead 已获本机部署授权）：重建 release + 重启一次；验证端点 200、head 五处、回归通过。

## Acceptance

- [ ] 32px 下能看出「黑发 + 蓝眼 + 问号」特征。
- [ ] 透明版角像素 alpha = 0（无白框）；暗色变体在深色底上主体可辨、彩色元素保留。
- [ ] head 五处就位；旧插画四 PNG 的引用被替换（同名文件被覆盖即可）。
- [ ] 门禁 fmt / clippy / test 全绿；本机上线后端点与回归通过。
- [ ] 提交、journal、归档。
