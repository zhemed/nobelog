# nobelog 博客结构文档

> 文档路径：`docs/blog-tree.md`
> 核对基线：2026-11-01 08:05 UTC，分支 `main`，HEAD `70dd83a`，版本 `0.0.30`
> 核对方式：仓库只读命令与源码阅读，加本机 `127.0.0.1:10086` 动态模式实例的 HTTP 实测；本次只新增本文件，未改动其它文件，未重启服务
> 行号说明：本文所有 `文件:行号` 均基于 HEAD `70dd83a`，改动源码后请同步复核

---

## 一、文档说明

### 1. 目的

把博客的真实结构一次性落下来，供以后直接查阅，不再每次现场 grep：

- 内容放在哪里、怎么变成页面；
- 每个 URL 由哪个函数渲染、输出什么 DOM；
- CSS/JS 从哪些源文件来、如何拼接、如何命名、输出到哪些目录；
- 侧栏与整页的 DOM 树、主样式与补丁样式的分工；
- 深色模式与 849、850、1400、1650 四个关键断点的实际行为；
- 哪些文件可以改、哪些禁止改、改完怎么验证。

### 2. 站点一句话

`nobelog` 是一个 Rust 写的静态博客生成器：Markdown 内容经 Rust 渲染成 HTML/CSS/JS，由 `nobelog build` 输出静态产物，或由 `nobelog serve` 以静态/动态两种模式提供访问。没有 Node、没有数据库、没有前端框架、没有 CSS 预处理器。

### 3. 当前运行态

| 项目 | 实测值 | 取证方式 |
| --- | --- | --- |
| 版本 | `nobelog 0.0.30` | `./rust/target/release/nobelog --version` |
| 进程 | PID 443082，2026-11-01 07:50:40 UTC 启动 | `systemctl show nobelog-local` |
| 模式 | 动态模式 `--dynamic`，监听 `0.0.0.0:10086` | `ExecStart` 与 `ss -tlnp` |
| 站点绝对地址 | 未设置 `SITE_URL`，按请求推断为 `http://127.0.0.1:10086` | `systemctl show -p Environment` 为空，页面 canonical 实测 |
| 内容规模 | 2 篇文章、1 个单页、8 张图片、4 个标签、2 个分类 | 见第三章 |

---

## 二、仓库目录树与文件职责

### 1. 顶层目录

| 路径 | 角色 | 入库 | 说明 |
| --- | --- | --- | --- |
| `content/` | 内容源 | 是 | 文章、单页、站内图片，唯一的内容入口 |
| `rust/` | 生成器与服务 | 是 | Rust 源码、客户端 JS、构建工具、示例、数据 |
| `vendor/` | 上游样式与字体 | 是 | Chirpy 主题、GitHub Markdown 样式、自托管字体、本站补丁 |
| `tools/` | 运维辅助脚本 | 是 | 图标生成 `make-icons.py`、发布打包 `package.sh` |
| `docs/` | 文档 | 是 | 本文件所在目录，本次新增 |
| `.trellis/` | Trellis 流程 | 是 | 任务、规范 spec、workspace journal、脚本 |
| `.agents/`、`.dsh/` | 项目级 AI 技能 | 是 | Trellis 生成的平台技能副本 |
| `public/` | 静态产物与冻结基准 | 否 | 被 `.gitignore` 忽略，禁止手改 |
| `rust/dist/` | Rust 构建产物 | 否 | `nobelog build` 的默认输出目录 |
| `dist/` | 发布包 | 否 | `nobelog-<version>-linux-x86_64.tar.gz` 与 `SHA256SUMS` |
| `rust/target/` | cargo 编译缓存 | 否 | 体积约 7.1G，不影响仓库 |

`.gitignore` 关键项：`rust/target/`、`rust/dist/`、`public/`、`dist/`、`.backups/`、`*.log`。
仓库根**没有** `package.json`、`node_modules` 或任何前端构建链，构建与运行一律走 cargo。

### 2. Rust 模块职责

| 文件 | 行数 | 职责 |
| --- | --- | --- |
| `rust/src/main.rs` | 251 | CLI 入口与子命令：`build`、`serve`、`diff`；`default_root()` 定位仓库根 |
| `rust/src/build.rs` | 499 | 构建器：清空输出目录、准备资源、写页面/404/robots、统计输出 |
| `rust/src/content.rs` | 1187 | 读 `content/`、解析 frontmatter、排序、摘要、分类/标签聚合、校验 |
| `rust/src/model.rs` | 52 | 中间表示：`Post`、`SitePage`、`Taxonomy`、`Content` |
| `rust/src/site.rs` | 128 | 站点常量 `SITE`：标题、导航、社交、分页数、许可；`SITE_URL` 读取 |
| `rust/src/paths.rs` | 20 | 路径常量：`STYLE_ORDER`、`VENDORED_ASSETS`、`PRESERVE_IN_PUBLIC` |
| `rust/src/routes.rs` | 537 | 路由表与页面装配：`Site::page`、`collect_paths`、`all_routes` |
| `rust/src/templates.rs` | 1698 | 所有 HTML 字符串：外壳、侧栏、卡片、目录、RSS、sitemap |
| `rust/src/markdown.rs` | 1165 | Markdown → HTML + TOC，提示块、代码块、表格、图片升级 |
| `rust/src/highlight.rs` | 484 | syntect 高亮，输出 highlight.js 类名 |
| `rust/src/highlight_map.rs` | 334 | syntect scope → `hljs-*` 类名映射 |
| `rust/src/assets.rs` | 881 | CSS/JS 构建、压缩、裁剪、内容哈希、manifest、字体复制 |
| `rust/src/serve.rs` | 2150 | axum 服务：静态/动态模式、缓存头、压缩、Range、热更新 |
| `rust/src/url.rs` | 24 | `encode`/`decode`，对齐 JS `encodeURIComponent` |
| `rust/client/main.js` | 501 | 唯一浏览器脚本：亮暗三态、移动侧栏、回顶、复制、TOC、分享、图片浮层 |
| `rust/tools/css_prune.rs` | 1306 | CSS 保守裁剪实现，被 `assets.rs` 以 `#[path]` 引入 |
| `rust/tools/css_report.rs` | 251 | 离线裁剪报告工具，不被 cargo 构建，需 `rustc` 单独编译 |
| `rust/examples/render.rs` | 89 | 渲染对照工具，输出 JSONL，用于和 TS 基准比对 |
| `rust/data/README.md` | 28 | 说明 `highlight-golden.json` 冻结语料，但该 json 当前不在仓库，见第九章 |

### 3. rust/ 子目录

| 路径 | 内容 |
| --- | --- |
| `rust/src/` | 14 个 `.rs` 源文件，全部编进单个二进制 |
| `rust/client/main.js` | 浏览器脚本源码，构建时压缩后内联进 `assets/main.<hash>.js` |
| `rust/tools/` | 构建期工具源码，`css_prune.rs` 以 `#[path]` 引入主 crate |
| `rust/examples/render.rs` | cargo example，`cargo run --example render` 可执行 |
| `rust/data/README.md` | 高亮 golden 语料说明，当前只有说明文件 |
| `rust/dist/` | 构建产物目录，当前为 2026-11-01 的试验构建，见第九章 |
| `rust/Cargo.toml` | 包名 `nobelog`、版本 `0.0.30`、edition 2024、rust-version 1.99 |
| `rust/rust-toolchain.toml` | 固定 Rust 1.99.1，含 rustfmt 与 clippy |
| `rust/Cargo.lock` | 锁文件，发布要求 `--locked` |

依赖栈要点：`axum 0.8`、`tower-http 0.6`、`tokio`、`pulldown-cmark 0.13`、`syntect 5`、`serde_yaml`、`chrono`、`sha2`、`rayon`、`indexmap`、`walkdir`、`percent-encoding`、`regex`、`clap`。

### 4. vendor/ 与 tools/

| 路径 | 内容 | 可写性 |
| --- | --- | --- |
| `vendor/chirpy/main.css` | 上游 Bootstrap 5.3.2 + Font Awesome 6.4.2 + Chirpy 6.1.0 官方编译产物，单行压缩，404272 字节 | 只读 |
| `vendor/chirpy/*.woff2` | Font Awesome 四个字体文件，文件名被 CSS 写死 | 只读 |
| `vendor/fonts/fonts.css` | 14 个 `@font-face` 声明，Lato 与 Source Sans Pro | 只读 |
| `vendor/fonts/*.woff2` | 自托管拉丁字体，10 个文件 | 只读 |
| `vendor/github-markdown/github-markdown.css` | GitHub Markdown 样式，作用于 `.markdown-body` | 只读 |
| `vendor/patches/patch.css` | 本站唯一手工样式层，1244 行，分四段 | 可追加 |
| `tools/make-icons.py` | 从 `tools/assets/*-source.*` 生成 `content/images/site-icon*` 全套图标 | 可改 |
| `tools/package.sh` | 发布打包：cargo 构建、重建 `public/`、tar + SHA256SUMS | 可改 |
| `tools/assets/` | 图标设计源图，不进构建产物 | 可改 |

### 5. 流程与文档位置

| 路径 | 用途 |
| --- | --- |
| `README.md` | 面向使用者的说明：写文章、本地运行、目录、部署提示 |
| `rust/README.md` | Rust 重写版说明：命令、模块表、验收标准、已知取舍 |
| `AGENTS.md` | AI 助手入口，指向 `.trellis/` |
| `.trellis/workflow.md` | Trellis 开发流程权威定义 |
| `.trellis/spec/frontend/index.md` | 前端规范索引，8 篇专题，本文件的权威依据 |
| `.trellis/spec/frontend/*.md` | 目录结构、内容与 frontmatter、渲染模板、Markdown 管线、样式资源、路由 URL、构建部署、质量基线 |
| `.trellis/workspace/zhemed/` | 开发者 journal 与索引 |

---

## 三、内容树与站点数据

### 1. content/ 结构

```text
content/
├── posts/                      文章目录，文件名即兜底 slug
│   ├── .gitkeep
│   ├── server-init-commands.md
│   └── typeset-test.md
├── pages/                      单页目录，slug 对应 /<slug>/
│   └── about.md
└── images/                     站内图片，构建时复制到产物 images/
    ├── site-icon.svg
    ├── site-icon-180.png
    ├── site-icon-192-dark.png
    ├── site-icon-192-transparent.png
    ├── site-icon-32-dark.png
    ├── site-icon-32-transparent.png
    ├── site-icon-512.png
    └── test-cover.svg
```

### 2. 文章清单

| 文件 | 标题 | slug | 日期 | 标签 | 分类 | 其它 |
| --- | --- | --- | --- | --- | --- | --- |
| `content/posts/typeset-test.md` | 全要素排版测试 | `typeset-test` | 2026-10-17 22:40:00 | 排版、测试 | 调试 | `pinned: true`，封面 `/images/test-cover.svg`，正文 130 行 |
| `content/posts/server-init-commands.md` | 常用服务器初始化命令 | `server-init-commands` | 2026-10-18 12:15:00 | 运维、Linux | 备忘 | 正文 98 行 |

正文总量 249 行，其中文章 228 行、单页 21 行。

### 3. 单页清单

| 文件 | 标题 | slug | 访问路径 |
| --- | --- | --- | --- |
| `content/pages/about.md` | 关于 | `about` | `/about/` |

### 4. 分类与标签聚合

| 维度 | 数量 | 名称 | 排序规则 |
| --- | --- | --- | --- |
| 标签 | 4 | 排版、测试、运维、Linux | 总览按文章数降序；同级顺序继承首次出现顺序 |
| 分类 | 2 | 调试、备忘 | 总览按文章数降序 |

- 聚合实现在 `rust/src/content.rs:447` 的 `build_taxonomy`，容器是 `IndexMap`，键顺序等于标签/分类在文章排序后首次出现的顺序，见 `rust/src/model.rs:44`。
- 标签详情页示例：`/tags/排版/` 与编码形式 `/tags/%E6%8E%92%E7%89%88/` 均返回 200。
- 分类详情页示例：`/categories/备忘/` 返回 200。
- 标签/分类总览排序见 `rust/src/templates.rs:1325-1326`。

### 5. frontmatter 字段与解析

| 字段 | 类型 | 缺省行为 |
| --- | --- | --- |
| `title` | string | 回退到 slug，再回退到文件名主干 |
| `date` | string | 无法解析时回退文件 mtime，再回退 Unix epoch |
| `slug` | string | 回退文件名主干 |
| `tags` | 数组或逗号串 | 空数组；字符串按英文逗号与中文逗号切分 |
| `categories` | 数组或逗号串 | 同上 |
| `description` | string | 空则取正文第一个 `<p>` 的纯文本前 160 个 UTF-16 单元 |
| `cover` | string | 无封面，卡片不渲染预览图 |
| `pinned` | bool | `false`；`true` 时排在首页最前 |
| `draft` | bool | `false`；`true` 在 `content::load` 阶段直接丢弃 |
| `updated` | string | 无更新时间；非法值 fail-fast |

解析细节以 `.trellis/spec/frontend/content-and-frontmatter.md` 为准，关键点：

- 日期统一存为 UTC+8 挂钟值，`Post::date` 的 `Utc` 只是载体，见 `rust/src/content.rs:372-433` 与 `rust/src/templates.rs:1509-1516`。
- RSS 输出前减 8 小时还原真实 UTC，见 `rust/src/templates.rs:1516` 的 `rss_pub_date`。
- 排序：`pinned` 优先，其余按日期倒序，稳定排序，见 `rust/src/content.rs:39-44`。
- 摘要与阅读时长见 `rust/src/content.rs:26-37`。
- 单页 slug 拒绝保留名，避免与固定路由抢路径，见 `rust/src/content.rs:300-321`。

### 6. 站点配置 site.rs

`rust/src/site.rs:63-128` 的 `SITE` 是编译期常量，模板原样输出：

| 字段 | 值 |
| --- | --- |
| `url` | `http://127.0.0.1:10086`，内置默认，可被环境变量 `SITE_URL` 覆盖 |
| `lang` | `zh-CN` |
| `title` | 吱吱 |
| `subtitle` | 冷知识 |
| `description` | 吱吱的冷知识：记录有用没用的小知识。 |
| `posts_per_page` | 8 |
| `license_name` | CC BY 4.0 |
| `license_note` | 除特别声明外，本站文章均按 CC BY 4.0 协议授权。 |

导航 5 项，见 `rust/src/site.rs:97-123`：首页 `/`、分类 `/categories/`、标签 `/tags/`、归档 `/archives/`、关于 `/about/`。
社交 4 项，见 `rust/src/site.rs:75-96`：GitHub、X、邮件、RSS，当前 `href` 全为空，渲染时降级为 `.social-disabled` 并带 `aria-disabled="true"`。

`SITE_URL` 读取与归一化见 `rust/src/site.rs:37-47`，缺失时启动只告警一次，见 `rust/src/site.rs:52-61`。

### 7. 派生数据

| 数据 | 规则 | 代码 |
| --- | --- | --- |
| 摘要 summary | `description` 优先；否则首个 `<p>` 去标签后按 UTF-16 截 160 单元 | `rust/src/content.rs:26-35` |
| 阅读时长字段 | 拉丁词数 /220 加 CJK 字数 /400，最少 1 分钟 | `rust/src/content.rs:533` |
| 文章页阅读时长展示 | 正文非空白字符 UTF-16 计数，分钟下限 `/450`、上限 `/350`，显示区间 | `rust/src/templates.rs:1056-1064` |
| 卡片日期 | 刚刚、N 小时前、昨天、N 天前，超过 30 天显示绝对日期 | `rust/src/templates.rs:124-147` |
| 文章页日期 tooltip | 月日不补零，双空格分隔，显示周几与 12 小时制 | `rust/src/templates.rs:102-123` |

注意：`Post.reading_minutes` 字段仍被计算，但当前模板没有使用它；文章页展示的分钟区间由 `post_page` 现场按 UTF-16 字数计算。见第九章差异记录。

---

## 四、路由与页面类型

### 1. 请求到页面的链路

静态模式：`nobelog serve` → `handle` → `serve_public` 命中 `public/` 文件后交给 tower-http `ServeFile`，未命中时按 404 处理。
动态模式：`nobelog serve --dynamic` → `handle_dynamic` → 字体与图片直读磁盘、内存资源直出、其余路径交给 `Site::page` 实时渲染。

两条链路的页面渲染入口相同：`rust/src/routes.rs:94` 的 `Site::page`，它只产出 `RenderedPage`，HTML 由 `rust/src/templates.rs` 里对应的 `pub fn` 拼装，外壳统一由 `render_layout` 生成。

### 2. 路由表

| 路径 | 状态 | 渲染函数 | PageKind |
| --- | --- | --- | --- |
| `/` | 200 | `home_page` | Home |
| `/page/{n}/` | 200 或 404 | `home_page` | Home |
| `/posts/{slug}/` | 200 或 404 | `post_page` | Post |
| `/archives/` | 200 | `archive_page` | Archive |
| `/tags/`、`/categories/` | 200 | `taxonomy_index` | Taxonomy |
| `/tags/{name}/`、`/categories/{name}/` | 200 或 404 | `list_page` | Taxonomy |
| `/{page-slug}/` | 200 | `about_page` | About |
| `/feed.xml` | 200 | `feed_xml` | Feed |
| `/sitemap.xml` | 200 | `sitemap_xml` | Sitemap |
| 其它 | 404 | `not_found_page` | NotFound |

路由分支实现见 `rust/src/routes.rs:112-307`，占位顺序为首页、分页、文章、归档、标签总览、分类总览、标签详情、分类详情、通用单页、feed、sitemap、兜底 404。

### 3. 路由规则

- **所有页面 URL 都带尾斜杠**，无尾斜杠的变体一律 404，不做 301：实测 `/about`、`/posts/typeset-test`、`/archives` 均为 404。
- `/page/{n}/` 越界返回 404；`n` 从 1 开始，`n=1` 不是合法页面，第一页就是 `/`。
- 单页 slug 只匹配单段路径，且必须存在于 pages 表；保留名与输出根同名文件不会走单页分支，见 `rust/src/content.rs:300-321`。
- 静态构建把 200 路由写成文件，另外写根 `404.html`，见 `rust/src/build.rs:56-75`。
- 路径解码后再匹配，中文标签/分类能命中，见 `rust/src/routes.rs:347` 的 `decode`。

### 4. 路由清单与产物数量

`collect_paths` 见 `rust/src/routes.rs:310-333`，生成 13 条路径：

- `/`、`/archives/`、`/tags/`、`/categories/` 共 4 条；
- `/page/{2..n}/`，当前只有 1 页，所以 0 条；
- 文章 2 条；
- 标签 4 条；
- 分类 2 条；
- 单页 1 条。

`all_routes` 见 `rust/src/routes.rs:335-344`，在 13 条之外追加 `/feed.xml`、`/sitemap.xml`，共 15 条路由；其中 13 条 200 页面写成 HTML 文件，加上根 `404.html`，`public/` 下共 14 个 HTML 文件。实测静态产物 `sitemap.xml` 含 13 个 `<url>`，`feed.xml` 含 2 个 `<item>`。

### 5. 页面类型与模板组成

| 页面 | 路由入口 | 模板函数与行号 | 关键 DOM |
| --- | --- | --- | --- |
| 首页与分页 | `routes.rs:112-139` | `home_page:879`、`post_list:621`、`post_card:590`、`pagination:642` | `#post-list` 下若干 `article.card-wrapper`，内含 `a.card.post-preview`、`h1.card-title`、`.card-text.content`、`.post-meta` |
| 文章页 | `routes.rs:141-171` | `post_page:1042`，辅助 `panel:431`、`toc_wrapper:532` | `header` 内 `h1[data-toc-skip]` 与 `.post-meta`；`.post-meta-line` 合并分类与标签；`.content.markdown-body` 正文；`.post-tail-wrapper` 许可与分享；相关文章区块按条件渲染 |
| 归档页 | `routes.rs:173-181` | `archive_page:1258` | `#archives.pl-xl-3` 按年分组，`.year.lead` 加 `ul.list-unstyled` 列表 |
| 标签总览 | `routes.rs:183-195` | `taxonomy_index:1322` | `h1.dynamic-title` 加 `#tags` 下 `.tag` 胶囊 |
| 分类总览 | `routes.rs:197-209` | `taxonomy_index:1322` | `h1.dynamic-title` 加 `.categories.card` 列表，每类一张卡 |
| 标签详情 | `routes.rs:211-236` | `list_page:930` | `#page-tag`，每行 `li.d-flex`，含链接、虚线、日期 |
| 分类详情 | `routes.rs:238-263` | `list_page:930` | `#page-category`，结构同标签详情 |
| 关于等单页 | `routes.rs:265-285` | `about_page:1448` | `h1[data-toc-skip]` 加 `.post-content.markdown-body`，无右侧目录 |
| RSS | `routes.rs:287-295` | `feed_xml:1521` | RSS 2.0，2 个 item，`link` 与 `guid` 使用 `base_url` |
| sitemap | `routes.rs:297-305` | `sitemap_xml:1573` | `urlset`，13 个 `url`，路径来自 `collect_paths` |
| 404 | `routes.rs:105-110`、`307` | `not_found_page:1473` | `.content` 内 `p.lead`，状态 404，缓存头 `no-store` |

### 6. 分页

- 每页文章数取 `SITE.posts_per_page`，当前为 8，见 `rust/src/site.rs:124`。
- 当前 2 篇文章，仅 1 页，`/page/2/` 实测 404，首页不渲染分页器。
- 分页器窗口、首尾页与省略号逻辑见 `rust/src/templates.rs:642-740`。
- 首页第一页始终存在，没有文章时也会推一个空页，见 `rust/src/routes.rs:39-62`。

### 7. 导航高亮与顶栏

- 侧栏导航渲染见 `rust/src/templates.rs:184-205` 的 `nav_list`，`is_active` 见 `rust/src/templates.rs:170-183`：首页精确匹配并兼容 `/page/N/`，其余去尾斜杠后做前缀匹配；因此文章页不会高亮任何导航项，归档页只高亮归档项。
- 顶栏标题按路径归类，见 `rust/src/templates.rs:318-335`：首页用站点名，归档、标签、分类、文章分别用「归档」「标签」「分类」「文章」，其它为空。
- 面包屑见 `rust/src/templates.rs:298-316` 与 `topbar:337-352`，首页面包屑固定为「主页」。

### 8. 公共外壳与 SEO

`render_layout` 见 `rust/src/templates.rs:743-837`，所有页面共用：

- `<html lang="zh-CN">`，`<head>` 内含 charset、viewport、两个 `theme-color`、`<title>`、description、generator、canonical、RSS alternate、6 个图标声明、首屏暗色脚本、CSS 与 JS 引用、OG 与 Twitter meta。
- `<title>` 规则：有页面标题时输出「页面标题 - 吱吱」，首页只输出「吱吱」。
- canonical 与 `og:url` 均为 `base_url` 加 `ctx.path`，`base_url` 来自 `SITE_URL` 或动态请求推断。
- 主栏栅格由右侧面板是否存在决定：面板为空时用 `col-12 col-lg-11 col-xl-12`，否则用 `col-12 col-lg-11 col-xl-9`，见 `rust/src/templates.rs:766-770`。
- 静态 `404.html` 构建时传入空的面板数据，所以是 `col-xl-12`；动态模式 404 走完整的 `Site::page` 上下文，面板非空，实际为 `col-xl-9`。这是两种模式的一个可见差异，见第九章。

---

## 五、渲染与资源管线

### 1. 静态构建链路

入口为 `rust/src/main.rs:75-82`，核心是 `rust/src/build.rs:18-89`：

1. 目标目录已存在则先清空，但保留 `PRESERVE_IN_PUBLIC` 中的 5 个文件名，见 `rust/src/paths.rs:19-20` 与 `rust/src/build.rs:91-107`。
2. 资源阶段：默认复用 `public/assets` 与 `public/assets-manifest.json`；带 `--self-assets` 时调用 `assets::build_assets` 自建，见 `rust/src/build.rs:30-43`。
3. 复制 `content/images` 到产物 `images/`，见 `rust/src/build.rs:45-49`。
4. `Site::new` 加载内容，`all_routes` 遍历并写出每个 200 页面，见 `rust/src/build.rs:51-63`。
5. 写根 `404.html`，见 `rust/src/build.rs:65-75`。
6. 写 `robots.txt`，自动生成的带标记行，手写的保留，见 `rust/src/build.rs:249-275`。

产物默认目录：`main.rs` 中 `build` 子命令的 `--out` 默认值是 `dist`，但规范要求写 `rust/dist/`，因为 `--out` 相对路径按当前工作目录解析，在仓库根写 `--out dist` 会清空根 `dist/` 的发布包。推荐命令见第八章。

### 2. 动态运行时链路

入口为 `rust/src/serve.rs:258-317` 的 `run`：

1. 动态模式初始化 `DynamicState::load`，见 `rust/src/serve.rs:110-127`：`build_runtime` 一次性构建内容、内存资源与内容指纹，见 `rust/src/serve.rs:240-252`。
2. 每个请求先查磁盘资源：`/assets/*.woff2` 查 vendor 字体，`/images/*` 查 `content/images`，见 `rust/src/serve.rs:654-668` 与 `879-917`。
3. `/assets-manifest.json` 直接返回内存 manifest，见 `rust/src/serve.rs:672-682`。
4. `/assets/main.<8 位 hex>.css|js` 返回内存资源，见 `rust/src/serve.rs:684-698` 与 `927-939`。
5. 其余路径实时渲染，见 `rust/src/serve.rs:711-741`。
6. 内容热更新：每次请求计算 `content/` 指纹，TTL 1 秒内复用上次结果，见 `rust/src/serve.rs:1099-1148`；指纹变化则 `spawn_blocking` 重建并原子替换；重建失败后退避 10 秒，见 `rust/src/serve.rs:54`、`57`、`169-190`。

### 3. CSS 装配顺序

样式顺序定义在 `rust/src/paths.rs:3-8`，拼接实现在 `rust/src/assets.rs:124-138`，顺序即层叠优先级：

| 顺序 | 源文件 | 大小 | 角色 |
| --- | --- | --- | --- |
| 1 | `vendor/chirpy/main.css` | 404272 字节 | Bootstrap 5.3.2、Font Awesome 6.4.2、Chirpy 6.1.0 官方编译产物，单行压缩 |
| 2 | `vendor/fonts/fonts.css` | 5315 字节 | 14 个 `@font-face`，Lato 与 Source Sans Pro |
| 3 | `vendor/github-markdown/github-markdown.css` | 31068 字节 | GitHub Markdown 正文样式，只作用于 `.markdown-body` |
| 4 | `vendor/patches/patch.css` | 33829 字节 | 本站补丁，最后加载，天然覆盖上游 |

四个源文件合计 474484 字节。构建时先按顺序直接拼接，再交给自写压缩器 `minify_css`，见 `rust/src/assets.rs:319`；默认再做一次保守的 CSS 裁剪，见 `rust/src/assets.rs:106-121` 与 `rust/tools/css_prune.rs`。当前 `public/` 里的最终 CSS 为 192311 字节。

裁剪开关与排障：

- `NOBELOG_CSS_PRUNE=0` 或 `off`、`false`、`no` 关闭裁剪，见 `rust/src/assets.rs:141-145`。
- `NOBELOG_CSS_REPORT=1` 打印裁剪统计，见 `rust/src/assets.rs:113-119`。
- 样式上线后消失，先关裁剪复测，确认是误删后修 `css_prune` 保守规则，不要长期关闭裁剪。

### 4. JS 装配

唯一源文件是 `rust/client/main.js`，501 行、16391 字节，零依赖，按 7 个编号区块组织：

| 区块 | 行号 | 功能 |
| --- | --- | --- |
| 1 | 14-51 | 亮暗三态，维护 `html[data-mode]` 与 localStorage |
| 2 | 53-64 | 移动端侧栏，开关 `body[sidebar-display]`，遮罩与 Esc 关闭 |
| 3 | 66-75 | 回到顶部，滚动超过 50px 显示 |
| 4 | 77-141 | 代码复制，`[data-copy]` 按钮配合 `.highlight` 容器 |
| 5 | 143-199 | TOC 跟随与折叠，`a.toc-link` 与 `is-active-li`、`is-active-link` |
| 6 | 201-248 | 分享里的复制链接，`[data-copy-link]` |
| 7 | 249-501 | 图片浮层，缩放、拖拽、双指手势 |

构建时经 `minify_js` 去注释与缩进，见 `rust/src/assets.rs:232-241` 与 `541`，当前产物为 12082 字节的 `main.ea281e5e.js`。

### 5. 内容哈希与产物命名

- 哈希算法：对最终内容取 `sha256`，取十六进制前 8 位，见 `rust/src/assets.rs:244-247`。
- 输出文件：`assets/main.<hash>.css`、`assets/main.<hash>.js`、`assets-manifest.json`，见 `rust/src/assets.rs:78-103`。
- manifest 格式为两行 pretty JSON，键是 `css` 与 `js`，值是带前导斜杠的绝对路径，当前内容：

```json
{
  "css": "/assets/main.1ca5c955.css",
  "js": "/assets/main.ea281e5e.js"
}
```

- 字体与图标文件名被上游 CSS 写死，原样复制、不加哈希，见 `rust/src/assets.rs:264-306` 与 `rust/src/paths.rs:11-16`。当前 `assets/` 共 16 个文件：2 个哈希资源加 14 个字体文件，其中 Font Awesome 4 个、Lato 6 个、Source Sans Pro 4 个。
- 内容不变则文件名不变，可安全使用一年强缓存。

### 6. public / rust/dist / dist 三者区别

| 路径 | 性质 | 来源 | 当前状态 |
| --- | --- | --- | --- |
| `public/` | 静态产物，同时是逐字节对比的冻结基准 | `nobelog build --self-assets --out public` | 42 个文件，2026-10-30 11:55 构建，`SITE_URL=https://example.com`，canonical 为 example.com，feed 的 `lastBuildDate` 为 2026-10-30 11:55 |
| `rust/dist/` | 开发构建产物 | `nobelog build` 默认输出 | 42 个文件，2026-11-01 07:23 的试验构建，CSS 哈希 86c19fcb，与 public 不一致，见第九章 |
| `dist/` | 发布包 | `tools/package.sh` | `nobelog-0.0.29` 与 `nobelog-0.0.30` 两个 tar.gz 加 `SHA256SUMS`，0.0.30 包内 110 个条目 |

`public/` 目录结构，42 个文件：

```text
public/
├── 404.html
├── index.html                        首页，1 页
├── about/index.html
├── archives/index.html
├── categories/index.html
├── categories/调试/index.html
├── categories/备忘/index.html
├── posts/typeset-test/index.html
├── posts/server-init-commands/index.html
├── tags/index.html
├── tags/Linux/index.html
├── tags/排版/index.html
├── tags/测试/index.html
├── tags/运维/index.html
├── feed.xml
├── sitemap.xml
├── robots.txt
├── assets-manifest.json
├── assets/                           16 个文件，2 个哈希资源加 14 个字体
└── images/                           8 张站内图片
```

### 7. 静态模式与动态模式对比

| 维度 | 静态模式，默认 | 动态模式，`--dynamic` |
| --- | --- | --- |
| 触发方式 | `nobelog serve` 不加参数 | `nobelog serve --dynamic` |
| 资源来源 | 磁盘 `public/assets` | `assets::build_in_memory` 常驻内存 |
| 页面来源 | 优先磁盘 `public/` 文件；无 `public/index.html` 时运行时直出 | `Site::page` 实时渲染 |
| 内容更新 | 需重新构建 | 改 `content/` 后最多 1 秒内热更新，无需重启 |
| 页面缓存头 | `PAGE_CACHE` | `NO_CACHE` |
| 资源缓存头 | `ASSET_CACHE` | `IMMUTABLE_CACHE` |
| 404 | 读 `public/404.html` | 运行时 `not_found_page`，带侧栏与面板 |
| 绝对地址 | `SITE_URL` 或 `SITE.site.url` 内置默认 | `SITE_URL` 或按请求 `Host` 与转发头推断 |
| 压缩与 Range | 有 | 有 |
| 当前 10086 实例 | 否 | 是 |

### 8. 缓存头常量

定义在 `rust/src/serve.rs:37-44`，与历史 TS 版逐字一致：

| 常量 | 值 | 使用场景 |
| --- | --- | --- |
| `ASSET_CACHE` | `public, max-age=31536000, s-maxage=15552000, stale-while-revalidate=2592000` | 静态模式带哈希资源 |
| `PAGE_CACHE` | `public, s-maxage=3600, stale-while-revalidate=86400` | 静态模式页面 |
| `NO_CACHE` | `no-cache` | 动态模式页面与清单 |
| `IMMUTABLE_CACHE` | `public, max-age=31536000, immutable` | 动态模式带哈希资源 |
| `NO_STORE` | `no-store` | 动态模式 404 与内容接口 |

实测响应头：

| 请求 | 关键头 | 实测值 |
| --- | --- | --- |
| `GET /` | cache-control、etag | `no-cache`、`"98da36edf19dd36f"` |
| `GET /assets/main.1ca5c955.css` | cache-control | `public, max-age=31536000, immutable`，无 ETag |
| `GET /images/test-cover.svg` | cache-control、etag | `no-cache`、`W/"307-1a14a44edb9"` |
| `GET /assets-manifest.json` | cache-control | `no-cache` |
| `GET /nope/` | 状态、cache-control | `404`、`no-cache` |
| 带 `Accept-Encoding: gzip` 的首页 | content-encoding | `gzip`，仅文本类且响应不小于 1024 字节才压缩 |
| 带 `Range: bytes=0-99` 的文章页 | 状态、content-range | `206`、`bytes 0-99/28907` |

压缩策略见 `rust/src/serve.rs:443-486`，`Vary: Accept-Encoding` 统一注入，见 `rust/src/serve.rs:497-515`。

### 9. 构建验证与本次实测

推荐验证命令：

```bash
cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test
cd .. && cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out rust/dist
./rust/target/release/nobelog diff public rust/dist
```

本次只读核对跑了对比命令，没有改代码，也没有重新构建：

```text
./rust/target/release/nobelog diff public rust/dist --normalize-assets
diff: 40 相同 / 1 不同 / 1 缺失 / 1 多出
  缺失  assets/main.1ca5c955.css
  多出  assets/main.86c19fcb.css
  不同  index.html，首个差异在第 114 行：12 天前 对 14 天前
```

结论：HTML 结构一致，唯一内容差异是首页卡片的相对日期文案随当前时间漂移；资源文件差异来自 `rust/dist` 的试验构建。正式对比时应先用与 `public/` 相同的 `SITE_URL=https://example.com` 重新构建，相对日期漂移属环境差异，不算回归。

---

## 六、侧栏 DOM 与样式分层

### 1. 侧栏 DOM 树

实测按 `curl` 拉取首页 HTML 得到，与模板 `rust/src/templates.rs:241-294` 一致：

```text
body
└── aside#sidebar.d-flex.flex-column.align-items-end   aria-label="Sidebar"
    ├── header.profile-wrapper
    │   ├── h1.site-title
    │   │   └── a[href="/"]                             站点标题，吱吱
    │   └── p.site-subtitle
    │       ├── 冷
    │       ├── span.brand-dot                          aria-hidden="true"
    │       ├── 知
    │       ├── span.brand-dot
    │       └── 识
    ├── nav.nav.flex-column.flex-grow-1.w-100.ps-0
    │   └── ul.nav
    │       └── li.nav-item × 5，当前页带 .active
    │           └── a.nav-link
    │               ├── i.fa-fw.fa-solid.fa-home       5 项图标不同
    │               └── span                             导航文字
    ├── div.sidebar-bottom.d-flex.flex-wrap.align-items-center.w-100
    │   ├── button#mode-toggle.mode-toggle.btn
    │   │   └── i.fas.fa-adjust
    │   ├── span.icon-border
    │   └── a.social-disabled × 4                      aria-disabled="true"
    │       └── i.fab.fa-github / fab.fa-x-twitter / fas.fa-envelope / fas.fa-rss
    └── 注释 .sidebar-bottom
```

关键实现：

- `sidebar` 外壳见 `rust/src/templates.rs:241-294`，头像块在 `SITE.avatar` 为空时不渲染，当前为空，所以没有头像。
- 副标题的字符间圆点由 `brand_subtitle` 生成，见 `rust/src/templates.rs:229-239`。
- 导航项数据来自 `SITE.nav`，见 `rust/src/site.rs:97-123`。
- 社交项数据来自 `SITE.social`，见 `rust/src/site.rs:75-96`；`href` 为空时输出 `social-disabled`，见 `rust/src/templates.rs:206-228`。
- 当前页在导航中的高亮类为 `li.nav-item.active`，判定见 `rust/src/templates.rs:170-183`。

### 2. 整页骨架 DOM

来自 `render_layout`，见 `rust/src/templates.rs:743-837`，与线上 HTML 完全一致：

```text
body
├── aside#sidebar                                    见上一节
├── div#main-wrapper.d-flex.justify-content-center
│   └── div.container.d-flex.flex-column.px-xxl-5
│       ├── header#topbar-wrapper
│       │   └── div#topbar.d-flex.align-items-center.justify-content-between.px-lg-3.h-100
│       │       ├── nav#breadcrumb                   aria-label="Breadcrumb"
│       │       │   └── span × N                      面包屑，分隔符为 CSS 生成
│       │       ├── button#sidebar-trigger.btn.btn-link
│       │       └── div#topbar-title
│       ├── div.row.flex-grow-1
│       │   ├── main[aria-label="Main Content"].col-12.col-lg-11.col-xl-9.px-md-4.pb-5
│       │   │   └── article.px-1
│       │   │       └── 各页面 body
│       │   └── aside#panel-wrapper.col-xl-3.ps-2.text-muted
│       │       ├── div.access
│       │       │   ├── section#access-lastmod.post
│       │       │   │   └── h2.panel-heading 最近更新 + ul.post-content ...
│       │       │   └── section
│       │       │       └── h2.panel-heading 热门标签 + div 下 a.post-tag
│       │       └── div.access-sticky
│       │           └── section#toc-wrapper.ps-0.pe-4.mb-5
│       │               └── nav#toc
│       │                   └── ul.toc-list
│       │                       └── li.toc-list-item
│       │                           └── a.toc-link.node-name--H2
│       ├── div.row
│       │   └── div#tail-wrapper.col-12.col-lg-11.col-xl-9.px-md-4
│       │       └── footer[aria-label="Site Info"]
│       ├── aside[aria-label="Scroll to Top"]
│       │   └── button#back-to-top.btn.btn-lg.btn-box-shadow
│       └── 注释 tail
└── div#mask
```

补充说明：

- 右侧面板由 `panel` 函数生成，见 `rust/src/templates.rs:431-457`；最近更新取前 5 篇，见 `rust/src/routes.rs:68-78`；热门标签按文章数取前 10，见 `rust/src/routes.rs:80-92`；目录只出现在文章页，见 `rust/src/templates.rs:532-546`。
- 目录结构为 `#toc-wrapper > nav#toc > ul.toc-list`，子级 `ul` 带 `is-collapsible`，条目带 `li.toc-list-item` 与 `a.toc-link.node-name--H2`，对应实现见 `rust/src/templates.rs:469-546`。
- 移动端顶栏的 `#sidebar-trigger` 与 `#mask` 配合 JS 控制 `body[sidebar-display]`，见 `rust/client/main.js:53-64` 与第五章 JS 区块表。
- 页脚内容由 `footer` 生成，见 `rust/src/templates.rs:356-384`，含年份、站点链接、许可文案、主题署名。
- 主栏列宽随右侧面板变化：面板存在时为 `col-xl-9`，为空时为 `col-xl-12`，判断见 `rust/src/templates.rs:766-770`。
- 回到顶部按钮固定挂在 `main-wrapper` 内，显示逻辑见 `rust/client/main.js:66-75`。

### 3. 样式分层：四个 CSS 文件的分工

| 文件 | 性质 | 负责范围 | 禁止事项 |
| --- | --- | --- | --- |
| `vendor/chirpy/main.css` | 上游官方编译产物 | 全部基础布局：栅格、侧栏 260px 定位、顶栏、卡片、分页、归档时间线、目录、页脚、Bootstrap 组件与 Font Awesome 图标、Chirpy 自带暗色媒体查询 | 不手改 |
| `vendor/fonts/fonts.css` | 自托管字体声明 | 14 个 `@font-face`，拉丁字体本地化，中文回退系统字体 | 不手改字体文件名 |
| `vendor/github-markdown/github-markdown.css` | 上游 GitHub 样式 | `.markdown-body` 正文：标题、段落、列表、表格、引用、代码块、图片，含自身 `prefers-color-scheme` 暗色变量 | 不手改 |
| `vendor/patches/patch.css` | 本站唯一手工层 | 内核特有元素、GitHub 正文接入、Chirpy 新旧版兼容、品牌块与和纸配色、移动端修正 | 只按注释分区追加 |

`main.css` 的实际内容：单行压缩，18 行文件 404272 字节，开头是 Bootstrap 5.3.2 版权块，中部是 Font Awesome 6.4.2 与 Chirpy 6.1.0 规则。侧栏基础定位来自其中 `#sidebar` 规则：`position: fixed; top: 0; left: 0; height: 100%; width: 260px; z-index: 99`。顶栏基础来自 `#topbar-wrapper { height: 3rem }`，目录基础来自 `#toc-wrapper { position: sticky; top: 4rem }`。

### 4. patch.css 分节

`vendor/patches/patch.css` 1244 行，按文件内注释分区，依次叠加：

| 行号 | 分区 | 内容 |
| --- | --- | --- |
| 1-20 | 文件头 | 说明只放三类内容：内核特有元素、构建/运行时修正、外来样式接入与兼容层 |
| 22-186 | GitHub Markdown 接入 | ① 正文字体 36-40；② 代码块与复制按钮 42-102；③ 显式暗色变量 104-162；④ 标题锚点 164-186 |
| 188-817 | compat 兼容层 | ① 调色板 199-301；② 版式 303-412；③ 组件细节 414-817 |
| 819-873 | 品牌块基础 | `--brand-line`、`--brand-hover` 变量与侧栏标题/副标题基础排版 |
| 875-942 | 和纸配色 | light 变量 878-905，dark 变量 907-933；页脚颜色与 `.markdown-body` 透明容器 935-942 |
| 944-1046 | 金字塔式两行标题 | 侧栏标题 1.6rem、副标题 1.75rem、`brand-dot` 圆点、目录/复选框/图钉细节 |
| 1028-1046 | 文字可选中 | 侧栏文字恢复可选中，站点标题与副标题保持不可选 |
| 1049-1120 | 移动端与正文细节 | ≤849 顶栏标题绝对居中 1049-1061；面包屑加深 1063-1073；图片浮层 1075-1103；提示块 1105-1110；代码块内边距 1112-1115；文章底部留白 1117-1120 |
| 1122-1204 | 交互细节 | 图片浮层缩放 1122-1139；手机端对齐 1142-1164；分类卡片与分类/标签整行点击 1166-1202 |
| 1204-1244 | 文章顶部一行 | `.post-meta-line` 分类与标签合并一行的间距、分隔线、暗色适配 |

覆盖规则：`patch.css` 在 `STYLE_ORDER` 里排最后，同特异度选择器后者胜，所以本站所有视觉调整都能压过上游。文件内也按行序覆盖，例如站点标题字号先在后段 819-855 设为 1.9rem，随后 953-958 改为 1.6rem 并生效。

### 5. 关键类名层级速查

| 层级 | 关键类名与 ID |
| --- | --- |
| 页面骨架 | `#main-wrapper`、`.container`、`.row`、`main`、`#panel-wrapper`、`#tail-wrapper` |
| 侧栏 | `#sidebar`、`.profile-wrapper`、`.site-title`、`.site-subtitle`、`.brand-dot`、`.nav`、`.nav-item`、`.nav-link`、`.active`、`.sidebar-bottom`、`#mode-toggle`、`.icon-border`、`.social-disabled` |
| 顶栏 | `#topbar-wrapper`、`#topbar`、`#breadcrumb`、`#sidebar-trigger`、`#topbar-title` |
| 主栏内容 | `article`、`#post-list`、`.card-wrapper`、`.card.post-preview`、`.card-body`、`.card-title`、`.card-text.content`、`.post-meta`、`.pin`、`.categories` |
| 文章页 | `h1[data-toc-skip]`、`.post-meta.text-muted`、`.readtime`、`.post-meta-line`、`.post-tags`、`.post-tag`、`.content.markdown-body`、`.post-tail-wrapper`、`.post-tail-bottom`、`.license-wrapper`、`.share-wrapper`、`.share-icons`、`#copy-link`、`#related-posts`、`.anchor` |
| 目录面板 | `.access`、`#access-lastmod`、`.panel-heading`、`.post-tag`、`.access-sticky`、`#toc-wrapper`、`#toc`、`.toc-list`、`.is-collapsible`、`.toc-list-item`、`.toc-link`、`.is-active-li`、`.is-active-link` |
| 列表页 | `#archives`、`.year`、`.date.day`、`.date.month`、`#tags`、`.tag`、`.categories`、`.card-header`、`#page-tag`、`#page-category`、`.dynamic-title`、`.dash` |
| 全站组件 | `#back-to-top`、`#mask`、`.lightbox`、`.prompt-tip`、`.prompt-info`、`.prompt-warning`、`.prompt-danger`、`.highlight`、`.copy-button`、`.anchor` |

---

## 七、深色模式与响应式断点

### 1. 深色模式三态

亮暗模式有 3 个状态，默认跟随系统：

| 状态 | `html` 属性 | 生效方式 |
| --- | --- | --- |
| 跟随系统 | 无 `data-mode` | CSS 的 `@media (prefers-color-scheme)` 接管 |
| 显式亮色 | `data-mode="light"` | 显式变量块生效 |
| 显式暗色 | `data-mode="dark"` | 显式变量块生效 |

关键实现：

- 状态存在 `localStorage` 的 `nobelog-mode` 键中，只认 `light` 与 `dark`，其余按跟随系统处理，见 `rust/client/main.js:14-27`。
- 首屏前脚本 `EARLY_SCRIPT` 内联在 `<head>`，只恢复显式选择，未选过不写属性，避免闪烁，见 `rust/src/templates.rs:741` 与 `796`。
- 点击切换循环为跟随系统 → 亮色 → 暗色 → 跟随系统，见 `rust/client/main.js:41-51`。
- `applyMode` 在跟随系统时移除属性，见 `rust/client/main.js:29-39`。
- 两个 `theme-color` meta 分别对应亮色 `#f7f7f7` 与暗色 `#1b1b1e`，见 `rust/src/templates.rs:783-784`。

CSS 侧有两条并行路径：

| 路径 | 位置 | 作用 |
| --- | --- | --- |
| 显式 `html[data-mode]` | `vendor/patches/patch.css:201-262`、`878-933` | 用户显式选择时覆盖调色板与和纸配色 |
| 系统媒体查询 | `vendor/patches/patch.css:264-301`、`1240-1244`；`vendor/github-markdown/github-markdown.css` 自身的 `@media`；`vendor/chirpy/main.css` 自带暗色媒体查询 | 未显式选择时跟随系统 |

改暗色样式时要同时考虑这两条路径，否则会出现「显式暗色正常、系统暗色缺样式」或相反的情况。

### 2. 断点 849 与 850

两个断点是互补的一对，分界在 850px：小于 850 为移动布局，大于等于 850 为桌面布局。

小于等于 849px 的行为，来源 `vendor/chirpy/main.css` 的 `@media all and (max-width: 849px)` 与 `vendor/patches/patch.css:318-322`、`337-341`、`509-514`、`1049-1061`：

- 侧栏收起为抽屉：`#sidebar` 默认 `transform: translateX(-260px)`，`body[sidebar-display]` 时回到 0；
- 主区与页脚在抽屉展开时右移 260px，`#back-to-top` 隐藏；
- 隐藏 `#breadcrumb` 与 `#search-wrapper`，显示 `#sidebar-trigger`、`#topbar-title`、`#search-trigger`；
- `main` 最小高度改为 `calc(100vh - 9rem)`；
- `footer` 高度 6rem，上下内边距 1.5rem；
- 顶栏标题绝对定位居中，最大宽度 60%；
- `h1.dynamic-title ~ .content` 上边距 2.5rem。

大于等于 850px 的行为，来源 `vendor/chirpy/main.css` 的 `@media all and (min-width: 850px)` 与 `vendor/patches/patch.css:306-310`、`349-353`、`532-537`：

- `html` 恢复纵向滚动条，`overflow-y: scroll`；
- 侧栏固定可见，宽 260px；`#main-wrapper` 与 `footer` 左边距 260px；
- 顶栏标题左对齐，面包屑显示；
- `main h1` 上边距 3rem；
- `#back-to-top` 位于右侧 5%、底部 3.625rem，由补丁覆盖上游位置。

### 3. 断点 1400

- 上游 `vendor/chirpy/main.css` 的 `@media all and (min-width: 1400px)` 只调整 `#back-to-top` 水平位置，公式含 260px 侧栏。
- 本站补丁 `vendor/patches/patch.css:539-543` 用同选择器覆盖为 `right: calc((100vw - 1400px) / 2 + 3rem)`，按 1400px 宽的居中内容区计算，让按钮贴近主栏右缘。
- 同级选择器后者胜，实际生效的是补丁里的公式。

### 4. 断点 1650

只有上游 `vendor/chirpy/main.css` 有 `@media all and (min-width: 1650px)`，本站补丁没有在同断点覆盖，所以 1650 以上全部沿用上游规则：

- 侧栏加宽为 300px，`#main-wrapper` 与 `footer` 左边距同步改为 300px；
- `#topbar-wrapper` 左偏移 300px；
- `#main` 与 `footer > .container` 最大宽度 1250px，左右内边距 1.75rem；
- `#core-wrapper`、`#tail-wrapper` 右内边距 4.5rem；
- 侧栏内边距整体放大，导航项左右内边距 2.75rem；
- `#back-to-top` 重新按 300px 侧栏与 1250px 主栏计算位置。

### 5. 其它断点

| 断点 | 行为 |
| --- | --- |
| 小于等于 1199px | `#panel-wrapper` 隐藏，右侧目录与最近更新面板不显示 |
| 大于等于 1200px | 面板出现，`col-xl-9` 栅格生效；`#back-to-top` 底部 6.5rem；普通内容区字号 1.03rem，见 `vendor/patches/patch.css:331-335` |
| 850px 到 1199px | 面包屑宽度 65%，超出省略 |
| 992px 到 1199px | `#main .col-lg-11` 宽度弹到 96% |
| 小于等于 830px | 分页只保留首页与末页按钮 |
| 小于 768px | 首页卡片改为纵向排列，图片在上 |
| 小于等于 600px | 图片浮层内边距缩小 |
| 小于等于 576px | 首页卡片、文章底部许可与分享按行靠右排列 |

### 6. 移动端侧栏交互

- 触发按钮 `#sidebar-trigger` 在 ≤849px 显示，点击后给 `body` 加 `sidebar-display` 属性，见 `rust/client/main.js:56-59`。
- 关闭方式：点遮罩 `#mask`、点侧栏内任意链接、按 Esc，见 `rust/client/main.js:60-64`。
- 打开状态由 CSS 的 `[sidebar-display]` 属性选择器接管，见 `vendor/chirpy/main.css` 的 849 断点规则。
- 桌面端无抽屉状态，侧栏常驻，见 850 断点规则。

---

## 八、维护约定

### 1. 可改与禁改

| 对象 | 可写性 | 说明 |
| --- | --- | --- |
| `content/posts/*.md`、`content/pages/*.md` | 正常编辑 | 文件名即兜底 slug，改后动态模式自动生效 |
| `content/images/` | 加图片 | 构建时复制到产物 `images/`，同名替换即时生效 |
| `rust/src/*.rs` | 可改 | 改完必须跑格式、静态检查、测试与 diff |
| `rust/client/main.js` | 可改 | 唯一浏览器脚本，零依赖，改完自动压缩进产物 |
| `rust/tools/*.rs` | 可改 | `css_prune.rs` 被主 crate 以 `#[path]` 引入 |
| `tools/make-icons.py`、`tools/package.sh` | 可改 | 图标生成与发布脚本 |
| `vendor/patches/patch.css` | 按分区追加 | 本站唯一手工样式层 |
| `vendor/chirpy/main.css`、`vendor/chirpy/*.woff2` | 禁止改 | 上游产物，重新抓取即丢失 |
| `vendor/github-markdown/github-markdown.css` | 禁止改 | 上游产物 |
| `public/` | 禁止手改 | 冻结基准与部署产物，改模板或补丁后重新构建 |
| `rust/dist/`、`dist/` | 禁止手改 | 构建与发布产物 |
| `.trellis/` | 按流程 | 任务、spec、journal 走 Trellis 流程 |

### 2. 新增路由与新增页面

新增可访问路由必须同时改三处，缺一不可，见 `.trellis/spec/frontend/routes-and-urls.md:28-34`：

1. `rust/src/routes.rs` 的 `Site::page` 加分支，位置放在通用单页分支之前；
2. `rust/src/routes.rs` 的 `collect_paths` 加路径，否则 sitemap 和静态构建都拿不到；
3. `rust/src/templates.rs` 新增一个 `pub fn xxx_page`，内部复用 `render_layout`。

改模板后必须跑 diff，确认差异只包含本次预期部分。

### 3. 提交规范

- 提交消息格式：`<type>: <中文说明> [task:<任务目录名>]`，type 用 `feat`、`fix`、`style`、`content`、`refactor`、`test`、`chore`。
- 任务锚点必填，全局 git 钩子会拦截；缺 `directive.md` 或用比 directive 更旧的文件冒充改动也会被拦。
- 不要用 `--no-verify` 绕过。
- 规则来源：`.trellis/spec/frontend/quality-guidelines.md:18-31`。

### 4. 验证命令

```bash
# 提交前质量门禁
cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test

# 构建静态产物，必须写 rust/dist，不要写仓库根 dist
cd .. && cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out rust/dist

# 与冻结基准逐字节对比，只允许预期差异
./rust/target/release/nobelog diff public rust/dist

# 本地动态模式预览，改 md 后最多 1 秒生效
cargo run --release --manifest-path rust/Cargo.toml -- serve --host 0.0.0.0 --port <port> --dynamic

# 本地静态模式预览
cargo run --release --manifest-path rust/Cargo.toml -- serve --host 0.0.0.0 --port <port>

# 发布打包，输出 dist/nobelog-<version>-linux-x86_64.tar.gz 与 SHA256SUMS
SITE_URL=https://<domain> VERSION=<x.y.z> tools/package.sh
```

约束与注意：

- 本机 cargo 不在默认 PATH，`tools/package.sh` 使用 `/root/.cargo/bin/cargo`；手工执行前先确认 cargo 可调用。
- 修改 `rust/src/serve.rs` 必须跑 `cargo test`，其中包含路径穿越、百分号解码等用例。
- 修改渲染输出必须跑 diff，对比前要设置与 `public/` 相同的 `SITE_URL`，当前冻结基准使用 `https://example.com`。
- 首页卡片的相对日期文案会随构建时间漂移，隔天对比出现的「N 天前」差异属环境差异，不算回归。
- 修改代码后线上不生效时，先确认运行的是 release 二进制，再执行 `cargo build --release` 并重启 systemd 单元。
- 重启服务需当轮明确授权，禁止未经授权执行 `systemctl restart nobelog-local`。
- 仓库内无 Node 生态，禁止为它引入 npm 依赖或打包器。

### 5. 内容发布流程

1. 在 `content/posts/` 新增 Markdown，写 frontmatter，`slug` 不填则用文件名；
2. 本地动态模式预览，或构建后检查产物；
3. 检查首页、文章页、归档、标签、分类、RSS、sitemap 是否出现新内容；
4. 按 Trellis 流程建任务、提交、写 journal。

### 6. 常见排障

| 现象 | 原因 | 处理 |
| --- | --- | --- |
| 样式上线后丢失 | CSS 裁剪误删动态拼接的 class | 设 `NOBELOG_CSS_PRUNE=0` 复测，确认后修 `css_prune` 保守规则 |
| 中文标签页 404 | 手拼 URL 漏了 `url::encode` | 改用 `templates::tag_url`、`category_url`、`post_url` |
| 新文章日期显示刚刚 | 日期按 UTC+8 挂钟比较，写了未来时间 | 检查 frontmatter 日期 |
| 草稿被发布 | 绕过 `content::load` 直接读文件 | 保持 `draft` 过滤在加载阶段 |
| 改了代码但线上没变 | 运行的是旧 release 二进制 | 重建并重启单元 |
| RSS 时间偏 8 小时 | 把 UTC+8 挂钟当真实 UTC 序列化 | 走 `rss_pub_date` 减 8 小时 |
| 目录或面板不显示 | 视口小于 1200px 时 `#panel-wrapper` 隐藏 | 属预期响应式行为 |
| 构建报 slug 或标签非法 | 安全校验拒绝非法字符 | 改内容命名，不要放宽校验 |

---

## 九、已知差异与核对结果

### 1. 本次核对通过项

| 核对项 | 结果 |
| --- | --- |
| 版本与运行态 | 二进制 0.0.30，systemd 单元 active，动态模式监听 10086 |
| 路由冒烟 | `/`、`/posts/typeset-test/`、`/posts/server-init-commands/`、`/archives/`、`/tags/`、`/categories/`、`/about/`、`/feed.xml`、`/sitemap.xml` 全部 200；`/page/2/`、`/nope/` 为 404 |
| 无尾斜杠规则 | `/posts/typeset-test`、`/about`、`/archives`、`/tags`、`/categories`、`/page/1` 全部 404 |
| 中文路径 | `/tags/排版/`、`/categories/备忘/` 与编码形式均 200 |
| sitemap 与 feed | sitemap 13 个 `url`，feed 2 个 `item` |
| 静态产物 | `public/` 42 个文件，14 个 HTML，含 13 个页面与根 404 |
| 资源 | `assets/` 16 个文件，manifest 两个哈希名与线上一致 |
| 线上资源一致性 | `curl` 拉取的 CSS 与 JS 与 `public/` 同名文件逐字节相同 |
| 缓存与压缩 | 页面 `no-cache`、资源 `immutable`、图片 `no-cache`；gzip 生效；Range 返回 206 |
| 断点行为 | 849/850、1400、1650 规则均已按上表核对源码与产物 |

### 2. 发现的文档漂移与不一致

| 编号 | 位置 | 现状 | 影响 |
| --- | --- | --- | --- |
| D1 | `rust/data/README.md` | 描述 `highlight-golden.json` 是冻结语料且命中率 80/80，但该文件不在仓库，git 历史也从未提交过；`rust/examples/render.rs:87-88` 只打印命中计数，不读该文件 | 文档过期，不影响构建与运行 |
| D2 | `.trellis/spec/frontend/styling-and-assets.md:62` | 写「main.js 功能固定为六块」，实际 `rust/client/main.js` 有 7 个编号区块，第 7 块为 249 行起的图片浮层 | 规范落后于实现，改 JS 时以实际文件为准 |
| D3 | `rust/README.md:8` | 写「默认产物写 `public/`」，与现状不符：`main.rs` 的 `build --out` 默认是 `dist`，规范要求写 `rust/dist/`，`public/` 只作冻结基准与发布产物 | 容易误把日常构建写进冻结基准 |
| D4 | `rust/dist/` | 当前是 2026-11-01 07:23 的试验构建：CSS 194196 字节、哈希 86c19fcb，与 `public/` 的 192311 字节、哈希 1ca5c955 不同 | 不要拿它当发布产物；日常对比前先重建 |
| D5 | 静态与动态 404 | 静态 `public/404.html` 主栏为 `col-xl-12` 无面板；动态 404 主栏为 `col-xl-9` 带侧栏面板 | 同一 URL 两种模式布局略有差异，属既有行为 |
| D6 | `Post.reading_minutes` | `content.rs` 仍计算该字段，模板层未使用；文章页展示的分钟区间由 `templates.rs` 另行计算 | 字段冗余，不产生错误 |
| D7 | 并行任务 `11-01-sidebar-clean-adapt` | 当前工作区存在未跟踪任务目录，正在做侧栏视觉适配，`vendor/patches/patch.css` 可能在本文写成后继续变化 | 本文第六、七节以 HEAD 70dd83a 为准，样式改动后需同步复核 |

### 3. 基线数字表

| 指标 | 当前值 |
| --- | --- |
| HEAD | `70dd83a`，2026-10-30 15:56:56 UTC |
| 分支 | `main`，跟踪 `origin/main` |
| 版本 | `0.0.30` |
| Rust 工具链 | 1.99.1，edition 2024 |
| 文章 / 单页 / 图片 | 2 / 1 / 8 |
| 标签 / 分类 | 4 / 2 |
| 路由数 | `collect_paths` 13 条，`all_routes` 15 条，含 feed 与 sitemap |
| HTML 产物 | 13 个页面加根 `404.html`，共 14 个 |
| `public/` 文件数 | 42 |
| `public/assets/` 文件数 | 16 |
| CSS 源合计 | 474484 字节，4 个文件 |
| CSS 产物 | 192311 字节，`main.1ca5c955.css` |
| JS 源 / 产物 | 16391 字节 / 12082 字节，`main.ea281e5e.js` |
| 在线实例 | 动态模式 `0.0.0.0:10086`，PID 443082 |
| 发布包 | `dist/nobelog-0.0.29-linux-x86_64.tar.gz`、`dist/nobelog-0.0.30-linux-x86_64.tar.gz` |

---

## 十、快速索引

| 想改什么 | 去哪里 |
| --- | --- |
| 写文章、改单页、换图片 | `content/` |
| 改页面结构或新增页面类型 | `rust/src/templates.rs` 与 `rust/src/routes.rs` |
| 改路由或 URL 规则 | `rust/src/routes.rs` 的 `Site::page` 与 `collect_paths` |
| 改视觉样式 | `vendor/patches/patch.css`，按注释分区追加 |
| 改上游主题或字体 | 不直接改 `vendor/` 上游文件，写补丁或提上游升级任务 |
| 改 CSS/JS 构建与哈希 | `rust/src/assets.rs`、`rust/src/paths.rs` |
| 改静态/动态服务行为 | `rust/src/serve.rs`，改完跑 `cargo test` |
| 改站点配置与导航 | `rust/src/site.rs` |
| 改 Markdown 渲染 | `rust/src/markdown.rs`，改完跑 diff |
| 改客户端交互 | `rust/client/main.js`，同步检查模板选择器 |
| 查权威规范 | `.trellis/spec/frontend/` 下的 8 篇专题 |
| 查历史决策 | `.trellis/tasks/archive/` 与 `.trellis/workspace/zhemed/journal-1.md` |

---

维护提示：本文以 2026-11-01 的 HEAD `70dd83a` 为准。改了路由、模板、资源管线、样式分层或断点后，请先跑第八章的验证命令，再回来同步本文对应的章节与行号。
