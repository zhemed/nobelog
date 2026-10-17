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
        let file = file_for_route(out, &route.path)?;
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file, route.html.as_bytes())?;
        html_files += 1;
        bytes += route.html.len();
    }

    // 静态托管的 404 约定文件
    let ctx = crate::templates::Ctx {
        assets: &assets,
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
fn file_for_route(out: &Path, pathname: &str) -> Result<PathBuf> {
    let decoded = percent_encoding::percent_decode_str(pathname)
        .decode_utf8_lossy()
        .to_string();
    let decoded = if decoded.contains('\0') {
        return Err(anyhow::anyhow!("非法路径：{pathname}"));
    } else {
        decoded
    };
    if decoded == "/" {
        return Ok(out.join("index.html"));
    }
    let rel = decoded.trim_start_matches('/');
    if decoded.ends_with('/') {
        Ok(out.join(rel).join("index.html"))
    } else {
        Ok(out.join(rel))
    }
}
