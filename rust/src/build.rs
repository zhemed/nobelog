// 构建器（阶段一）：复用 public/assets 与 manifest，只重算 content → HTML。
// 产物写 rust/dist/，绝不覆盖冻结基准 public/。

use crate::paths::PRESERVE_IN_PUBLIC;
use crate::routes::Site;
use crate::templates::PageAssets;
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Manifest {
    css: String,
    js: String,
}

pub fn build(root: &Path, out: &Path, self_assets: bool) -> Result<()> {
    let started = std::time::Instant::now();
    if out.exists() {
        clean_dir(out)?;
    }
    fs::create_dir_all(out.join("assets"))?;

    // 阶段一默认沿用 Node 构建好的资源与 manifest；`--self-assets` 切到 Rust 自建（阶段二）。
    //
    // 注意：自建资源的 CSS 裁剪使用集合只读 vendor 样式、`content/` 与编译期内嵌源码，
    // **不读 out（可能等于 public 且刚被清空）也不读 public HTML**，所以这里的清空顺序
    // 不会影响 CSS 哈希；改动 assets.rs 时请保持这个性质。
    let assets = if self_assets {
        crate::assets::build_assets(root, out)?
    } else {
        let manifest_path = root.join("public/assets-manifest.json");
        let manifest_raw = fs::read_to_string(&manifest_path)
            .with_context(|| format!("缺少 {}", manifest_path.display()))?;
        let manifest: Manifest = serde_json::from_str(&manifest_raw)?;
        copy_dir(&root.join("public/assets"), &out.join("assets"))?;
        fs::write(out.join("assets-manifest.json"), &manifest_raw)?;
        PageAssets {
            css: manifest.css,
            js: manifest.js,
        }
    };

    // content/images → out/images
    let images = root.join("content/images");
    if images.is_dir() {
        copy_dir(&images, &out.join("images"))?;
    }

    let site = Site::new(root)?;
    let routes = site.all_routes(&assets);

    let mut html_files = 0usize;
    let mut bytes = 0usize;
    for route in &routes {
        if route.status != 200 {
            continue;
        }
        write_route_file(out, &route.path, route.html.as_bytes())?;
        html_files += 1;
        bytes += route.html.len();
    }

    // 静态托管的 404 约定文件
    let base_url = crate::site::env_url().unwrap_or_else(|| crate::site::SITE.url.to_string());
    let ctx = crate::templates::Ctx {
        assets: &assets,
        base_url: &base_url,
        path: "/",
        recent: &[],
        top_tags: &[],
    };
    let not_found = crate::templates::not_found_page(&ctx, "");
    fs::write(out.join("404.html"), not_found.as_bytes())?;

    // robots.txt：站点有 sitemap，顺带把抓取入口和 sitemap 地址告诉爬虫。
    // base_url 与 404.html / feed 完全同源（SITE_URL 优先，否则内置默认）。
    write_robots_txt(out, &base_url)?;

    println!(
        "nobelog-rs build: {} 个页面 · {} 字节 · {} ms → {}",
        html_files,
        bytes,
        started.elapsed().as_millis(),
        out.display()
    );
    Ok(())
}

fn clean_dir(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if PRESERVE_IN_PUBLIC.contains(&name.as_ref()) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in walkdir::WalkDir::new(from).min_depth(1) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(from)?;
        let target = to.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// 路由 → 磁盘文件；目录名取 URL 解码后的真实名字。
///
/// 路由来自内容（slug / 标签 / 分类），所以这里是磁盘层的最后一道闸：
/// 解码后逐段校验，`.`、`..`、空段、绝对路径、路径分隔符与控制字符一律报错，
/// 绝不让 `out` 之外的目标混进 `out.join(..)`。
fn file_for_route(out: &Path, pathname: &str) -> Result<PathBuf> {
    let decoded = percent_encoding::percent_decode_str(pathname)
        .decode_utf8_lossy()
        .to_string();
    if decoded.chars().any(char::is_control) {
        return Err(anyhow::anyhow!("非法路由（含控制字符）：{pathname:?}"));
    }
    // 站点内路径必须以单个 '/' 开头：`/x` 合法，`x` 与 `//x`（可解析成绝对路径）不合法。
    let Some(rest) = decoded.strip_prefix('/') else {
        return Err(anyhow::anyhow!(
            "非法路由（不是以 / 开头的站点路径）：{pathname:?}"
        ));
    };
    if rest.starts_with('/') {
        return Err(anyhow::anyhow!(
            "非法路由（多余的前导斜杠，疑似绝对路径）：{pathname:?}"
        ));
    }
    if rest.is_empty() {
        return Ok(out.join("index.html"));
    }
    // 目录路由（以 '/' 结尾）落到 index.html。
    let (body, is_dir) = match rest.strip_suffix('/') {
        Some(body) => (body, true),
        None => (rest, false),
    };
    let mut rel = PathBuf::new();
    for segment in body.split('/') {
        if segment.is_empty() {
            return Err(anyhow::anyhow!("非法路由（含空路径段）：{pathname:?}"));
        }
        if segment == "." || segment == ".." {
            return Err(anyhow::anyhow!(
                "非法路由（含 `{segment}` 路径段，可能是路径穿越）：{pathname:?}"
            ));
        }
        if segment.contains('\\') {
            return Err(anyhow::anyhow!("非法路由（路径段含反斜杠）：{pathname:?}"));
        }
        if windows_absolute_segment(segment) {
            return Err(anyhow::anyhow!(
                "非法路由（路径段形如绝对路径）：{pathname:?}"
            ));
        }
        rel.push(segment);
    }
    if is_dir {
        rel.push("index.html");
    }
    Ok(out.join(rel))
}

/// Windows 盘符前缀（`C:`），在某些平台上会被当成绝对路径。
fn windows_absolute_segment(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// 断言 `target` 或它已存在的最深祖先规范化后仍位于 `out` 之内。
///
/// 建目录之前也要查：目标目录可能还不存在，此时要保证“将要被创建”的路径挂在
/// `out` 里（否则 `out/posts` 是指向外面的符号链接时，create_dir_all 就已经越界了）。
fn ensure_inside_out(out: &Path, target: &Path) -> Result<()> {
    let out_canon = out
        .canonicalize()
        .with_context(|| format!("规范化输出目录 {}", out.display()))?;
    let probe = deepest_existing(target)
        .ok_or_else(|| anyhow::anyhow!("无法定位已存在目录：{}", target.display()))?;
    let probe_canon = probe
        .canonicalize()
        .with_context(|| format!("规范化目录 {}", probe.display()))?;
    if !probe_canon.starts_with(&out_canon) {
        return Err(anyhow::anyhow!(
            "拒绝写出输出目录：{} 不在 {} 之内",
            probe_canon.display(),
            out_canon.display()
        ));
    }
    Ok(())
}

/// 自身或最近的已存在祖先（`out` 一定存在，所以循环必然终止）。
fn deepest_existing(path: &Path) -> Option<&Path> {
    let mut probe = path;
    loop {
        if probe.exists() {
            return Some(probe);
        }
        probe = probe.parent()?;
    }
}

/// 写单条路由产物：先确认将要创建的位置仍在 `out` 之内，再建父目录，
/// 建完对规范化后的父目录复查一次（防符号链接/竞态），最后才落盘。
fn write_route_file(out: &Path, pathname: &str, bytes: &[u8]) -> Result<()> {
    let file = file_for_route(out, pathname)?;
    let Some(parent) = file.parent() else {
        return Err(anyhow::anyhow!("路由没有父目录：{pathname:?}"));
    };
    ensure_inside_out(out, parent)
        .with_context(|| format!("路由 {pathname:?} 的目标 {}", parent.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("创建目录 {}", parent.display()))?;
    ensure_inside_out(out, parent)
        .with_context(|| format!("路由 {pathname:?} 的目标 {}", parent.display()))?;
    if fs::symlink_metadata(&file).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err(anyhow::anyhow!("拒绝写入符号链接：{}", file.display()));
    }
    fs::write(&file, bytes).with_context(|| format!("写入 {}", file.display()))?;
    Ok(())
}

/// 自动生成文件的标记行：`#` 在 robots.txt 里是合法注释，同时用来区分来源——
/// 首行是这个标记的文件是「上次自动生成」的，可以安全覆盖；没有标记的就是用户手写的，保留。
const ROBOTS_MARKER: &str = "# Generated by nobelog — 手写请删除本行";

/// robots.txt 内容：站点有 sitemap，顺带告诉爬虫抓取入口与 sitemap 地址。
/// `base_url` 与 404.html / feed 同源（`SITE_URL` 优先，否则内置默认）。
fn robots_txt(base_url: &str) -> String {
    format!("{ROBOTS_MARKER}\nUser-agent: *\nAllow: /\n\nSitemap: {base_url}/sitemap.xml\n")
}

/// 写 robots.txt，按来源决定是否覆盖：
/// - 文件不存在 → 生成；
/// - 已存在且首行是 [`ROBOTS_MARKER`] → 上一次自动生成，**覆盖**（否则改 `SITE_URL` 后
///   sitemap 地址永远不更新）；
/// - 已存在且首行不是标记 → 用户手写（`PRESERVE_IN_PUBLIC` 会把它保留下来），**不覆盖**，
///   手写规则优先（比如自定义 `Disallow`）。
///
/// 返回值表示本次是否真的写入了文件。
fn write_robots_txt(out: &Path, base_url: &str) -> Result<bool> {
    let path = out.join("robots.txt");
    if path.exists() {
        // 读不出来（权限/非 UTF-8）时按手写处理：宁可保留，也不要冲掉用户文件。
        let generated = match fs::read_to_string(&path) {
            Ok(existing) => existing.lines().next() == Some(ROBOTS_MARKER),
            Err(_) => false,
        };
        if !generated {
            return Ok(false);
        }
    }
    fs::write(&path, robots_txt(base_url)).with_context(|| format!("写入 {}", path.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    /// 每个测试一个独立的 `<tmp>/<name>/out`，同级目录用来断言“没有写出 out”。
    fn temp_out(tag: &str) -> (PathBuf, PathBuf) {
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "nobelog-build-test-{}-{tag}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        let out = base.join("out");
        fs::create_dir_all(&out).unwrap();
        (base, out)
    }

    fn expected_err(out: &Path, pathname: &str) -> String {
        match file_for_route(out, pathname) {
            Ok(path) => panic!("{pathname:?} 应被拒绝，实际得到 {}", path.display()),
            Err(err) => format!("{err:#}"),
        }
    }

    #[test]
    fn file_for_route_rejects_traversal() {
        let (base, out) = temp_out("traversal");
        let marker = format!(
            "nobelog-escape-{}-{}",
            std::process::id(),
            SEQ.load(Ordering::Relaxed)
        );
        let cases = [
            // 任务点名的三组形状 + 变体
            "/../x".to_string(),
            "/%2e%2e/%2e%2e/etc/passwd".to_string(),
            "/posts/..%2F..%2Fpwned/".to_string(),
            format!("/../{marker}"),
            format!("/posts/../../{marker}"),
            "/posts/%2E%2E/%2E%2E/x".to_string(),
            "/./x".to_string(),
            "/posts/./x".to_string(),
            "/posts/..%5C..%5Cx".to_string(),
            "/posts\\..\\..\\x".to_string(),
            "/%00/x".to_string(),
        ];
        for bad in &cases {
            let msg = expected_err(&out, bad);
            assert!(
                msg.contains("非法路由"),
                "{bad:?} 的错误信息应说明非法路由：{msg}"
            );
        }
        // 同级目录不应被创建。
        assert!(!base.join(&marker).exists());
        assert!(!base.join("etc").exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn file_for_route_rejects_absolute_and_empty_segments() {
        let (base, out) = temp_out("absolute");
        for bad in [
            "posts/hello/",   // 不是站点绝对路径
            "//etc/passwd",   // 多余前导斜杠 → 绝对路径
            "/posts//hello/", // 空段
            "/posts/hello//", // 尾部空段
            "/C:/Windows/system32/",
            "/posts/%5C%5Cserver%5Cshare/",
        ] {
            let msg = expected_err(&out, bad);
            assert!(msg.contains("非法路由"), "{bad:?} 应报非法路由：{msg}");
        }
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn file_for_route_keeps_normal_routes() {
        let (base, out) = temp_out("normal");
        assert_eq!(file_for_route(&out, "/").unwrap(), out.join("index.html"));
        assert_eq!(
            file_for_route(&out, "/posts/hello/").unwrap(),
            out.join("posts").join("hello").join("index.html")
        );
        assert_eq!(
            file_for_route(&out, "/tags/%E4%B8%AD%E6%96%87/").unwrap(),
            out.join("tags").join("中文").join("index.html")
        );
        assert_eq!(
            file_for_route(&out, "/feed.xml").unwrap(),
            out.join("feed.xml")
        );
        assert_eq!(
            file_for_route(&out, "/page/2/").unwrap(),
            out.join("page").join("2").join("index.html")
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn write_route_file_stays_inside_out() {
        let (base, out) = temp_out("write");
        // 合法路由：正常建目录并落盘。
        write_route_file(&out, "/posts/hello/", b"ok").unwrap();
        assert_eq!(
            fs::read_to_string(out.join("posts/hello/index.html")).unwrap(),
            "ok"
        );
        // 穿越路由：直接报错，且不产生 out 之外的文件。
        let marker = format!(
            "nobelog-escape-{}-{}",
            std::process::id(),
            SEQ.load(Ordering::Relaxed)
        );
        assert!(write_route_file(&out, &format!("/../{marker}/"), b"pwned").is_err());
        assert!(write_route_file(&out, "/%2e%2e/%2e%2e/etc/passwd", b"x").is_err());
        assert!(!base.join(&marker).exists());
        assert!(!base.join("etc").exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn write_route_file_refuses_symlinked_directory() {
        let (base, out) = temp_out("symlink");
        let outside = base.join("outside");
        fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, out.join("posts")).unwrap();
        let err = write_route_file(&out, "/posts/hello/", b"pwned").unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("拒绝写出输出目录"),
            "应被 canonicalize 断言拦住：{msg}"
        );
        assert!(!outside.join("hello").exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn robots_txt_lists_sitemap_for_env_url_and_default_url() {
        // SITE_URL 设置时的形态（build 里 env_url() 会先 trim 掉尾部 `/`）。
        assert_eq!(
            robots_txt("https://z.miio.cc"),
            format!(
                "{ROBOTS_MARKER}\nUser-agent: *\nAllow: /\n\nSitemap: https://z.miio.cc/sitemap.xml\n"
            )
        );
        // 首行标记是合法注释，也是“自动生成”的唯一判据。
        assert!(robots_txt("https://z.miio.cc").starts_with(ROBOTS_MARKER));
        // SITE_URL 未设置时的回退值（与 404.html / feed 同一个 `SITE.url`）。
        let default_url = crate::site::SITE.url;
        assert_eq!(
            robots_txt(default_url),
            format!(
                "{ROBOTS_MARKER}\nUser-agent: *\nAllow: /\n\nSitemap: {default_url}/sitemap.xml\n"
            )
        );
        assert!(robots_txt(default_url).contains("Sitemap: http://127.0.0.1:10086/sitemap.xml"));
    }

    #[test]
    fn write_robots_txt_keeps_existing_handwritten_file() {
        let (base, out) = temp_out("robots");
        let path = out.join("robots.txt");
        // 缺失时生成默认内容。
        assert!(write_robots_txt(&out, "https://z.miio.cc").unwrap());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            robots_txt("https://z.miio.cc")
        );
        // 手写文件（`PRESERVE_IN_PUBLIC` 会保留）没有标记行，不被覆盖。
        let handwritten = "User-agent: *\nDisallow: /private/\n";
        fs::write(&path, handwritten).unwrap();
        assert!(!write_robots_txt(&out, "https://z.miio.cc").unwrap());
        assert_eq!(fs::read_to_string(&path).unwrap(), handwritten);
        // 手写文件即使第一行是注释、但不是我们的标记，也按手写保留。
        let other_comment = "# 自定义 robots\nUser-agent: *\nDisallow: /\n";
        fs::write(&path, other_comment).unwrap();
        assert!(!write_robots_txt(&out, "https://z.miio.cc").unwrap());
        assert_eq!(fs::read_to_string(&path).unwrap(), other_comment);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn write_robots_txt_rewrites_generated_file_after_base_url_change() {
        // 自动产物（带标记）应随 SITE_URL 更新，否则改域名后 sitemap 地址永远不更新。
        let (base, out) = temp_out("robots-update");
        let path = out.join("robots.txt");
        assert!(write_robots_txt(&out, "https://a.example").unwrap());
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .contains("Sitemap: https://a.example/sitemap.xml")
        );
        assert!(
            write_robots_txt(&out, "https://b.example").unwrap(),
            "上一次自动生成的 robots.txt 应被新的 base_url 覆盖"
        );
        let updated = fs::read_to_string(&path).unwrap();
        assert!(updated.starts_with(ROBOTS_MARKER));
        assert!(updated.contains("Sitemap: https://b.example/sitemap.xml"));
        assert!(
            !updated.contains("https://a.example"),
            "旧地址应被清掉：{updated}"
        );
        let _ = fs::remove_dir_all(&base);
    }
}
