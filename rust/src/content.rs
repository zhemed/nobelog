// 内容层：content/ 目录就是全部数据源。逐行对齐 src/content.ts 的语义。
// 时区约定：日期字段统一存 **UTC+8 挂钟**（frontmatter 写本地时间，文件 mtime 也换算到同一挂钟），
// 展示层直接读挂钟分量，RSS 输出前再减 8 小时还原真实 UTC。

use crate::markdown;
use crate::model::{Content, Post, SitePage, Taxonomy};
use anyhow::{Context as _, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use indexmap::IndexMap;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

pub fn load(root: &Path) -> Result<Content> {
    let mut posts = read_posts(root)?;
    posts.retain(|p| !p.draft);
    for post in &mut posts {
        let (html, toc) = markdown::render_shared(&post.body);
        post.html = html;
        post.toc = toc;
        post.summary = if post.description.is_empty() {
            slice_utf16(&first_paragraph(&post.html), 160)
        } else {
            post.description.clone()
        };
        post.reading_minutes = count_reading_minutes(&post.body);
    }

    // pinned 在前；其余按发布时间倒序（稳定排序，与 JS 一致）
    posts.sort_by(|a, b| match (a.pinned, b.pinned) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.date.cmp(&a.date),
    });

    let tags = build_taxonomy(&posts, |p| &p.tags);
    let categories = build_taxonomy(&posts, |p| &p.categories);
    let pages = read_pages(root)?;

    Ok(Content {
        posts,
        tags,
        categories,
        pages,
    })
}

fn read_posts(root: &Path) -> Result<Vec<Post>> {
    let dir = root.join("content/posts");
    let mut posts = Vec::new();
    if !dir.is_dir() {
        return Ok(posts);
    }
    let mut files: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            matches!(
                p.extension().and_then(|s| s.to_str()),
                Some("md") | Some("mdx")
            )
        })
        .collect();
    files.sort();
    let sources: Vec<Result<Post>> = files
        .par_iter()
        .map(|path| parse_source(root, path))
        .collect();
    for source in sources {
        posts.push(source?);
    }
    Ok(posts)
}

fn parse_source(root: &Path, path: &Path) -> Result<Post> {
    let raw = fs::read_to_string(path).with_context(|| format!("读取 {}", path.display()))?;
    let (front, body) = split_front_matter(&raw);
    let data: serde_yaml::Value = if front.trim().is_empty() {
        serde_yaml::Value::Null
    } else {
        serde_yaml::from_str(front).unwrap_or(serde_yaml::Value::Null)
    };

    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let slug = get_str(&data, "slug").unwrap_or(file_stem);
    // slug / 标签 / 分类都会变成 URL，再由构建器还原成磁盘路径：必须在入口处就拒绝
    // 任何可能逃出输出目录的形状，并带上文件路径让构建响亮失败（而不是静默跳过）。
    let display = display_path(root, path);
    validate_slug(&slug).with_context(|| display.clone())?;
    let title = get_str(&data, "title").unwrap_or_else(|| slug.clone());
    let date = get_str(&data, "date")
        .as_deref()
        .and_then(parse_date)
        .or_else(|| {
            fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(wall_clock_from_system_time)
        })
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());
    let updated = get_str(&data, "updated").as_deref().and_then(parse_date);
    let tags = get_list(&data, "tags");
    let categories = get_list(&data, "categories");
    for tag in &tags {
        validate_taxonomy_name(tag).with_context(|| format!("{display}: 标签"))?;
    }
    for category in &categories {
        validate_taxonomy_name(category).with_context(|| format!("{display}: 分类"))?;
    }
    let description = get_str(&data, "description")
        .unwrap_or_default()
        .trim()
        .to_string();
    let cover = get_str(&data, "cover");
    let pinned = get_bool(&data, "pinned");
    let draft = get_bool(&data, "draft");
    let body = body.trim().to_string();

    // 渲染必须按文件顺序串行（Node 的 slugCount 跨文章累积）；这里只做 IO/解析。
    let html = String::new();
    let toc = Vec::new();
    let summary = String::new();
    let reading_minutes = 0;
    let file_path = display;

    Ok(Post {
        slug,
        title,
        date,
        updated,
        tags,
        categories,
        description,
        cover,
        pinned,
        draft,
        body,
        file_path,
        html,
        toc,
        summary,
        reading_minutes,
    })
}

fn read_pages(root: &Path) -> Result<IndexMap<String, SitePage>> {
    let mut pages = IndexMap::new();
    let dir = root.join("content/pages");
    if !dir.is_dir() {
        return Ok(pages);
    }
    let mut files: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();
    files.sort();
    // 单页之间互相独立（各自新建 renderer），可以并行；结果按文件顺序插入。
    let rendered: Vec<Result<(String, SitePage)>> = files
        .par_iter()
        .map(|path| {
            let display = display_path(root, path);
            let raw = fs::read_to_string(path).with_context(|| display.clone())?;
            let (front, body) = split_front_matter(&raw);
            let title = front_line(front, "title").unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string()
            });
            let slug = front_line(front, "slug").unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string()
            });
            validate_slug(&slug).with_context(|| format!("{display}: 单页 slug"))?;
            let (html, _toc) = markdown::render(body.trim());
            Ok((slug.clone(), SitePage { slug, title, html }))
        })
        .collect();
    for page in rendered {
        let (slug, page) = page?;
        pages.insert(slug, page);
    }
    Ok(pages)
}

// ---------------------------------------------------------------- 路径安全校验
//
// slug / 标签 / 分类会先编码进 URL，再由 build::file_for_route 解码成磁盘路径。
// 任何含路径分隔符、`.` / `..` 路径段、控制字符的名字都能把产物写到 --out 之外，
// 所以在这里（数据入口）就拒绝，并带上源文件路径，让构建**明确失败**而不是静默跳过。

/// 相对 `root` 的文件路径，统一用 `/` 分隔，用于错误信息。
fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// 名字里是否含空段 / `.` / `..` 路径段（按两种分隔符切分，作为纵深防御）。
fn has_traversal_segment(value: &str) -> bool {
    value
        .split(['/', '\\'])
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

/// 文章 / 单页 slug：必须是不含路径分隔符的单段名字。
fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        anyhow::bail!("slug 不能为空");
    }
    if slug.contains('/') || slug.contains('\\') {
        anyhow::bail!("slug 不能包含路径分隔符：{slug:?}");
    }
    if slug.chars().any(char::is_control) {
        anyhow::bail!("slug 不能包含控制字符：{slug:?}");
    }
    if has_traversal_segment(slug) {
        anyhow::bail!("slug 不能是 `.` / `..`（路径段含义）：{slug:?}");
    }
    Ok(())
}

/// 标签 / 分类：允许中文与空格，但同样必须是单段名字。
fn validate_taxonomy_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("标签/分类不能为空");
    }
    if name.contains('/') || name.contains('\\') {
        anyhow::bail!("标签/分类不能包含路径分隔符：{name:?}");
    }
    if name.chars().any(char::is_control) {
        anyhow::bail!("标签/分类不能包含控制字符：{name:?}");
    }
    if has_traversal_segment(name) {
        anyhow::bail!("标签/分类不能是 `.` / `..`：{name:?}");
    }
    Ok(())
}

fn front_line(front: &str, key: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?m)^{}:\s*(.+)$", regex::escape(key))).ok()?;
    let cap = re.captures(front)?;
    let value = cap
        .get(1)?
        .as_str()
        .trim()
        .trim_matches(|c| c == '"' || c == '\'');
    Some(value.to_string())
}

fn split_front_matter(raw: &str) -> (&str, &str) {
    let start = raw.strip_prefix("---");
    let Some(rest) = start else { return ("", raw) };
    let rest = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))
        .unwrap_or(rest);
    if let Some(idx) = rest.find("\n---") {
        let front = &rest[..idx];
        let after = &rest[idx + 4..];
        let body = after.strip_prefix('\n').unwrap_or(after);
        (front, body)
    } else {
        ("", raw)
    }
}

/// 站内日期统一按 UTC+8 挂钟存储（frontmatter 写的是本地时间）。
/// 展示层直接读挂钟分量，RSS 输出前再减 8 小时还原真实 UTC
/// （见 `templates::card_date` 与 `templates::rss_pub_date` 的同一约定）。
const WALL_CLOCK_OFFSET_HOURS: i64 = 8;

/// 真实 UTC 时刻 → 站内 UTC+8 挂钟值（`Utc` 只是承载体，不表示时区）。
fn wall_clock_from_utc(utc: DateTime<Utc>) -> DateTime<Utc> {
    utc + chrono::Duration::hours(WALL_CLOCK_OFFSET_HOURS)
}

/// 文件 mtime（系统给的是真实 UTC）→ 站内挂钟；没写 `date` 的文章靠它兜底，
/// 必须走同一换算，否则这类文章的 feed 会比真实时刻早 8 小时、页面显示 UTC 时间。
fn wall_clock_from_system_time(time: std::time::SystemTime) -> DateTime<Utc> {
    wall_clock_from_utc(DateTime::<Utc>::from(time))
}

/// 规范化 frontmatter 的日期串：
/// - 第一个空白换成 `T`（`2026-10-17 22:40:00` → `2026-10-17T22:40:00`），
///   若空白前已经是 ISO 的 `T`/`t`（如 `...T22:40:00 +08:00`）则直接丢掉；
/// - 其余空白一律删除，这样 `2026-10-17 22:40:00 +08:00` 与
///   `2026-10-17T22:40:00+08:00` 走同一条解析路径。
fn normalize_date(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut separator_done = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            if !separator_done && !out.contains(['T', 't']) {
                out.push('T');
            }
            separator_done = true;
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn parse_date(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let normalized = normalize_date(value);
    if let Ok(dt) = DateTime::parse_from_rfc3339(&normalized) {
        // 带偏移的输入描述的是**真实时刻**，而内部约定是 UTC+8 挂钟：先换算成 UTC，
        // 再加回 8 小时转成挂钟值。否则同一时刻写成 Z 或 +08:00 时，会比裸写挂钟差 8 小时：
        //   裸写 2026-10-17 22:40:00 == 2026-10-17T22:40:00+08:00 == 2026-10-17T14:40:00Z
        // 三者都归一成挂钟 2026-10-17 22:40:00。
        return Some(wall_clock_from_utc(dt.with_timezone(&Utc)));
    }
    // 无偏移的时间按挂钟原样存储（`Utc` 只是承载体，不表示时区）。
    for fmt in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"] {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(&normalized, fmt) {
            return Some(Utc.from_utc_datetime(&ndt));
        }
    }
    if let Ok(d) = NaiveDate::parse_from_str(&normalized, "%Y-%m-%d") {
        return Some(Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0)?));
    }
    None
}

fn build_taxonomy(posts: &[Post], pick: impl Fn(&Post) -> &Vec<String>) -> Taxonomy {
    let mut map: Taxonomy = IndexMap::new();
    for (idx, post) in posts.iter().enumerate() {
        for name in pick(post) {
            map.entry(name.clone()).or_default().push(idx);
        }
    }
    for list in map.values_mut() {
        list.sort_by(|a, b| posts[*b].date.cmp(&posts[*a].date));
    }
    map
}

fn get_str(data: &serde_yaml::Value, key: &str) -> Option<String> {
    match data.get(key) {
        Some(serde_yaml::Value::String(s)) => Some(s.clone()),
        Some(serde_yaml::Value::Number(n)) => Some(n.to_string()),
        Some(serde_yaml::Value::Bool(b)) => Some(b.to_string()),
        _ => None,
    }
}

fn get_list(data: &serde_yaml::Value, key: &str) -> Vec<String> {
    match data.get(key) {
        Some(serde_yaml::Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| match v {
                serde_yaml::Value::String(s) => Some(s.trim().to_string()),
                serde_yaml::Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .filter(|s| !s.is_empty())
            .collect(),
        Some(serde_yaml::Value::String(s)) => s
            .split([',', '，'])
            .map(|part| part.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn get_bool(data: &serde_yaml::Value, key: &str) -> bool {
    matches!(data.get(key), Some(serde_yaml::Value::Bool(true)))
}

fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                out.push(' ');
            }
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn first_paragraph(html: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(?s)<p[^>]*>(.*?)</p>").unwrap());
    match re.captures(html) {
        Some(cap) => strip_tags(&cap[1]),
        None => strip_tags(html),
    }
}

fn slice_utf16(text: &str, max: usize) -> String {
    let mut out = String::new();
    let mut units = 0usize;
    for ch in text.chars() {
        let size = ch.len_utf16();
        if units + size > max {
            break;
        }
        units += size;
        out.push(ch);
    }
    out
}

fn count_reading_minutes(text: &str) -> u32 {
    static LATIN: OnceLock<Regex> = OnceLock::new();
    let latin_re = LATIN.get_or_init(|| Regex::new(r"[A-Za-z0-9]+").unwrap());
    let latin = latin_re.find_iter(text).count() as f64;
    let cjk = text
        .chars()
        .filter(|c| ('\u{4e00}'..='\u{9fa5}').contains(c))
        .count() as f64;
    let minutes = (latin / 220.0 + cjk / 400.0).round();
    (minutes as u32).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    /// 每次调用给一个独立的临时 root（content/posts 与 content/pages 已建好）。
    fn temp_root(tag: &str) -> PathBuf {
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "nobelog-content-test-{}-{tag}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("content/posts")).unwrap();
        fs::create_dir_all(root.join("content/pages")).unwrap();
        root
    }

    fn write_file(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    #[test]
    fn slug_validation_rejects_traversal_shapes() {
        for bad in [
            "",
            ".",
            "..",
            "../../pwned",
            "/etc/passwd",
            "posts/../../pwned",
            r"..\..\pwned",
            "a/b",
            r"a\b",
            "with\u{0}nul",
            "line\nbreak",
        ] {
            assert!(
                validate_slug(bad).is_err(),
                "slug {bad:?} 必须被拒绝：{:?}",
                validate_slug(bad)
            );
        }
    }

    #[test]
    fn slug_validation_accepts_normal_names() {
        for good in [
            "hello",
            "server-init-commands",
            "中文-slug",
            "a..b",
            ".hidden",
        ] {
            assert!(
                validate_slug(good).is_ok(),
                "slug {good:?} 应通过：{:?}",
                validate_slug(good)
            );
        }
    }

    #[test]
    fn taxonomy_validation_allows_chinese_but_rejects_separators() {
        for good in ["运维", "排版", "Linux", "中文 标签", "C++"] {
            assert!(
                validate_taxonomy_name(good).is_ok(),
                "标签 {good:?} 应通过：{:?}",
                validate_taxonomy_name(good)
            );
        }
        for bad in [
            "",
            ".",
            "..",
            "../escape",
            "a/b",
            r"a\b",
            "tab\there",
            "nul\0",
        ] {
            assert!(
                validate_taxonomy_name(bad).is_err(),
                "标签 {bad:?} 必须被拒绝：{:?}",
                validate_taxonomy_name(bad)
            );
        }
    }

    #[test]
    fn load_rejects_traversing_post_slug_with_file_path() {
        let root = temp_root("post-slug");
        write_file(
            &root,
            "content/posts/evil.md",
            "---\ntitle: evil\nslug: ../../pwned\n---\n\nbody\n",
        );
        let err = load(&root).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("content/posts/evil.md"),
            "错误需带文件路径：{msg}"
        );
        assert!(msg.contains("slug"), "错误需说明是 slug：{msg}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn load_rejects_traversing_tag_and_category() {
        let root = temp_root("tags");
        write_file(
            &root,
            "content/posts/tagged.md",
            "---\ntitle: t\nslug: tagged\ntags: [\"../escape\"]\n---\n\nbody\n",
        );
        let err = load(&root).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("content/posts/tagged.md"),
            "错误需带文件路径：{msg}"
        );
        assert!(msg.contains("标签"), "错误需说明是标签：{msg}");
        let _ = fs::remove_dir_all(&root);

        let root = temp_root("categories");
        write_file(
            &root,
            "content/posts/cat.md",
            "---\ntitle: c\nslug: cat\ncategories: [\"..\\\\..\\\\escape\"]\n---\n\nbody\n",
        );
        let err = load(&root).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("content/posts/cat.md"),
            "错误需带文件路径：{msg}"
        );
        assert!(msg.contains("分类"), "错误需说明是分类：{msg}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn load_rejects_traversing_page_slug() {
        let root = temp_root("page-slug");
        write_file(
            &root,
            "content/pages/about.md",
            "---\ntitle: 关于\nslug: ../about\n---\n\nhi\n",
        );
        let err = load(&root).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("content/pages/about.md"),
            "错误需带文件路径：{msg}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn load_accepts_chinese_slug_and_taxonomy() {
        let root = temp_root("ok");
        write_file(
            &root,
            "content/posts/ok.md",
            "---\ntitle: 正常\nslug: 中文-slug\ntags: [运维, \"中文 标签\"]\ncategories: [备忘]\n---\n\n正文\n",
        );
        write_file(
            &root,
            "content/pages/about.md",
            "---\ntitle: 关于\nslug: about\n---\n\nhi\n",
        );
        let content = load(&root).unwrap();
        assert_eq!(content.posts.len(), 1);
        assert_eq!(content.posts[0].slug, "中文-slug");
        assert!(content.tags.contains_key("运维"));
        assert!(content.tags.contains_key("中文 标签"));
        assert!(content.categories.contains_key("备忘"));
        assert!(content.pages.contains_key("about"));
        let _ = fs::remove_dir_all(&root);
    }

    fn wall_clock_text(value: &str) -> String {
        parse_date(value)
            .unwrap_or_else(|| panic!("{value:?} 应能解析"))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }

    #[test]
    fn parse_date_normalizes_offsets_to_utc8_wall_clock() {
        // 同一个时刻的三种写法（裸写挂钟 / +08:00 / Z）必须归一到同一挂钟值 22:40。
        assert_eq!(
            wall_clock_text("2026-10-17 22:40:00"),
            "2026-10-17 22:40:00"
        );
        assert_eq!(
            wall_clock_text("2026-10-17T22:40:00+08:00"),
            "2026-10-17 22:40:00"
        );
        assert_eq!(
            wall_clock_text("2026-10-17T14:40:00Z"),
            "2026-10-17 22:40:00"
        );
        // Z 写法的真实时刻是 22:40Z，换算成 UTC+8 挂钟是次日 06:40。
        assert_eq!(
            wall_clock_text("2026-10-17T22:40:00Z"),
            "2026-10-18 06:40:00"
        );
        // 非 +08:00 的偏移同样按真实时刻换算：14:40-05:00 == 19:40Z == 次日 03:40 挂钟。
        assert_eq!(
            wall_clock_text("2026-10-17T14:40:00-05:00"),
            "2026-10-18 03:40:00"
        );
        // 纯日期行为不变：当天 00:00 挂钟。
        assert_eq!(wall_clock_text("2026-10-17"), "2026-10-17 00:00:00");
        // 偏移前带空格的写法（P3 回归）：规范化后与无空格写法完全等价。
        assert_eq!(
            wall_clock_text("2026-10-17 22:40:00 +08:00"),
            "2026-10-17 22:40:00"
        );
        assert_eq!(
            wall_clock_text("2026-10-17T22:40:00 +08:00"),
            "2026-10-17 22:40:00"
        );
        assert_eq!(
            wall_clock_text("2026-10-17  22:40:00  +08:00"),
            "2026-10-17 22:40:00"
        );
        assert_eq!(
            wall_clock_text("  2026-10-17 14:40:00Z  "),
            "2026-10-17 22:40:00"
        );
    }

    #[test]
    fn mtime_fallback_is_stored_as_utc8_wall_clock() {
        // 没有 `date` 的文章用文件 mtime 兜底；mtime 是真实 UTC，内部必须换算成 UTC+8 挂钟，
        // 这样 feed 的 pubDate 才等于 mtime 本身的真实时刻（−0h 误差）。
        let root = temp_root("mtime");
        write_file(
            &root,
            "content/posts/no-date.md",
            "---\ntitle: 无日期\nslug: no-date\n---\n\nbody\n",
        );
        let path = root.join("content/posts/no-date.md");
        // 固定 mtime = 2026-10-17T14:40:00Z（真实 UTC）。
        let instant = Utc.with_ymd_and_hms(2026, 10, 17, 14, 40, 0).unwrap();
        let file = fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.set_modified(std::time::SystemTime::from(instant))
            .unwrap();
        drop(file);

        let content = load(&root).unwrap();
        assert_eq!(
            content.posts[0]
                .date
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            "2026-10-17 22:40:00",
            "mtime 未归一成 UTC+8 挂钟"
        );
        let refs: Vec<&Post> = content.posts.iter().collect();
        let xml = crate::templates::feed_xml(&refs, "https://example.com", "站点描述");
        assert!(
            xml.contains("<pubDate>Sat, 17 Oct 2026 14:40:00 GMT</pubDate>"),
            "mtime 兜底的 pubDate 应等于 mtime 的真实 UTC：{xml}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn feed_pub_date_reflects_real_utc_for_every_date_syntax() {
        // 端到端：frontmatter 写法不同，feed 的 pubDate 必须是同一真实时刻。
        let cases = [
            // 裸写 = UTC+8 挂钟 22:40 → 真实 UTC 14:40。
            ("2026-10-17 22:40:00", "Sat, 17 Oct 2026 14:40:00 GMT"),
            // 显式 +08:00，同一时刻。
            ("2026-10-17T22:40:00+08:00", "Sat, 17 Oct 2026 14:40:00 GMT"),
            // 偏移前带空格的写法（P3 回归），同一时刻。
            (
                "2026-10-17 22:40:00 +08:00",
                "Sat, 17 Oct 2026 14:40:00 GMT",
            ),
            // 显式 Z，同一时刻。
            ("2026-10-17T14:40:00Z", "Sat, 17 Oct 2026 14:40:00 GMT"),
            // Z 写法的真实时刻就是 22:40Z，不能被再减 8 小时。
            ("2026-10-17T22:40:00Z", "Sat, 17 Oct 2026 22:40:00 GMT"),
        ];
        for (front, expected) in cases {
            let root = temp_root("rss");
            write_file(
                &root,
                "content/posts/dated.md",
                &format!("---\ntitle: t\nslug: dated\ndate: {front}\n---\n\nbody\n"),
            );
            let content = load(&root).unwrap();
            let refs: Vec<&Post> = content.posts.iter().collect();
            let xml = crate::templates::feed_xml(&refs, "https://example.com", "站点描述");
            assert!(
                xml.contains(&format!("<pubDate>{expected}</pubDate>")),
                "frontmatter `date: {front}` 的 pubDate 应为 {expected}：{xml}"
            );
            let _ = fs::remove_dir_all(&root);
        }
    }
}
