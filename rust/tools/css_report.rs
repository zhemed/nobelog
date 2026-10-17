// rust/tools/css_report.rs
//
// 离线 CSS 裁剪报告工具（纯 std，无 Cargo 依赖）。
//
// 编译并运行：
//   rustc --edition 2024 -O rust/tools/css_report.rs -o rust/target/css-report
//   rust/target/css-report --root . --css rust/target/css-baseline-dist/assets/main.<hash>.css
//
// 产出：
//   - rust/target/css-pruned.css        裁剪后的 CSS（供预览/对比）
//   - rust/target/css-prune-report.txt  完整报告（统计 + 被删除选择器清单及原因）
//
// 选项：
//   --root <dir>     仓库根目录（默认自动探测当前目录/上级）
//   --css <file>     未裁剪的 CSS（默认取 rust/dist/assets/main.*.css 最新一个）
//   --out <file>     裁剪后 CSS 输出路径
//   --report <file>  报告输出路径

#[path = "css_prune.rs"]
mod css_prune;

use css_prune::{Usage, collect_html_usage, collect_js_usage, prune_css};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() {
    let mut root = PathBuf::from(".");
    let mut css_path: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    let mut report_path: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => {
                if let Some(value) = args.next() {
                    root = PathBuf::from(value);
                }
            }
            "--css" => css_path = args.next().map(PathBuf::from),
            "--out" => out_path = args.next().map(PathBuf::from),
            "--report" => report_path = args.next().map(PathBuf::from),
            other => {
                eprintln!("未知参数：{other}");
                eprintln!(
                    "用法：css-report [--root DIR] [--css FILE] [--out FILE] [--report FILE]"
                );
                std::process::exit(2);
            }
        }
    }

    if !root.join("public").is_dir() && root.join("../public").is_dir() {
        root = root.join("..");
    }
    let root = root.canonicalize().unwrap_or(root);
    let public = root.join("public");
    if !public.is_dir() {
        eprintln!("找不到 public/：{}", public.display());
        std::process::exit(1);
    }

    let css_path = css_path.or_else(|| default_css(&root)).unwrap_or_else(|| {
        eprintln!("找不到 rust/dist/assets/main.*.css，请用 --css 指定输入");
        std::process::exit(1);
    });
    let out_path = out_path.unwrap_or_else(|| root.join("rust/target/css-pruned.css"));
    let report_path = report_path.unwrap_or_else(|| root.join("rust/target/css-prune-report.txt"));

    let css = fs::read_to_string(&css_path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", css_path.display()));

    let mut usage = Usage::new();
    let mut html_files = 0usize;
    let mut html_paths = Vec::new();
    walk_html(&public, &mut html_paths);
    for path in &html_paths {
        if let Ok(html) = fs::read_to_string(path) {
            collect_html_usage(&html, &mut usage);
            html_files += 1;
        }
    }
    let js_path = root.join("rust/client/main.js");
    let js = fs::read_to_string(&js_path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", js_path.display()));
    collect_js_usage(&js, &mut usage);
    usage.add_builtin_dynamic();

    let result = prune_css(&css, &usage, true);
    fs::write(&out_path, result.css.as_bytes())
        .unwrap_or_else(|error| panic!("写入 {} 失败：{error}", out_path.display()));
    if let Some(parent) = report_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let (classes, ids, tags) = usage.counts();
    let input_gzip = gzip_size(css.as_bytes());
    let output_gzip = gzip_size(result.css.as_bytes());
    let report = &result.report;

    let mut lines = Vec::new();
    lines.push("# CSS prune report".to_string());
    lines.push(String::new());
    lines.push(format!("root: {}", root.display()));
    lines.push(format!("css:  {}", css_path.display()));
    lines.push(format!(
        "html: {} files under {}",
        html_files,
        public.display()
    ));
    lines.push(format!("js:   {}", js_path.display()));
    lines.push(format!("usage: classes={classes} ids={ids} tags={tags}"));
    lines.push(String::new());
    lines.push(format!(
        "input:  {} bytes, gzip {}",
        report.input_bytes,
        fmt_opt(input_gzip)
    ));
    lines.push(format!(
        "output: {} bytes, gzip {}",
        report.output_bytes,
        fmt_opt(output_gzip)
    ));
    lines.push(format!(
        "selectors: kept {} / {} , removed {}",
        report.selectors_kept, report.selectors_total, report.selectors_removed
    ));
    lines.push(format!(
        "rules: kept {} / {}",
        report.rules_kept, report.rules_total
    ));
    lines.push(String::new());
    lines.push(format!("## removed selectors ({})", report.removed.len()));
    for removed in &report.removed {
        lines.push(format!(
            "{}  [{}]  ({} B)",
            removed.selector, removed.reason, removed.bytes
        ));
    }
    let report_text = lines.join("\n") + "\n";
    fs::write(&report_path, report_text.as_bytes())
        .unwrap_or_else(|error| panic!("写入 {} 失败：{error}", report_path.display()));

    println!("CSS prune report");
    println!(
        "  input : {:>8} B  gzip {}",
        report.input_bytes,
        fmt_opt(input_gzip)
    );
    println!(
        "  output: {:>8} B  gzip {}",
        report.output_bytes,
        fmt_opt(output_gzip)
    );
    println!("  {}", report.summary());
    println!(
        "  selectors kept {}/{} , removed {} ; rules kept {}/{}",
        report.selectors_kept,
        report.selectors_total,
        report.selectors_removed,
        report.rules_kept,
        report.rules_total
    );
    println!("  usage: classes={classes} ids={ids} tags={tags} (from {html_files} HTML files)");
    println!("  pruned css: {}", out_path.display());
    println!("  report    : {}", report_path.display());
    if let (Some(before), Some(after)) = (input_gzip, output_gzip) {
        println!(
            "  gzip saved: {} B ({:.1}%)",
            before.saturating_sub(after),
            if before == 0 {
                0.0
            } else {
                (before as f64 - after as f64) * 100.0 / before as f64
            }
        );
    }
    println!("  removed sample:");
    for removed in report.removed.iter().take(8) {
        println!(
            "    {}  [{}]",
            truncate(&removed.selector, 110),
            removed.reason
        );
    }
}

fn default_css(root: &Path) -> Option<PathBuf> {
    let dir = root.join("rust/dist/assets");
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(&dir).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("main.") && name.ends_with(".css") {
            let modified = entry
                .metadata()
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            if best.as_ref().is_none_or(|(time, _)| modified > *time) {
                best = Some((modified, entry.path()));
            }
        }
    }
    best.map(|(_, path)| path)
}

fn walk_html(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_html(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "html") {
            out.push(path);
        }
    }
    out.sort();
}

fn gzip_size(bytes: &[u8]) -> Option<usize> {
    let mut child = Command::new("gzip")
        .args(["-9", "-c"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    {
        let mut stdin = child.stdin.take()?;
        stdin.write_all(bytes).ok()?;
    }
    let output = child.wait_with_output().ok()?;
    output.status.success().then_some(output.stdout.len())
}

fn fmt_opt(value: Option<usize>) -> String {
    value.map_or_else(|| "n/a".to_string(), |bytes| format!("{bytes} B"))
}

fn truncate(text: &str, max_chars: usize) -> String {
    let mut out: String = text.chars().take(max_chars).collect();
    if text.chars().count() > max_chars {
        out.push('…');
    }
    out
}
