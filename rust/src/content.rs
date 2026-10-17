// 内容层：content/ 目录就是全部数据源。逐行对齐 src/content.ts 的语义。
// 时区约定：构建机 TZ=UTC，Node 的 new Date("YYYY-MM-DD HH:MM:SS") 也按 UTC 解析，故统一用 UTC。

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
    let title = get_str(&data, "title").unwrap_or_else(|| slug.clone());
    let date = get_str(&data, "date")
        .as_deref()
        .and_then(parse_date)
        .or_else(|| {
            fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(DateTime::<Utc>::from)
        })
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());
    let updated = get_str(&data, "updated").as_deref().and_then(parse_date);
    let tags = get_list(&data, "tags");
    let categories = get_list(&data, "categories");
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
    let file_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");

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
            let raw = fs::read_to_string(path)?;
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

pub fn parse_date(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let normalized = value.replace(' ', "T");
    if let Ok(dt) = DateTime::parse_from_rfc3339(&normalized) {
        return Some(dt.with_timezone(&Utc));
    }
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
