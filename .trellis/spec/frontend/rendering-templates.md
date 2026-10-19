# 渲染与模板

所有 HTML 都是 `rust/src/templates.rs` 里 `format!` 拼出来的字符串，没有模板引擎、没有框架。
路由层（`routes.rs`）只调用这些 `pub fn`，不自己拼 HTML。

## 对外契约

```rust
pub struct PageAssets { pub css: String, pub js: String }
pub struct Ctx<'a> {
    pub assets: &'a PageAssets,
    pub base_url: &'a str,   // 绝对地址根：SITE_URL 或请求推断值
    pub path: &'a str,       // 当前路径，用于 canonical / 侧栏高亮 / 顶部标题
    pub recent: &'a [Link],  // 侧栏"最近发布"
    pub top_tags: &'a [TagLink],
}

// 页面级函数（routes.rs 的入口）
home_page / list_page / post_page / archive_page / taxonomy_index / about_page
not_found_page / feed_xml / sitemap_xml
```

- 每个页面函数内部都调 `render_layout` 拼外壳；**新增页面类型 = 新增一个 `pub fn`，只产出 body 片段**，
  外壳复用 `render_layout`。
- `render_layout(title, description, body, crumbs, topbar_title, toc)` 负责 `<title>`、canonical、OG/Twitter、
  侧栏、顶部栏、页脚、右侧面板（`panel`）、回到顶部按钮、`#mask`。
- 右侧面板是否存在决定主栏栅格：`panel.is_empty()` 时用 `col-12 col-lg-11 col-xl-12`，否则 `col-xl-9`。

## 转义（最容易出事的一处）

```rust
fn escape_html(input: &str) -> String {
    // & < > " ' 依次替换为 &amp; &lt; &gt; &quot; &#39;
}
```

- 所有 frontmatter 值、标题、描述、路径进入 HTML 前必须过 `escape_html`；
  `render_layout` 已对 `page_title` / `description` / `ctx.path` 做了处理，页面函数里拼卡片时也要自己做。
- `<title>` 规则：`页面标题 - 吱吱`，首页无标题时只用站点名（见 `render_layout` 的 `page_title`）。
- canonical / `og:url` 都是 `{base_url}{path}`，`path` 已转义；**不要**在这里再拼 encode。

## 日期格式（三种，不能混用）

| 函数 | 输出 | 用途 |
| --- | --- | --- |
| `format_date` | `2026-10-17`（月日补零） | 卡片、归档、RSS、sitemap |
| `format_date_time` | `2024-2-22  周四  2:55 下午`（月日**不**补零，双空格分隔） | 主题 tooltip |
| `card_date` | `刚刚` / `N 小时前` / `昨天` / `N 天前` / 绝对日期（>30 天） | 首页卡片 |

`card_date` 的注意点：注释写明"站内日期按 UTC+8 挂钟存储"，比较时把 `Utc::now()` 加 8 小时，
否则刚发的文章会被判成未来、一律显示"刚刚"。未来时间直接退回绝对日期。

## 分页

- 每页 `SITE.posts_per_page`（当前 8）篇，路径 `/page/{n}/`，第一页是 `/`。
- `pagination(current, total, href_for)`：`total <= 1` 返回空串；窗口 `current±2`；
  首尾页在距离 >2 时显示，且当间隔恰好 1 时不加省略号（`current != 4`、`remaining != 3` 两个分支）。
- 首页第一页永远存在（没有文章时 `home_pages` 也会推一个空页，见 `Site::new`）。

## 侧栏高亮

`is_active(path, href)`：`/` 精确匹配（`/page/N/` 也算首页）；其余去掉尾斜杠后做前缀匹配，
所以 `/posts/x/` 会让"归档"高亮（`/archives/` 不同前缀则不会）。新增导航项时确认高亮语义。

## RSS / sitemap

- `feed_xml` / `sitemap_xml` 与页面共用 `base_url`，所以 `SITE_URL` 决定它们的绝对地址。
- sitemap 的路径清单来自 `Site::collect_paths()`；**新增可访问路由必须同时加进那个函数**。
- `main.rs diff` 会归一化 `<lastBuildDate>` 再比较，因为 RSS 每次构建都会变。

## 改模板的正确姿势

1. 先看 `public/` 里冻结产物的对应片段（用 `diff` 判断你的改动是否改变了字节）。
2. 改 `templates.rs` 后跑 `pnpm build`（会写 `rust/dist/`）再跑
   `./rust/target/release/nobelog diff public dist`；预期差异必须只包含你这次想改的页面。
3. 只动视觉、不动结构时优先改 `vendor/patches/patch.css`，不要为了加个 class 去改模板。

## 反模式

- 在 `format!` 里直接插入未转义的 frontmatter 值。
- 为了"更好看"加缩进/换行 → 破坏与冻结产物的逐字节对比，且会让整站 diff 噪音化。
- 在页面函数里自己拼 `<html>`/`<head>`；外壳只由 `render_layout` 产出。
- 把 `base_url` 写死成 `SITE.url`；静态构建时 `SITE_URL` 会失效。
