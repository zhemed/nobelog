// 模板层：逐字节复刻 src/templates.ts 的输出。接口由 Lead 冻结，实现方按签名填充。
use crate::model::{Post, Taxonomy, TocEntry};
use crate::site::SITE;
use crate::url;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::cmp::Reverse;

#[derive(Debug, Clone)]
pub struct PageAssets {
    pub css: String,
    pub js: String,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // count 与 TS 面板数据同构
pub struct TagLink {
    pub name: String,
    pub url: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct RelatedLink {
    pub title: String,
    pub url: String,
    pub date: DateTime<Utc>,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub name: String,
    pub href: Option<String>,
}

pub struct Ctx<'a> {
    pub assets: &'a PageAssets,
    pub path: &'a str,
    pub recent: &'a [Link],
    pub top_tags: &'a [TagLink],
}

/* ------------------------------------------------------------ 基础工具 */

/// 对齐 src/templates.ts 的 escapeHtml：& < > " ' 按序替换。
fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn pad2(value: u32) -> String {
    format!("{value:02}")
}

/// 2026-02-11（构建机 TZ=UTC，Node 的本地时间与 UTC 一致）
fn format_date(date: &DateTime<Utc>) -> String {
    format!(
        "{}-{}-{}",
        date.year(),
        pad2(date.month()),
        pad2(date.day())
    )
}

const WEEKDAYS: [&str; 7] = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

/// 主题 tooltip 里的日期形态：2024-2-22  周四  2:55 下午
fn format_date_time(date: &DateTime<Utc>) -> String {
    let hours = date.hour();
    let ampm = if hours < 12 { "上午" } else { "下午" };
    let hour12 = if hours.is_multiple_of(12) {
        12
    } else {
        hours % 12
    };
    let weekday = WEEKDAYS[date.weekday().num_days_from_sunday() as usize];
    format!(
        "{}-{}-{}  {}  {}:{} {}",
        date.year(),
        date.month(),
        date.day(),
        weekday,
        hour12,
        pad2(date.minute()),
        ampm
    )
}

/// 卡片日期：超过 30 天给绝对日期，否则给相对说法。JS 用 Math.floor，这里用 div_euclid 保持负数语义。
fn card_date(date: &DateTime<Utc>) -> String {
    // 站内日期按 UTC+8 挂钟存储（frontmatter 写的是本地时间），而服务器可能是 UTC，
    // 所以“现在”也要换算到同一挂钟再比较，否则刚发的文章会被判成未来、一律显示“刚刚”。
    let now = Utc::now() + chrono::Duration::hours(8);
    let diff = now.signed_duration_since(*date);
    if diff.num_minutes() < 0 {
        return format_date(date);
    }
    if diff.num_minutes() < 60 {
        return "刚刚".to_string();
    }
    if diff.num_hours() < 24 {
        return format!("{} 小时前", diff.num_hours());
    }
    let days = diff.num_days();
    if days == 1 {
        return "昨天".to_string();
    }
    if days <= 30 {
        return format!("{days} 天前");
    }
    format_date(date)
}

pub fn post_url(post: &Post) -> String {
    format!("/posts/{}/", url::encode(&post.slug))
}

pub fn tag_url(name: &str) -> String {
    format!("/tags/{}/", url::encode(name))
}

pub fn category_url(name: &str) -> String {
    format!("/categories/{}/", url::encode(name))
}

fn icon(class: &str) -> String {
    format!("<i class=\"{class}\"></i>")
}

fn icon_extra(class: &str, extra: &str) -> String {
    format!("<i class=\"{class} {extra}\"></i>")
}

/* ------------------------------------------------------------ 侧边栏 */

fn is_home_page_path(path: &str) -> bool {
    path.strip_prefix("/page/")
        .and_then(|rest| rest.strip_suffix('/'))
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

fn is_active(path: &str, href: &str) -> bool {
    if href == "/" {
        return path == "/" || is_home_page_path(path);
    }
    let base = href.strip_suffix('/').unwrap_or(href);
    path == href || path == base || path.starts_with(&format!("{base}/"))
}

fn nav_list(path: &str) -> String {
    SITE.nav
        .iter()
        .map(|item| {
            let active = is_active(path, item.href);
            let active_class = if active { " active" } else { "" };
            let name = escape_html(item.name);
            format!(
                r##"      <li class="nav-item{active_class}">
        <a class="nav-link" href="{href}">
          <i class="{icon}"></i>
          <span>{name}</span>
        </a>
      </li>"##,
                href = item.href,
                icon = item.icon
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn social_list() -> String {
    SITE
        .social
        .iter()
        .map(|item| {
            let name = escape_html(item.name);
            if item.href.is_empty() {
                format!(
                    r##"    <a href="#" class="social-disabled" aria-label="{name}" title="{name}" aria-disabled="true"><i class="{icon}"></i></a>"##,
                    icon = item.icon
                )
            } else {
                format!(
                    r##"    <a href="{href}" aria-label="{name}" title="{name}"><i class="{icon}"></i></a>"##,
                    href = item.href,
                    icon = item.icon
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn brand_subtitle(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return escape_html(text);
    }
    chars
        .iter()
        .map(|c| escape_html(&c.to_string()))
        .collect::<Vec<_>>()
        .join(r#"<span class="brand-dot" aria-hidden="true"></span>"#)
}

fn sidebar(path: &str) -> String {
    let avatar_block = if SITE.avatar.is_empty() {
        String::new()
    } else {
        format!(
            r##"    <a class="rounded-circle" href="/" id="avatar">
      <img src="{avatar}" width="112" height="112" alt="avatar" decoding="async">
    </a>"##,
            avatar = SITE.avatar
        )
    };
    let title = escape_html(SITE.title);
    let subtitle_block = if SITE.subtitle.is_empty() {
        String::new()
    } else {
        format!(
            r##"    <p class="site-subtitle">
      {}
    </p>"##,
            brand_subtitle(SITE.subtitle)
        )
    };
    let nav = nav_list(path);
    let social = social_list();
    let social_block = if social.is_empty() {
        String::new()
    } else {
        format!("    <span class=\"icon-border\"></span>\n{social}")
    };
    let adjust = icon("fas fa-adjust");
    format!(
        r##"<aside aria-label="Sidebar" id="sidebar" class="d-flex flex-column align-items-end">
  <header class="profile-wrapper">
{avatar_block}
    <h1 class="site-title">
      <a href="/">{title}</a>
    </h1>
{subtitle_block}
  </header>
  <!-- .profile-wrapper -->
  <nav class="nav flex-column flex-grow-1 w-100 ps-0">
    <ul class="nav">
{nav}
    </ul>
  </nav>
  <!-- ul.nav.flex-column -->
  <div class="sidebar-bottom d-flex flex-wrap align-items-center w-100">
    <button class="mode-toggle btn" id="mode-toggle" type="button" aria-label="Switch Mode" title="切换亮暗模式">{adjust}</button>
{social_block}
  </div>
  <!-- .sidebar-bottom -->
</aside>"##
    )
}

/* ------------------------------------------------------------ 顶栏 */

fn breadcrumb(items: &[Breadcrumb]) -> String {
    let last = items.len().saturating_sub(1);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let name = escape_html(&item.name);
            match &item.href {
                Some(href) if index != last => format!(
                    r##"    <span>
      <a href="{href}">{name}</a>
    </span>"##
                ),
                _ => format!("    <span>{name}</span>"),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn topbar_title_for(path: &str) -> String {
    if path == "/" || is_home_page_path(path) {
        return SITE.title.to_string();
    }
    if path.starts_with("/archives/") {
        return "归档".to_string();
    }
    if path.starts_with("/tags/") {
        return "标签".to_string();
    }
    if path.starts_with("/categories/") {
        return "分类".to_string();
    }
    if path.starts_with("/posts/") {
        return "文章".to_string();
    }
    String::new()
}

fn topbar(crumbs: &str, title: &str) -> String {
    let title = escape_html(title);
    format!(
        r##"    <header id="topbar-wrapper">
      <div id="topbar" class="d-flex align-items-center justify-content-between px-lg-3 h-100">
        <nav id="breadcrumb" aria-label="Breadcrumb">
{crumbs}
        </nav>
        <button type="button" id="sidebar-trigger" class="btn btn-link" aria-label="打开侧栏">
          <i class="fas fa-bars fa-fw"></i>
        </button>
        <div id="topbar-title">{title}</div>
      </div>
    </header>"##
    )
}

/* ------------------------------------------------------------ 页脚 */

fn footer() -> String {
    let year = Utc::now().year();
    let title = escape_html(SITE.title);
    let note = escape_html(SITE.license_note);
    let reserved = escape_html("保留部分权利。");
    let powered = concat!(
        "主题：",
        r#"<a href="https://github.com/cotes2020/jekyll-theme-chirpy" target="_blank" rel="noopener">Chirpy</a>"#,
        " · 作者 Cotes Chung"
    );
    format!(
        r##"        <footer aria-label="Site Info" class="d-flex flex-column justify-content-center text-muted flex-lg-row justify-content-lg-between align-items-lg-center pb-lg-3">
          <p>
            ©{year}
            <a href="{url}">{title}</a>.
            <span data-bs-toggle="tooltip" data-bs-placement="top" title="{note}">
              {reserved}
            </span>
          </p>
          <p>
            {powered}
          </p>
        </footer>"##,
        url = SITE.url
    )
}

/* ------------------------------------------------------------ 右侧面板 */

fn panel_post_list(links: &[Link]) -> String {
    let items = links
        .iter()
        .map(|link| {
            format!(
                r##"          <li class="text-truncate lh-lg">
            <a href="{url}">{title}</a>
          </li>"##,
                url = link.url,
                title = escape_html(&link.title)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r##"      <section id="access-lastmod" class="post">
        <h2 class="panel-heading">最近更新</h2>
        <ul class="post-content list-unstyled ps-0 pb-1 ms-1 mt-2">
{items}
        </ul>
      </section>"##
    )
}

fn panel_tag_list(tags: &[TagLink]) -> String {
    let items = tags
        .iter()
        .map(|tag| {
            format!(
                r##"          <a class="post-tag btn btn-outline-primary" href="{url}">{name}</a>"##,
                url = tag.url,
                name = escape_html(&tag.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r##"      <section>
        <h2 class="panel-heading">热门标签</h2>
        <div class="d-flex flex-wrap mt-3 mb-1 me-3">
{items}
        </div>
      </section>"##
    )
}

fn panel(ctx: &Ctx, toc: &str) -> String {
    let mut blocks: Vec<String> = Vec::new();
    if !ctx.recent.is_empty() {
        blocks.push(panel_post_list(ctx.recent));
    }
    if !ctx.top_tags.is_empty() {
        blocks.push(panel_tag_list(ctx.top_tags));
    }
    if toc.trim().is_empty() && blocks.is_empty() {
        return String::new();
    }
    let access = if blocks.is_empty() {
        String::new()
    } else {
        format!(
            "\n    <div class=\"access\">\n{}\n    </div>",
            blocks.join("\n")
        )
    };
    format!(
        r##"      <aside aria-label="Panel" id="panel-wrapper" class="col-xl-3 ps-2 text-muted">{access}
    <div class="access-sticky">
{toc}
    </div>
  </aside>"##
    )
}

/* ------------------------------------------------------------ 目录 */

struct TocNode {
    level: usize,
    id: String,
    text: String,
    children: Vec<usize>,
}

/// 与 TS 的 tocTree 完全一致：用栈切分父子层级（arena + 索引）。
fn toc_tree(entries: &[TocEntry]) -> (Vec<TocNode>, Vec<usize>) {
    let mut nodes: Vec<TocNode> = Vec::with_capacity(entries.len());
    let mut roots: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    for entry in entries {
        let index = nodes.len();
        nodes.push(TocNode {
            level: entry.level,
            id: entry.id.clone(),
            text: entry.text.clone(),
            children: Vec::new(),
        });
        while let Some(&top) = stack.last() {
            if nodes[top].level >= entry.level {
                stack.pop();
            } else {
                break;
            }
        }
        match stack.last() {
            Some(&parent) => nodes[parent].children.push(index),
            None => roots.push(index),
        }
        stack.push(index);
    }
    (nodes, roots)
}

fn toc_list(nodes: &[TocNode], ids: &[usize], parent_level: usize, indent: &str) -> String {
    if ids.is_empty() {
        return String::new();
    }
    let inner = format!("{indent}  ");
    let class = if parent_level == 0 {
        "toc-list "
    } else {
        "toc-list is-collapsible"
    };
    let items = ids
        .iter()
        .map(|&index| {
            let node = &nodes[index];
            let id = escape_html(&node.id);
            let text = escape_html(&node.text);
            let children = toc_list(nodes, &node.children, node.level, &inner);
            format!(
                r##"{inner}<li class="toc-list-item">
{inner}  <a href="#{id}" class="toc-link node-name--H{level} ">{text}</a>
{children}{inner}</li>"##,
                level = node.level
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r##"{indent}<ul class="{class}">
{items}
{indent}</ul>
"##
    )
}

/// 文章页右侧的目录区块（无标题时返回空串，整个区块不渲染）
fn toc_wrapper(entries: &[TocEntry]) -> String {
    let (nodes, roots) = toc_tree(entries);
    if roots.is_empty() {
        return String::new();
    }
    let list = toc_list(&nodes, &roots, 0, "        ");
    format!(
        r##"      <section id="toc-wrapper" class="ps-0 pe-4 mb-5">
        <h2 class="panel-heading ps-3 pt-2 mb-2">目录</h2>
        <nav id="toc" data-toc-start-level="2" data-toc-end-level="4" data-toc-collapse-depth="6">
{list}        </nav>
      </section>
"##
    )
}

/* ------------------------------------------------------------ 文章卡片 */

fn post_meta(post: &Post) -> String {
    let categories = if post.categories.is_empty() {
        String::new()
    } else {
        let names = post
            .categories
            .iter()
            .map(|name| format!("            {}", escape_html(name)))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            r##"          <i class="far fa-folder-open fa-fw me-1"></i>
          <span class="categories">
{names}
          </span>"##
        )
    };
    let pin = if post.pinned {
        r##"
        <div class="pin ms-1">
          <i class="fas fa-thumbtack fa-fw"></i>
        </div>"##
            .to_string()
    } else {
        String::new()
    };
    let date = card_date(&post.date);
    format!(
        r##"        <div class="post-meta flex-grow-1 d-flex align-items-end">
          <div class="me-auto">
            <!-- posted date -->
            <i class="far fa-calendar fa-fw me-1"></i>
            <em class="">{date}</em>
{categories}
          </div>{pin}
        </div>
        <!-- .post-meta -->"##
    )
}

fn post_card(post: &Post) -> String {
    let cover = match &post.cover {
        Some(cover) => format!(
            r##"        <div class="preview-img">
          <img src="{src}" alt="{title}" loading="lazy" decoding="async">
        </div>
"##,
            src = escape_html(cover),
            title = escape_html(&post.title)
        ),
        None => String::new(),
    };
    let url = post_url(post);
    let title = escape_html(&post.title);
    let summary = escape_html(&post.summary);
    let meta = post_meta(post);
    format!(
        r##"      <article class="card-wrapper">
        <a href="{url}" class="card post-preview flex-md-row-reverse">
{cover}        <div class="card-body d-flex flex-column">
          <h1 class="card-title my-2 mt-md-0">{title}</h1>
          <div class="card-text content mt-0 mb-2">
            <p>{summary}</p>
          </div>
{meta}
        </div>
      </a>
      </article>"##
    )
}

fn post_list(posts: &[&Post]) -> String {
    if posts.is_empty() {
        return r##"    <div id="post-list" class="flex-grow-1 pe-xl-2">
      <p class="text-muted">这里还没有文章。</p>
    </div>"##
            .to_string();
    }
    let cards = posts
        .iter()
        .map(|post| post_card(post))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r##"    <div id="post-list" class="flex-grow-1 pe-xl-2">
{cards}
    </div>"##
    )
}

/* ------------------------------------------------------------ 分页器 */

fn pagination<F: Fn(usize) -> String>(current: usize, total: usize, href_for: F) -> String {
    if total <= 1 {
        return String::new();
    }
    let mut items: Vec<String> = Vec::new();

    let prev_href = if current > 1 {
        href_for(current - 1)
    } else {
        "#".to_string()
    };
    items.push(format!(
        r##"        <li class="page-item{disabled}">
          <a aria-label="previous-page" class="page-link" href="{prev_href}">
            <i class="fas fa-angle-left"></i>
          </a>
        </li>"##,
        disabled = if current > 1 { "" } else { " disabled" }
    ));

    if current > 3 {
        items.push(format!(
            r##"        <li class="page-item">
          <a class="page-link" href="{href}">1</a>
        </li>"##,
            href = href_for(1)
        ));
        if current != 4 {
            items.push(
                r##"        <li class="page-item disabled">
          <span class="page-link">…</span>
        </li>"##
                    .to_string(),
            );
        }
    }

    let mut page = current as i64 - 2;
    let last = current as i64 + 2;
    while page <= last {
        if page >= 1 && page <= total as i64 {
            let page = page as usize;
            items.push(format!(
                r##"        <li class="page-item{active}">
          <a class="page-link" href="{href}">{page}</a>
        </li>"##,
                active = if page == current { " active" } else { "" },
                href = href_for(page)
            ));
        }
        page += 1;
    }

    let remaining = total as i64 - current as i64;
    if remaining > 2 {
        if remaining != 3 {
            items.push(
                r##"        <li class="page-item disabled">
          <span class="page-link">…</span>
        </li>"##
                    .to_string(),
            );
        }
        items.push(format!(
            r##"        <li class="page-item{active}">
          <a class="page-link" href="{href}">{total}</a>
        </li>"##,
            active = if total == current { " active" } else { "" },
            href = href_for(total)
        ));
    }

    let next_href = if current < total {
        href_for(current + 1)
    } else {
        "#".to_string()
    };
    items.push(format!(
        r##"        <li class="page-item{disabled}">
          <a aria-label="next-page" class="page-link" href="{next_href}">
            <i class="fas fa-angle-right"></i>
          </a>
        </li>"##,
        disabled = if current < total { "" } else { " disabled" }
    ));

    format!(
        r##"    <nav aria-label="Page Navigation">
      <ul class="pagination align-items-center mt-4 mb-0">
{items}
      </ul>
    </nav>"##,
        items = items.join("\n")
    )
}

/* ------------------------------------------------------------ 外壳 */

/// 首屏前恢复亮暗模式；单独成常量，避免在 format 模板里处理花括号。
const EARLY_SCRIPT: &str = r#"  <script>/* 首屏前恢复亮暗模式（只认用户显式选择；未选过就不写 data-mode，交给 CSS 的 prefers-color-scheme 跟随系统） */try{var m=localStorage.getItem("nobelog-mode");if(m==="dark"||m==="light")document.documentElement.setAttribute("data-mode",m)}catch(e){}</script>"#;

fn render_layout(
    ctx: &Ctx,
    title: Option<&str>,
    description: &str,
    body: &str,
    crumbs: &str,
    topbar_title: Option<&str>,
    toc: &str,
) -> String {
    let page_title = match title {
        Some(title) => format!("{title} - {}", SITE.title),
        None => SITE.title.to_string(),
    };
    let page_title = escape_html(&page_title);
    let description = escape_html(description);
    let topbar_title = topbar_title
        .map(str::to_string)
        .unwrap_or_else(|| topbar_title_for(ctx.path));
    let path = escape_html(ctx.path);
    let sidebar = sidebar(ctx.path);
    let topbar = topbar(crumbs, &topbar_title);
    let footer = footer();
    let panel = panel(ctx, toc);
    let main_col = if panel.is_empty() {
        "col-12 col-lg-11 col-xl-12"
    } else {
        "col-12 col-lg-11 col-xl-9"
    };
    let site_title = escape_html(SITE.title);
    let css = &ctx.assets.css;
    let js = &ctx.assets.js;
    let lang = SITE.lang;
    let url = SITE.url;
    format!(
        r##"<!DOCTYPE html>
<html lang="{lang}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1, shrink-to-fit=no, viewport-fit=cover">
  <meta name="theme-color" content="#f7f7f7" media="(prefers-color-scheme: light)">
  <meta name="theme-color" content="#1b1b1e" media="(prefers-color-scheme: dark)">
  <title>{page_title}</title>
  <meta name="description" content="{description}">
  <meta name="generator" content="nobelog">
  <link rel="canonical" href="{url}{path}">
  <link rel="alternate" type="application/rss+xml" title="{site_title}" href="/feed.xml">
{EARLY_SCRIPT}
  <link rel="stylesheet" href="{css}">
  <meta property="og:title" content="{page_title}">
  <meta property="og:description" content="{description}">
  <meta property="og:type" content="website">
  <meta property="og:url" content="{url}{path}">
  <meta name="twitter:card" content="summary">
  <script src="{js}" type="module"></script>
</head>
<body>
{sidebar}
<div id="main-wrapper" class="d-flex justify-content-center">
  <div class="container d-flex flex-column px-xxl-5">
{topbar}
    <div class="row flex-grow-1">
      <main aria-label="Main Content" class="{main_col} px-md-4 pb-5">
        <article class="px-1">
{body}
        </article>
      </main>
{panel}
    </div>
    <div class="row">
      <!-- tail -->
      <div id="tail-wrapper" class="{main_col} px-md-4">
{footer}
      </div>
    </div>
  </div>
  <aside aria-label="Scroll to Top">
    <button id="back-to-top" type="button" class="btn btn-lg btn-box-shadow">
      <i class="fas fa-angle-up"></i>
    </button>
  </aside>
</div>
<div id="mask"></div>
</body>
</html>
"##
    )
}

/* ------------------------------------------------------------ 面包屑默认值 */

fn crumbs_home(name: &str) -> Vec<Breadcrumb> {
    vec![Breadcrumb {
        name: name.to_string(),
        href: Some("/".to_string()),
    }]
}

/* ------------------------------------------------------------ 面板数据兜底 */

fn panel_post_links(posts: &[&Post], limit: usize) -> Vec<Link> {
    let mut sorted = posts.to_vec();
    sorted.sort_by_key(|post| Reverse(post.date));
    sorted.truncate(limit);
    sorted
        .iter()
        .map(|post| Link {
            title: post.title.clone(),
            url: post_url(post),
        })
        .collect()
}

fn panel_tag_links(map: &Taxonomy, limit: usize) -> Vec<TagLink> {
    let mut sorted: Vec<(&String, &Vec<usize>)> = map.iter().collect();
    sorted.sort_by_key(|entry| Reverse(entry.1.len()));
    sorted.truncate(limit);
    sorted
        .into_iter()
        .map(|(name, list)| TagLink {
            name: name.clone(),
            url: tag_url(name),
            count: list.len(),
        })
        .collect()
}

/* ------------------------------------------------------------ 各页面 */

pub fn home_page(ctx: &Ctx, posts: &[&Post], page: usize, total: usize) -> String {
    let fallback = if ctx.recent.is_empty() && page == 1 {
        panel_post_links(posts, 5)
    } else {
        Vec::new()
    };
    let recent = if fallback.is_empty() {
        ctx.recent
    } else {
        &fallback
    };
    let ctx = Ctx {
        assets: ctx.assets,
        path: ctx.path,
        recent,
        top_tags: ctx.top_tags,
    };

    let body = format!(
        "{}\n{}",
        post_list(posts),
        pagination(page, total, |p| if p == 1 {
            "/".to_string()
        } else {
            format!("/page/{p}/")
        })
    );
    let title = if page == 1 {
        None
    } else {
        Some(format!("第 {page} 页"))
    };
    let mut crumbs = crumbs_home("主页");
    if page != 1 {
        crumbs.push(Breadcrumb {
            name: format!("第 {page} 页"),
            href: None,
        });
    }
    render_layout(
        &ctx,
        title.as_deref(),
        SITE.description,
        &body,
        &breadcrumb(&crumbs),
        None,
        "",
    )
}

pub fn list_page(ctx: &Ctx, title: &str, posts: &[&Post], crumbs: &[Breadcrumb]) -> String {
    let is_category = ctx.path.starts_with("/categories/");
    let name = {
        let raw = ctx.path.split('/').nth(2).unwrap_or("");
        let decoded = url::decode(raw);
        if !decoded.is_empty() {
            decoded
        } else {
            let cut = title
                .char_indices()
                .find(|(_, ch)| *ch == '：' || *ch == ':')
                .map(|(index, ch)| index + ch.len_utf8());
            match cut {
                Some(index) => title[index..].to_string(),
                None => title.to_string(),
            }
        }
    };
    let heading = if is_category { "分类" } else { "标签" };
    let icon_name = if is_category {
        icon_extra("far fa-folder-open fa-fw", "text-muted")
    } else {
        icon_extra("fa fa-tag fa-fw", "text-muted")
    };

    let items = posts
        .iter()
        .map(|post| {
            let date = if is_category {
                format!(
                    "{}年{}月{}日",
                    post.date.year(),
                    pad2(post.date.month()),
                    post.date.day()
                )
            } else {
                format!(
                    "{}-{}-{}",
                    post.date.year(),
                    post.date.month(),
                    post.date.day()
                )
            };
            format!(
                r##"      <li class="d-flex justify-content-between px-md-3">
        <a href="{url}">{title}</a>
        <span class="dash flex-grow-1"></span>
        <span class="text-muted small">{date}</span>
      </li>"##,
                url = post_url(post),
                title = escape_html(&post.title)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = format!(
        r##"  <div id="{id}">
    <h1 class="ps-lg-2">
      {icon_name}
      {name}
      <span class="lead text-muted ps-2">{count}</span>
    </h1>
    <ul class="{list_class} ps-0">
{items}
    </ul>
  </div>"##,
        id = if is_category {
            "page-category"
        } else {
            "page-tag"
        },
        name = escape_html(&name),
        count = posts.len(),
        list_class = if is_category {
            "content"
        } else {
            "post-content"
        }
    );

    let mut all_crumbs = crumbs_home("主页");
    all_crumbs.extend(crumbs.iter().cloned());
    let page_title = format!("{heading}：{name}");
    let fallback_recent = if ctx.recent.is_empty() {
        panel_post_links(posts, 5)
    } else {
        Vec::new()
    };
    let recent = if fallback_recent.is_empty() {
        ctx.recent
    } else {
        &fallback_recent
    };
    let ctx = Ctx {
        assets: ctx.assets,
        path: ctx.path,
        recent,
        top_tags: ctx.top_tags,
    };
    render_layout(
        &ctx,
        Some(&page_title),
        SITE.description,
        &body,
        &breadcrumb(&all_crumbs),
        Some(heading),
        "",
    )
}

pub fn post_page(ctx: &Ctx, post: &Post, related: &[RelatedLink]) -> String {
    let tooltip = format_date_time(&post.date);
    let updated = match &post.updated {
        Some(updated) => format!(
            r##"      <span>
        更新于
        <em class="" data-bs-placement="bottom" data-bs-toggle="tooltip" data-bs-title="{tooltip}">{date}</em>
      </span>
"##,
            tooltip = format_date_time(updated),
            date = format_date(updated)
        ),
        None => String::new(),
    };
    // JS 的 String.length 按 UTF-16 code unit 计数，正则 \s 去空白。
    let words: usize = post
        .body
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .map(char::len_utf16)
        .sum();
    let read_min = words / 450;
    let read_max = words / 350;

    let categories = if post.categories.is_empty() {
        String::new()
    } else {
        let links = post
            .categories
            .iter()
            .map(|name| {
                format!(
                    r##"<a class="post-tag no-text-decoration" href="{}">{}</a>"##,
                    category_url(name),
                    escape_html(name)
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!(
            r##"      <div class="post-meta mb-3">
        <i class="far fa-folder-open fa-fw me-1"></i>
        {links}
      </div>
"##
        )
    };
    let tags = if post.tags.is_empty() {
        String::new()
    } else {
        let links = post
            .tags
            .iter()
            .map(|name| {
                format!(
                    r##"        <a class="post-tag no-text-decoration" href="{}">{}</a>"##,
                    tag_url(name),
                    escape_html(name)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            r##"      <div class="post-tags">
        <i class="fa fa-tags fa-fw me-1"></i>
{links}
      </div>
"##
        )
    };

    let share_url = url::encode(&format!("{}{}", SITE.url, post_url(post)));
    let share = format!(
        r##"      <div class="share-wrapper">
        <span class="share-label text-muted me-1">分享</span>
        <span class="share-icons">
          <span>
            <a href="https://t.me/share/url?url={share_url}" data-bs-toggle="tooltip" data-bs-placement="top" target="_blank" rel="noopener" aria-label="Telegram" title="Telegram">{telegram}</a>
          </span><span>
            <a href="https://www.facebook.com/sharer/sharer.php?u={share_url}" data-bs-toggle="tooltip" data-bs-placement="top" target="_blank" rel="noopener" aria-label="Facebook" title="Facebook">{facebook}</a>
          </span><span>
            <a href="https://twitter.com/intent/tweet?url={share_url}" data-bs-toggle="tooltip" data-bs-placement="top" target="_blank" rel="noopener" aria-label="X" title="X">{twitter}</a>
          </span>
          <a href="javascript:void(0)" id="copy-link" data-copy-link title="复制链接">
            <i class="fa-fw fas fa-link small" id="copy-link-icon"></i>
          </a>
        </span>
      </div>"##,
        telegram = icon("fa-fw fab fa-telegram"),
        facebook = icon("fa-fw fab fa-facebook-square"),
        twitter = icon("fa-fw fa-brands fa-square-x-twitter")
    );

    let related = if related.is_empty() {
        String::new()
    } else {
        let items = related
            .iter()
            .take(3)
            .map(|item| {
                format!(
                    r##"              <article class="col">
                <a href="{url}" class="post-preview card h-100">
                  <div class="card-body">
                    <time class="small" style="color: #a4a4a4">{date}</time>
                    <h4 class="pt-0 my-2">{title}</h4>
                    <div class="text-muted small">
                      <p>{summary}</p>
                    </div>
                  </div>
                </a>
              </article>"##,
                    url = item.url,
                    date = format_date(&item.date),
                    title = escape_html(&item.title),
                    summary = escape_html(&item.summary)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            r##"        <aside id="related-posts" aria-labelledby="related-label">
          <h3 class="mb-4">相关文章</h3>
          <div>
            <nav class="row row-cols-1 row-cols-md-2 row-cols-xl-3 g-4 mb-4">
{items}
            </nav>
          </div>
        </aside>
"##
        )
    };

    let title = escape_html(&post.title);
    let date = format_date(&post.date);
    let author_block = if SITE.author.is_empty() {
        String::new()
    } else {
        format!(
            "        <!-- author(s) -->\n        <span>\n          作者\n          <em>{}</em>\n        </span>",
            escape_html(SITE.author)
        )
    };
    let license_url = SITE.license_url;
    let license_name = escape_html(SITE.license_name);
    let html = &post.html;
    let body = format!(
        r##"  <header>
    <h1 data-toc-skip>{title}</h1>
    <div class="post-meta text-muted">
      <!-- published date -->
      <span>
        发表于
        <em class="" data-bs-placement="bottom" data-bs-toggle="tooltip" data-bs-title="{tooltip}">{date}</em>
      </span>
      <!-- lastmod date -->
{updated}      <div class="d-flex justify-content-between">
{author_block}
        <div>
          <!-- read time -->
          <span class="readtime" data-bs-original-title="{words} 字" data-bs-placement="bottom" data-bs-toggle="tooltip">
            <em>{read_min}~{read_max} 分钟</em>
            阅读
          </span>
        </div>
      </div>
      <!-- .d-flex -->
    </div>
    <!-- .post-meta -->
  </header>

  <div class="post-meta-line d-flex align-items-center flex-wrap text-muted">
{categories}{tags}  </div>

  <div class="content markdown-body">
{html}
  </div>

  <div class="post-tail-wrapper text-muted">
    <!-- license && sharing -->
    <div class="post-tail-bottom d-flex justify-content-between align-items-center mt-3 pt-5 pb-2">
      <div class="license-wrapper">
        许可协议:&nbsp;
        <a id="myLink" href="{license_url}">{license_name}</a>
      </div>
{share}
    </div>
    <!-- .post-tail-bottom -->
  </div>
  <!-- further reading -->
  <div class="row">
    <div class="col-12 px-md-4">
{related}    </div>
  </div>
"##
    );

    let mut crumbs = crumbs_home("主页");
    crumbs.push(Breadcrumb {
        name: post.title.clone(),
        href: None,
    });
    let toc = toc_wrapper(&post.toc);
    render_layout(
        ctx,
        Some(&post.title),
        &post.summary,
        &body,
        &breadcrumb(&crumbs),
        Some("文章"),
        &toc,
    )
}

pub fn archive_page(ctx: &Ctx, posts: &[&Post]) -> String {
    let mut by_year: Vec<(i32, Vec<&Post>)> = Vec::new();
    for post in posts {
        let year = post.date.year();
        match by_year.iter_mut().find(|(y, _)| *y == year) {
            Some((_, items)) => items.push(post),
            None => by_year.push((year, vec![post])),
        }
    }
    by_year.sort_by_key(|entry| Reverse(entry.0));

    let sections = by_year
        .into_iter()
        .map(|(year, mut items)| {
            items.sort_by_key(|post| Reverse(post.date));
            let list = items
                .iter()
                .map(|post| {
                    format!(
                        r##"          <li>
            <span class="date day">{day}</span>
            <span class="date month small text-muted ms-1">{month}月</span>
            <a href="{url}">{title}</a>
          </li>"##,
                        day = pad2(post.date.day()),
                        month = pad2(post.date.month()),
                        url = post_url(post),
                        title = escape_html(&post.title)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                r##"      <div class="year lead">{year}</div>
      <ul class="list-unstyled">
{list}
      </ul>"##
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = format!(
        r##"  <div class="pl-xl-3" id="archives">
{sections}
  </div>"##
    );

    let mut crumbs = crumbs_home("主页");
    crumbs.push(Breadcrumb {
        name: "归档".to_string(),
        href: None,
    });
    render_layout(
        ctx,
        Some("归档"),
        SITE.description,
        &body,
        &breadcrumb(&crumbs),
        Some("归档"),
        "",
    )
}

pub fn taxonomy_index(ctx: &Ctx, kind: &str, map: &Taxonomy, posts: &[Post]) -> String {
    let is_tags = kind == "tags";
    let heading = if is_tags { "标签" } else { "分类" };
    let mut sorted: Vec<(&String, &Vec<usize>)> = map.iter().collect();
    sorted.sort_by_key(|entry| Reverse(entry.1.len()));

    let content = if is_tags {
        let items = sorted
            .iter()
            .map(|(name, list)| {
                format!(
                    r##"          <div>
            <a class="tag" href="{url}">
              {name}
              <span class="text-muted">{count}</span>
            </a>
          </div>"##,
                    url = tag_url(name),
                    name = escape_html(name),
                    count = list.len()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            r##"      <div class="content">
        <div class="d-flex flex-wrap mx-xl-2" id="tags">
{items}
        </div>
      </div>"##
        )
    } else {
        let items = sorted
            .iter()
            .enumerate()
            .map(|(index, (name, list))| {
                format!(
                    r##"        <div class="card categories">
          <!-- top-category -->
          <div class="card-header d-flex justify-content-between hide-border-bottom" id="h_{id}">
            <!-- not have subcategory -->
            <span class="ms-2">
              <i class="far fa-folder fa-fw"></i>
              <a class="mx-2" href="{url}">{name}</a>
              <!-- content count -->
              <span class="text-muted small font-weight-light">
                {count} 篇文章
              </span>
            </span>
            <!-- arrow -->
            <span class="category-trigger hide-border-bottom disabled" data-bs-toggle="collapse">
              <i class="fas fa-fw fa-angle-right"></i>
            </span>
          </div>
          <!-- Sub-categories -->
        </div>"##,
                    id = index + 1,
                    url = category_url(name),
                    name = escape_html(name),
                    count = list.len()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            r##"      <div class="content">
{items}
      </div>"##
        )
    };

    let body = format!(
        r##"  <h1 class="dynamic-title">{heading}</h1>
{content}"##
    );

    let fallback_recent;
    let recent = if ctx.recent.is_empty() {
        let mut flat: Vec<&Post> = map
            .values()
            .flat_map(|list| list.iter().map(|index| &posts[*index]))
            .collect();
        flat.sort_by_key(|post| Reverse(post.date));
        flat.truncate(5);
        fallback_recent = flat
            .iter()
            .map(|post| Link {
                title: post.title.clone(),
                url: post_url(post),
            })
            .collect::<Vec<_>>();
        fallback_recent.as_slice()
    } else {
        ctx.recent
    };
    let fallback_tags;
    let top_tags = if is_tags && ctx.top_tags.is_empty() {
        fallback_tags = panel_tag_links(map, 10);
        fallback_tags.as_slice()
    } else {
        ctx.top_tags
    };
    let ctx = Ctx {
        assets: ctx.assets,
        path: ctx.path,
        recent,
        top_tags,
    };

    let mut crumbs = crumbs_home("主页");
    crumbs.push(Breadcrumb {
        name: heading.to_string(),
        href: None,
    });
    render_layout(
        &ctx,
        Some(heading),
        SITE.description,
        &body,
        &breadcrumb(&crumbs),
        Some(heading),
        "",
    )
}

pub fn about_page(ctx: &Ctx, title: &str, html: &str) -> String {
    let title_esc = escape_html(title);
    let body = format!(
        r##"  <h1 data-toc-skip>{title_esc}</h1>
  <div class="post-content markdown-body">
{html}
  </div>"##
    );

    let mut crumbs = crumbs_home("主页");
    crumbs.push(Breadcrumb {
        name: title.to_string(),
        href: None,
    });
    render_layout(
        ctx,
        Some(title),
        SITE.description,
        &body,
        &breadcrumb(&crumbs),
        Some(""),
        "",
    )
}

pub fn not_found_page(ctx: &Ctx, path: &str) -> String {
    let body = r##"  <div class="content">
    <p class="lead">
      没有找到你请求的页面。
      <br>
      点击 <a href="/">这里</a> 返回首页。
    </p>
  </div>"##;

    let effective_path = if path.is_empty() { "/" } else { path };
    let ctx = Ctx {
        assets: ctx.assets,
        path: effective_path,
        recent: ctx.recent,
        top_tags: ctx.top_tags,
    };
    let title = "404: 页面不存在";
    let mut crumbs = crumbs_home("主页");
    crumbs.push(Breadcrumb {
        name: title.to_string(),
        href: None,
    });
    render_layout(
        &ctx,
        Some(title),
        SITE.description,
        body,
        &breadcrumb(&crumbs),
        Some("404"),
        "",
    )
}

/* ------------------------------------------------------------ feed / sitemap */

pub fn feed_xml(posts: &[&Post], root_url: &str, description: &str) -> String {
    let items = posts
        .iter()
        .take(20)
        .map(|post| {
            let url = format!("{root_url}{}", post_url(post));
            let categories = post
                .tags
                .iter()
                .map(|tag| format!("<category>{}</category>", escape_html(tag)))
                .collect::<Vec<_>>()
                .join("\n    ");
            format!(
                r##"  <item>
    <title>{title}</title>
    <link>{url}</link>
    <guid isPermaLink="true">{url}</guid>
    <pubDate>{pub_date}</pubDate>
    <description>{description}</description>
    {categories}
  </item>"##,
                title = escape_html(&post.title),
                pub_date = post.date.format("%a, %d %b %Y %H:%M:%S GMT"),
                description = escape_html(&post.summary)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let title = escape_html(SITE.title);
    let description = escape_html(description);
    let last_build = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT");
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
  <title>{title}</title>
  <link>{root_url}/</link>
  <description>{description}</description>
  <language>{lang}</language>
  <atom:link href="{root_url}/feed.xml" rel="self" type="application/rss+xml"/>
  <lastBuildDate>{last_build}</lastBuildDate>
{items}
</channel>
</rss>
"##,
        lang = SITE.feed_lang
    )
}

pub fn sitemap_xml(paths: &[String], root_url: &str) -> String {
    let entries = paths
        .iter()
        .map(|path| format!("  <url><loc>{root_url}{path}</loc></url>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{entries}
</urlset>
"##
    )
}
