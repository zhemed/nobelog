# 换用橙发少女插画做网站图标

## Goal

用户指定新的满幅动漫插画（橙发少女 + 橙色连帽衫）作为站点图标。该图与之前的线稿不同：
背景满幅、无白底、无透明需求，所以走新的「bleed（满幅插画）」处理路径：裁切 → 缩放 → 不透明 PNG。

## 输入

- `tools/assets/girl-icon-source.jpg`（1024x1024，满幅插画，sha256 6a1de4ee…）。

## 交付

1. `tools/make-icons.py` 增加模式（保留原 lineart 路径不动）：
   - `--mode bleed`：不做白底/暗色/透明处理；按参数裁切（默认**脸部特写**：以画面中心偏上为中心，
     边长约原图 62%，可用 `--crop` 调整），LANCZOS 缩放，输出不透明 PNG。
   - 另出两张**仅预览用**对比图：`full`（整幅缩到方形）与 `face`（脸部特写）。
2. `content/images/`（替换现有熊脸图标）：
   - `site-icon-32.png`、`site-icon-192.png`、`site-icon-180.png`、`site-icon-512.png`（不透明，脸部特写）
   - 删除不再使用的 `site-icon.svg`、`site-icon-32-transparent.png`、`site-icon-192-transparent.png`、
     `site-icon-32-dark.png`、`site-icon-192-dark.png`、`site-icon-transparent-512.png`（熊脸专用产物）
3. `rust/src/templates.rs` head 改为：
   ```html
   <link rel="icon" type="image/png" sizes="32x32" href="/images/site-icon-32.png">
   <link rel="icon" type="image/png" sizes="192x192" href="/images/site-icon-192.png">
   <link rel="apple-touch-icon" sizes="180x180" href="/images/site-icon-180.png">
   <meta property="og:image" content="{base_url}/images/site-icon-512.png">
   ```
   测试断言同步。
4. 预览（`.run/review/girl-icon/`）：16/32/48/180/512 多尺寸 + **整幅 vs 脸部特写对比** +
   明/暗标签页模拟；截图 `preview.png`。
5. 上线本机（Lead 已获本机部署授权）：重建 release + 重启一次；验证端点 200、head 就位、回归通过。

## Acceptance

- [ ] 32px 下能看出「橙发 + 大眼睛」的特征（脸部特写优先）。
- [ ] 模板 head 指向 4 个不透明 PNG；熊脸专用文件已删除。
- [ ] 门禁 fmt / clippy / test 全绿；静态产物 `images/` 只含新图标与 test-cover.svg。
- [ ] 本机上线后端点 200、head 正确、9 路由回归通过。
- [ ] 提交、journal、归档。
