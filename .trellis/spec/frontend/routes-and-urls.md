# 路由与 URL

路由层是 `rust/src/routes.rs`：`Site::page(path)` 是纯函数（给定路径产出完整 HTML），
同一套路由同时服务静态构建与动态模式。

## 路由表

| 路径 | 状态 | 内容 |
| --- | --- | --- |
| `/` | 200 | 首页第一页 |
| `/page/{n}/` | 200 / 404 | 第 n 页（越界 404） |
| `/posts/{slug}/` | 200 / 404 | 文章页 |
| `/archives/` | 200 | 归档（按年分组） |
| `/tags/` · `/categories/` | 200 | 标签/分类总览 |
| `/tags/{name}/` · `/categories/{name}/` | 200 / 404 | 标签/分类详情 |
| `/{page-slug}/` | 200 | 单页（slug 仅字母数字与 `-`，如 `/about/`） |
| `/feed.xml` | 200 | RSS |
| `/sitemap.xml` | 200 | 站点地图 |
| 其它 | 404 | `404.html` |

规则：

- 所有页面路由**带尾斜杠**；`/posts/x`（无斜杠）走 404，不做 301。
- 首页分页路径由 `home_pages` 决定，`SITE.posts_per_page`（8）控制每页篇数。
- 单页 slug 只匹配 `[A-Za-z0-9-]+`，避免和 `/posts/`、`/tags/` 等前缀抢路由。
- 未知路径返回 `status: 404` + `templates::not_found_page`；静态构建把 200 路由写成文件，另外写根 `404.html`。

## 新增路由的清单

1. `routes.rs::Site::page` 加分支（放在通用单页分支之前，否则会被单页规则吞掉）。
2. `routes.rs::collect_paths()` 加路径 —— 否则 sitemap 与静态构建都拿不到它
   （静态构建走 `all_routes()` → `collect_paths()`）。
3. `templates.rs` 加一个 `pub fn xxx_page`，内部复用 `render_layout`。
4. 如果页面来自内容层，确认 frontmatter 字段与 `content.rs` 解析同时到位。

## URL 编码（必须与 JS 对齐）

`rust/src/url.rs` 刻意复刻 `encodeURIComponent`：`NON_ALPHANUMERIC` 保留
`- _ . ! ~ * ' ( )` 不转义。解码用 `percent_decode_str(..).decode_utf8_lossy()`。

- 文章/标签/分类 URL 一律通过 `templates::post_url` / `tag_url` / `category_url`。
- 路由匹配前先 `url::decode` 再查表（`/tags/%E6%8E%92%E7%89%88/` → 标签"排版"）。
- 产物落盘时 `build.rs::file_for_route` 再次解码，目录名用真实中文名（`public/tags/排版/index.html`）。
- 不要改用别的 percent-encoding 集合：`+`、`(`、`'` 的处理差异会让静态文件与请求路径不一致。

## 站点绝对地址（`SITE_URL`）

`site.rs::env_url()` 读取环境变量 `SITE_URL`，`trim()` 后去掉尾部 `/`，空值视为未设置。

优先级（由高到低）：

1. 环境变量 `SITE_URL`（静态构建、动态模式都生效）。
2. 动态模式的请求推断：`request_base_url(headers, fallback)`，按 `Host` / `X-Forwarded-Proto` 拼出
   `scheme://host`（见 `serve.rs`）。
3. `SITE.url` 的内置默认值（见 `rust/src/site.rs`；生产必须用 `SITE_URL` 覆盖，不要依赖它）。

影响的输出：canonical、`og:url`、RSS 链接、sitemap、页脚与分享链接。
**新增绝对链接时必须使用 `ctx.base_url`（模板层）或 `Site::root_url`（路由层），不要写死 `SITE.url`。**

## 站点配置（`site.rs`）

`static SITE: SiteConfig` 是编译期常量：`url` / `title` / `subtitle` / `lang` / `feed_lang` / `description` /
`author` / `avatar` / `email` / `github` / `icp` / `social` / `nav` / `posts_per_page` / `license_*`。

- 导航项 `nav` 的 `href` 决定侧栏高亮（`is_active` 前缀匹配），改 `href` 要一起看高亮行为。
- 空字符串字段（如 `github`）在模板里会禁用对应链接（`patch.css` 里有"社交图标占位"规则）。
- 这些值会被模板**原样输出**，改文案即改全站；不要在这里放未转义的 HTML。

## 反模式

- 手拼 URL：`format!("/tags/{}", name)`（漏编码）、`format!("/posts/{}/", slug)`（漏 encode）。
- 新路由只加在 `page()` 忘记 `collect_paths()` → 线上 sitemap 缺页、静态构建不产出该文件。
- 在模板里用 `SITE.url` 拼绝对地址 → 部署设置 `SITE_URL` 后链接仍指向本地。
- 给页面路径加上/去掉尾斜杠做"兼容" → 静态模式依赖尾斜杠映射到 `index.html`，破坏一致性。
