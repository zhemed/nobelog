// nobelog-rs：Rust 重写的静态站生成器。
// 子命令：build（产物写 rust/dist）、diff（与 Node 冻结产物逐字节对比）。

mod assets;
mod build;
mod content;
mod markdown;
mod model;
mod paths;
mod routes;
mod serve;
mod site;
mod templates;
mod url;

use anyhow::Result;
use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "nobelog", version, about = "nobelog 静态博客生成器（Rust）")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 生成站点到 dist/（不触碰 public/）
    Build {
        /// 输出目录
        #[arg(long, default_value = "dist")]
        out: PathBuf,
        /// 仓库根目录（默认 rust/ 的上一级）
        #[arg(long)]
        root: Option<PathBuf>,
        /// 阶段二：由 Rust 自建 CSS/JS 资源（默认复用 public/assets 做逐字节对齐）
        #[arg(long)]
        self_assets: bool,
    },
    /// 本地静态服务（axum），行为对齐 server/serve.ts
    Serve {
        /// 监听地址
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// 端口
        #[arg(long, default_value_t = 10086)]
        port: u16,
        /// 仓库根目录（默认 rust/ 的上一级）
        #[arg(long)]
        root: Option<PathBuf>,
        /// 动态模式：不依赖 public/，运行时渲染 + 资源内存化 + 内容热更新
        #[arg(long)]
        dynamic: bool,
    },
    /// 与基准目录逐字节对比（feed 的 lastBuildDate 归一化）
    Diff {
        /// 基准目录（冻结产物）
        #[arg(default_value = "public")]
        base: PathBuf,
        /// 待测目录（Rust 产物）
        #[arg(default_value = "dist")]
        other: PathBuf,
        /// 资源哈希归一化（阶段二自建资源的哈希必然不同）
        #[arg(long)]
        normalize_assets: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build {
            out,
            root,
            self_assets,
        } => {
            let root = root.unwrap_or_else(default_root);
            build::build(&root, &out, self_assets)
        }
        Command::Serve {
            host,
            port,
            root,
            dynamic,
        } => {
            let root = root.unwrap_or_else(default_root);
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(serve::run(&root, &host, port, dynamic))
        }
        Command::Diff {
            base,
            other,
            normalize_assets,
        } => diff(&base, &other, normalize_assets),
    }
}

fn default_root() -> PathBuf {
    // 打包后的二进制不能依赖编译机路径：优先环境变量、当前目录、可执行文件所在目录，
    // 最后才退回开发机上的 CARGO_MANIFEST_DIR。
    if let Ok(root) = std::env::var("NOBELOG_ROOT") {
        let path = PathBuf::from(root);
        if path.join("vendor/chirpy/main.css").exists() {
            return path;
        }
    }
    if Path::new("vendor/chirpy/main.css").exists() {
        return PathBuf::from(".");
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && dir.join("vendor/chirpy/main.css").exists()
    {
        return dir.to_path_buf();
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn diff(base: &Path, other: &Path, normalize_assets: bool) -> Result<()> {
    let ts = Regex::new(r"<lastBuildDate>[^<]*</lastBuildDate>")?;
    let asset = Regex::new(r"/assets/main\.[0-9a-f]{8}\.(css|js)")?;
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(base) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.path().strip_prefix(base)?.to_path_buf());
        }
    }
    files.sort();

    let mut same = 0usize;
    let mut different = Vec::new();
    let mut missing = Vec::new();

    for rel in &files {
        if normalize_assets && asset.is_match(&rel.to_string_lossy()) {
            // 自建资源文件名/内容必然不同，按引用归一化处理
            continue;
        }
        let left = fs::read(base.join(rel))?;
        let right_path = other.join(rel);
        if !right_path.exists() {
            missing.push(rel.clone());
            continue;
        }
        let right = fs::read(&right_path)?;
        let mut left_cmp = normalize(&left, &ts);
        let mut right_cmp = normalize(&right, &ts);
        if normalize_assets {
            left_cmp = normalize_assets_refs(&left_cmp, &asset);
            right_cmp = normalize_assets_refs(&right_cmp, &asset);
        }
        if left_cmp == right_cmp {
            same += 1;
        } else {
            different.push((rel.clone(), left_cmp, right_cmp));
        }
    }

    let mut extra = Vec::new();
    for entry in walkdir::WalkDir::new(other) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(other)?.to_path_buf();
            if !base.join(&rel).exists() {
                extra.push(rel);
            }
        }
    }
    extra.sort();

    println!(
        "diff: {same} 相同 / {} 不同 / {} 缺失 / {} 多出",
        different.len(),
        missing.len(),
        extra.len()
    );
    for rel in &missing {
        println!("  缺失  {}", rel.display());
    }
    for rel in &extra {
        println!("  多出  {}", rel.display());
    }
    for (rel, left, right) in different.iter().take(10) {
        let (line, l, r) = first_diff(left, right);
        println!(
            "  不同  {}（首个差异在第 {} 行）\n        base : {}\n        rust : {}",
            rel.display(),
            line,
            l,
            r
        );
    }
    if !different.is_empty() && different.len() > 10 {
        println!("  … 另有 {} 个不同文件未展开", different.len() - 10);
    }
    if different.is_empty() && missing.is_empty() && extra.is_empty() {
        println!("✓ 逐字节一致");
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn normalize_assets_refs(bytes: &[u8], re: &Regex) -> Vec<u8> {
    match std::str::from_utf8(bytes) {
        Ok(text) => re
            .replace_all(text, "/assets/main.HASH.$1")
            .as_bytes()
            .to_vec(),
        Err(_) => bytes.to_vec(),
    }
}

fn normalize(bytes: &[u8], ts: &Regex) -> Vec<u8> {
    match std::str::from_utf8(bytes) {
        Ok(text) => ts.replace_all(text, "<lastBuildDate/>").as_bytes().to_vec(),
        Err(_) => bytes.to_vec(),
    }
}

fn first_diff(left: &[u8], right: &[u8]) -> (usize, String, String) {
    let left_text = String::from_utf8_lossy(left);
    let right_text = String::from_utf8_lossy(right);
    let left_lines: Vec<&str> = left_text.lines().collect();
    let right_lines: Vec<&str> = right_text.lines().collect();
    for i in 0..left_lines.len().max(right_lines.len()) {
        let l = left_lines.get(i).copied().unwrap_or("<EOF>");
        let r = right_lines.get(i).copied().unwrap_or("<EOF>");
        if l != r {
            return (i + 1, truncate(l), truncate(r));
        }
    }
    (1, "<相同？>".into(), "<相同？>".into())
}

fn truncate(text: &str) -> String {
    let mut out: String = text.chars().take(240).collect();
    if text.chars().count() > 240 {
        out.push('…');
    }
    out
}
