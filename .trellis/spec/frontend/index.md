# Frontend Guidelines（本站的"前端"= 服务端渲染的 HTML / CSS / 浏览器 JS）

> nobelog 没有 React/Vue/TypeScript 前端工程，也没有前端构建框架。
> "前端"由 Rust 模板层生成 HTML、自建 CSS/JS 资源、再由 axum 直出或静态托管。
> 所以这里记录的是：**内容 → 渲染 → 资源 → 路由** 这条链上的真实约定。

---

## 一页速览

| 层 | 代码位置 | 职责 |
| --- | --- | --- |
| 内容层 | `rust/src/content.rs`、`rust/src/model.rs` | 读 `content/`、解析 frontmatter、排序、分类聚合 |
| Markdown | `rust/src/markdown.rs`、`highlight.rs`、`highlight_map.rs` | Markdown → HTML、TOC、提示块、代码高亮 |
| 模板层 | `rust/src/templates.rs` | 拼 HTML 字符串（layout / 侧栏 / 卡片 / RSS / sitemap） |
| 路由层 | `rust/src/routes.rs` | path → `RenderedPage`（纯函数，含 404） |
| 资源层 | `rust/src/assets.rs`、`rust/tools/css_prune.rs` | CSS 拼接/压缩/裁剪、JS 压缩、内容哈希命名 |
| 服务层 | `rust/src/serve.rs` | 静态模式 / 动态模式、缓存头、压缩、Range |
| 浏览器 JS | `rust/client/main.js` | 亮暗三态、移动端侧栏、代码复制、TOC 跟随、图片浮层 |
| 样式 | `vendor/chirpy/main.css`、`vendor/fonts/fonts.css`、`vendor/github-markdown/github-markdown.css`、`vendor/patches/patch.css` | 上游样式 + 本站补丁 |

命令入口：`rust/src/main.rs`（`build` / `serve` / `diff` 三个子命令）。

---

## 指南索引

| 指南 | 内容 | 什么时候读 |
| --- | --- | --- |
| [目录结构](./directory-structure.md) | 各目录归属、可写性、"产物 vs 源"边界 | 动手前、新增文件前 |
| [内容与 frontmatter](./content-and-frontmatter.md) | 文章字段、日期时区、排序、摘要、阅读时长 | 改内容解析、加字段 |
| [渲染与模板](./rendering-templates.md) | 模板函数契约、转义、日期格式、分页、RSS/sitemap | 改 HTML 结构 |
| [Markdown 管线](./markdown-pipeline.md) | 提示块、标题锚点、代码块、TOC、共享渲染器 | 改 Markdown 输出 |
| [样式与资源](./styling-and-assets.md) | 样式装配顺序、暗色模式、补丁 CSS、哈希命名、CSS 裁剪 | 改视觉、加资源 |
| [路由与 URL](./routes-and-urls.md) | 路由表、尾斜杠、百分号编码、`SITE_URL` 优先级 | 加路由、改链接 |
| [构建 · 服务 · 部署](./build-serve-deploy.md) | 构建两阶段、静态/动态服务、缓存头、打包、systemd | 跑构建、部署、排障 |
| [质量基线](./quality-guidelines.md) | fmt/clippy/test/diff、提交前缀、禁止事项 | 提交前 |

---

## 不可协商的三条

1. **`public/` 是产物，不是源码。** 不要手改 `public/` 下的 html/css/js；
   构建产物默认写 `rust/dist/`（`--out` 可改）。`public/` 里只允许手工维护
   `CNAME`、`_headers`、`_redirects`、`robots.txt`、`.gitkeep`（见 `rust/src/paths.rs` 的 `PRESERVE_IN_PUBLIC`）。
2. **所有用户可控文本必须经 `escape_html`。** 模板层拼字符串，没有框架帮你转义；
   Markdown 正文与 frontmatter 值进入 HTML 前都要过模板层的转义函数。
3. **URL 一律走 `crate::url::{encode,decode}` 与 `templates::{post_url,tag_url,category_url}`。**
   不要手拼 `format!("/tags/{}", name)`；中文标签/分类会漏编码，静态产物目录名与请求路径就会对不上。

---

## 与 TypeScript 版本的关系（重要背景）

本仓库是 `src/*.ts` + `server/*.ts` 的 Rust 重写，兼容基线是**逐字节一致的 HTML 输出**：

- `markdown.rs` 目标是对 `markdown-it` 产物的逐字节移植（唯一例外是代码块 token 层，用 syntect 替代 highlight.js，见该文件头注释）。
- `main.rs diff` 子命令拿 `public/`（冻结基准）与 `dist/` 逐字节对比，`--normalize-assets` 归一化资源哈希。
- 因此"顺手美化输出"（多加空格、换行、属性顺序）**会破坏兼容性**：改模板/markdown 输出后必须跑 `diff`。
