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
}
