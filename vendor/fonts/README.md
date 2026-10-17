# vendor/fonts —— 自托管的拉丁正文字体

主题样式（`vendor/chirpy/main.css`）声明了 `body → "Source Sans Pro","Microsoft YaHei",sans-serif`、
标题 → `Lato,"Microsoft YaHei",sans-serif`，但**没有提供对应的 @font-face** ——
装了这两个字体的人能看到设计字形，没装的人看到的是系统回退字形。
本目录把这两个字体族自托管进来，让所有访客看到同一套字形。

## 抓取方式（可复现）

```bash
node scripts/fetch-fonts.mjs            # 幂等：内容未变则跳过下载，重建 fonts.css 并清理多余文件
node scripts/fetch-fonts.mjs --force    # 强制重抓
```

- 来源：`fonts.googleapis.com`（CSS，现代 UA 才返回 woff2）→ `fonts.gstatic.com`（woff2 文件）
- 抓取时间：2026-10-17（CST）
- 只取 **latin + latin-ext** 子集：中文等 CJK 字符不在这些子集内，
  按主题设计**回退到系统字体**（Microsoft YaHei 等）——与原站行为一致。

## 文件清单（10 个 woff2，合计约 218 KB）

| 文件 | 大小 | sha256（前 16 位） | 说明 |
|---|---:|---|---|
| `lato-400-latin.woff2` | 23.0 KB | `918b7dc3e2e2d015` | 标题常规 |
| `lato-400-latin-ext.woff2` | 5.4 KB | `8b9fc9737043f88c` | 同上，扩展拉丁子集 |
| `lato-700-latin.woff2` | 22.5 KB | `c447dd7677b419db` | 标题粗体 |
| `lato-700-latin-ext.woff2` | 5.4 KB | `3a5797f440bc67b5` | 同上 |
| `lato-900-latin.woff2` | 21.9 KB | `08664859baab5ed9` | 标题特粗 |
| `lato-900-latin-ext.woff2` | 5.5 KB | `e266d64fae095874` | 同上 |
| `source-sans-pro-400-latin.woff2` | 28.1 KB | `19143dca075972bc` | 正文常规（**可变字体**，同时服务 400/600/700） |
| `source-sans-pro-400-latin-ext.woff2` | 58.6 KB | `a85a7459bdb3cdc1` | 同上，扩展拉丁子集 |
| `source-sans-pro-400-italic-latin.woff2` | 15.4 KB | `3c230c03425a2866` | 正文斜体（`<em>`） |
| `source-sans-pro-400-italic-latin-ext.woff2` | 31.4 KB | `97f4d6bf2b17b551` | 同上 |

- `fonts.css` 由脚本生成（14 条 @font-face），**请勿手工编辑**。
- ⚠️ **可变字体去重**：Google Fonts 对 Source Sans 3 的 400/600/700 返回**同一个文件**，
  所以 14 条 @font-face 只对应 10 个文件，多个字重共用同一个 URL（浏览器只下载一次）。
  统一文件名取首个命中者（`source-sans-pro-400-*.woff2`）。

## 许可

两个字体族均为 **SIL Open Font License 1.1**（允许免费使用、修改、**嵌入与再分发**，
包括商用；要求保留版权与许可声明，且不得单独售卖字体本身）：

- **Lato** —— Copyright (c) 2010-2014 by tyPoland Lukasz Dziedzic，Reserved Font Name "Lato"
  许可全文：[`OFL-Lato.txt`](./OFL-Lato.txt)（取自 `github.com/google/fonts/ofl/lato/OFL.txt`）
- **Source Sans 3**（原名 Source Sans Pro）—— Copyright 2010-2020 Adobe，Reserved Font Name 'Source'
  许可全文：[`OFL-Source-Sans.txt`](./OFL-Source-Sans.txt)（取自 `github.com/google/fonts/ofl/sourcesans3/OFL.txt`）

本目录只做**格式转换（原样下载 woff2，未改动字体数据）**，并保留上述版权与许可文本。

## 构建集成

- `server/paths.ts` 的 `STYLE_ORDER` 把 `vendor/fonts/fonts.css` 排在主题 CSS 之后、`patch.css` 之前；
- `server/build.ts` 会把本目录下所有 `*.woff2` 以**原文件名**拷到 `public/assets/`
  （不能加内容哈希 —— `fonts.css` 里写死了 `/assets/xxx.woff2`）；
- 因此首屏会多 1~3 个字体请求（只在实际用到对应字形时触发），全部同源、
  且带 `Cache-Control: public, max-age=31536000, …`。

## 回滚

删除 `vendor/fonts/` 与 `STYLE_ORDER` 里的 `vendor/fonts/fonts.css` 一行，重新构建 ——
即回到“依赖访客本机字体”的原状。
