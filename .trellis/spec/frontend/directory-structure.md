# 目录结构

## 仓库布局（真实路径）

| 路径 | 归属 | 可写性 | 说明 |
| --- | --- | --- | --- |
| `content/posts/*.md` | 内容源 | ✅ 正常编辑 | 文章，文件名即兜底 slug（`slug` frontmatter 优先） |
| `content/pages/*.md` | 内容源 | ✅ 正常编辑 | 单页（`about.md` → `/about/`） |
| `content/images/` | 内容源 | ✅ 加图片 | 站内图片，构建时复制到产物 `images/` |
| `rust/src/*.rs` | 源码 | ✅ | 见下方模块表 |
| `rust/client/main.js` | 浏览器 JS 源码 | ✅ | 唯一的前端脚本，无依赖、无打包器 |
| `rust/tools/css_prune.rs` | 构建工具 | ✅ | 被 `assets.rs` 以 `#[path]` 引入，不单独成 crate |
| `rust/tools/css_report.rs` | 构建工具 | ✅ | CSS 使用情况报告 |
| `rust/examples/render.rs` | 对照工具 | ✅ | 输出 JSONL，供与 TS 版 `oracle.mjs` 对比 |
| `vendor/` | 上游样式 | ⚠️ 只读为主 | 主题/字体/图标，见"vendor 规则" |
| `public/` | **构建产物（冻结基准）** | ⛔ 不手改 | 静态构建产物 + `assets-manifest.json` |
| `rust/dist/` | Rust 构建产物 | ⛔ 不手改 | `nobelog build` 默认输出目录，已被 `.gitignore` 忽略 |
| `tools/package.sh` | 发布脚本 | ✅ | 打 `dist/nobelog-<version>-linux-x86_64.tar.gz` |
| `tools/make-icons.py` | 图标生成 | ✅ | 从 `tools/assets/*-source.*` 生成 `content/images/cat-icon*`（SVG 明暗自适应 + PNG 兜底 + apple-touch + og:image） |
| `tools/assets/` | 设计源文件 | ✅ | 不入构建产物（源图不放 `content/images`，避免进发布包） |
| `.trellis/` | Trellis 流程 | ✅ 按流程 | 任务/spec/journal，见 `.trellis/workflow.md` |

根目录 `package.json` 只是 pnpm 别名（`serve` / `serve:static` / `build` / `dev` / `start`），
没有 `node_modules`、没有前端构建链，不要为它引入 npm 依赖。

## Rust 模块表

| 模块 | 职责 | 改它时注意 |
| --- | --- | --- |
| `main.rs` | CLI：`build` / `serve` / `diff`；`default_root()` 定位仓库根 | 打包后的二进制不能依赖编译机路径，`NOBELOG_ROOT` > cwd > 可执行文件目录 > `CARGO_MANIFEST_DIR` 父目录 |
| `site.rs` | 站点常量 `SITE`（标题/导航/社交/分页数/许可）与 `env_url()` | 模板会原样输出，改字段等于改全站文案 |
| `model.rs` | `Post` / `SitePage` / `Taxonomy` / `Content` | `taxonomy = IndexMap`，顺序 = 首次出现顺序 |
| `content.rs` | 解析 frontmatter、排序、摘要、阅读时长、分类聚合 | 详见 content-and-frontmatter.md |
| `markdown.rs` | Markdown → HTML + TOC | 详见 markdown-pipeline.md；输出需与 TS 版一致 |
| `highlight.rs` / `highlight_map.rs` | syntect 高亮 + hljs 类名映射 | token 层允许与 TS 版不同，类名结构必须保持 |
| `templates.rs` | 所有 HTML 字符串 | 详见 rendering-templates.md；转义与字节级输出 |
| `routes.rs` | 路由表与 `Site::page` / `all_routes` | 新路由要同时进 `collect_paths()`，否则 sitemap 漏页 |
| `assets.rs` | CSS/JS 构建、哈希命名、manifest、字体复制、CSS 裁剪 | 详见 styling-and-assets.md |
| `serve.rs` | axum 服务：静态/动态两模式、缓存头、压缩、Range | 改动要跑 `cargo test`（含路径穿越用例） |
| `paths.rs` | `STYLE_ORDER` / `VENDORED_ASSETS` / `PRESERVE_IN_PUBLIC` | 改样式顺序或产物保留名单只改这里 |
| `url.rs` | `encode` / `decode`，对齐 JS `encodeURIComponent` | 不要换库，字符集是刻意对齐的 |

## vendor 规则

- `vendor/chirpy/main.css` 是**上游主题产物（已压缩）**、`vendor/github-markdown/github-markdown.css` 是上游
  GitHub Markdown 样式、`vendor/fonts/` 是自托管字体（Lato / Source Sans Pro，含 OFL 许可文本）。
- **本站视觉调整一律写进 `vendor/patches/patch.css`**，按注释分区（调色板 / 版式 / 组件细节 …）追加，
  不要回改上游文件。`patch.css` 是最后加载的一层，所以补丁天然覆盖上游。
- 字体文件名是主题 CSS 里 `url(/assets/fa-*.woff2)`、`lato-*.woff2` 写死的，
  **不要重命名**，否则运行时字体 404（`VENDORED_ASSETS` 只列了 FA 四个，字体由 `copy_vendored_assets` 一并复制）。

## 反模式

- 在 `public/` 里"临时手改"某个 html 修样式 → 下次构建即丢，且污染 diff 基准。改模板或 `patch.css`。
- 往 `rust/src/` 里加一个"只在构建时用"的脚本却忘了它会被编译进二进制 → 构建工具放 `rust/tools/`，用 `#[path]` 引入。
- 新建 `rust/client/*.ts` 或引入打包器 → 本站只有 `rust/client/main.js` 这一个脚本，保持零依赖。
