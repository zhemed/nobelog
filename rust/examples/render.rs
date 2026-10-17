#![allow(dead_code)]
// 单文件/全量渲染对照工具：输出 JSONL（file/html/toc），与 examples/oracle.mjs 逐字节对比。
// 用法：
//   cargo run --example render            # 全部 content/posts/*.md + content/pages/*.md
//   cargo run --example render -- content/posts/x.md
#[path = "../src/markdown.rs"]
mod markdown;

#[path = "../src/model.rs"]
mod model;

use std::path::{Path, PathBuf};

fn strip_front(raw: &str) -> &str {
    let Some(rest) = raw.strip_prefix("---") else {
        return raw;
    };
    let rest = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))
        .unwrap_or(rest);
    if let Some(idx) = rest.find("\n---") {
        let after = &rest[idx + 4..];
        after.strip_prefix('\n').unwrap_or(after)
    } else {
        raw
    }
}

fn collect(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for (dir, exts) in [
        ("content/posts", &["md", "mdx"][..]),
        ("content/pages", &["md"][..]),
    ] {
        let mut names: Vec<String> = std::fs::read_dir(root.join(dir))
            .unwrap_or_else(|_| panic!("read {dir}"))
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|ext| exts.contains(&ext))
            })
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        for name in names {
            files.push(format!("{dir}/{name}"));
        }
    }
    files
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate 在仓库根下")
        .to_path_buf();
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let shared = args.first().is_some_and(|a| a == "--shared");
    if shared {
        args.remove(0);
        markdown::reset_shared();
    }
    let files = if args.is_empty() {
        collect(&root)
    } else {
        args
    };

    for rel in files {
        let raw = std::fs::read_to_string(root.join(&rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let body = strip_front(&raw).trim();
        let (html, toc) = if shared && rel.starts_with("content/posts/") {
            markdown::render_shared(body)
        } else {
            markdown::render(body)
        };
        let toc: Vec<_> = toc
            .iter()
            .map(|t| serde_json::json!({"level": t.level, "id": t.id, "text": t.text}))
            .collect();
        let out = serde_json::json!({"file": rel, "html": html, "toc": toc});
        println!("{}", serde_json::to_string(&out).unwrap());
    }
    let (hits, misses) = markdown::highlight::stats();
    eprintln!("highlight golden: {hits} hits, {misses} misses");
}
