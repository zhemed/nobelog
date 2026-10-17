// 资源构建（阶段二）：让 Rust 自建 assets，不再复用 Node 的 public/assets。
//
// 产出（写到 `out/`）：
//   - assets/main.<sha256 前 8 位>.css  —— STYLE_ORDER 顺序拼接后压缩
//   - assets/main.<sha256 前 8 位>.js   —— rust/client/main.js 最小化
//   - assets-manifest.json              —— { "css": "/assets/main.xxx.css", "js": "/assets/main.xxx.js" }
//   - assets/fa-*.woff2、assets/lato-*.woff2 … —— vendored 字体，文件名保持不变（主题 CSS 写死了路径）
//
// 哈希算法与命名对齐 server/build.ts 的 contentHash：sha256(内容) 的十六进制前 8 位。
// CSS/JS 的压缩器是自写的轻量实现（不追 esbuild 逐字节一致，只要求视觉等价）。
//
// CSS 裁剪（阶段三）：压缩后再按 public/**/*.html + rust/client/main.js 的实际使用情况
// 保守删除未使用选择器（实现见 rust/tools/css_prune.rs）。默认开启；
// `NOBELOG_CSS_PRUNE=0`（或 off/false/no）可回退到未裁剪版本，
// `NOBELOG_CSS_REPORT=1` 会在构建时打印一行裁剪统计。

use crate::paths::{STYLE_ORDER, VENDORED_ASSETS};
use crate::templates::PageAssets;
use anyhow::{Context as _, Result, bail};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../tools/css_prune.rs"]
mod css_prune;

/// 运行时（动态模式）用的资源包：全部在内存里，不落盘。
pub struct BuiltAssets {
    pub assets: PageAssets,
    pub css: String,
    pub js: String,
    pub manifest: String,
}

/// 构建 CSS/JS/manifest，但不写盘。动态模式由 serve 直接持有。
pub fn build_in_memory(root: &Path) -> Result<BuiltAssets> {
    let css = build_css(root)?;
    let js = build_js(root)?;
    let css_name = format!("main.{}.css", content_hash(&css));
    let js_name = format!("main.{}.js", content_hash(&js));
    let assets = PageAssets {
        css: format!("/assets/{css_name}"),
        js: format!("/assets/{js_name}"),
    };
    let manifest = format!(
        "{}\n",
        serde_json::to_string_pretty(&Manifest {
            css: &assets.css,
            js: &assets.js
        })?
    );
    Ok(BuiltAssets {
        assets,
        css,
        js,
        manifest,
    })
}

#[derive(serde::Serialize)]
struct Manifest<'a> {
    css: &'a str,
    js: &'a str,
}

/// 构建全部资源，返回模板层需要的两个 URL。
pub fn build_assets(root: &Path, out: &Path) -> Result<PageAssets> {
    let assets_dir = out.join("assets");
    fs::create_dir_all(&assets_dir)
        .with_context(|| format!("无法创建资源目录：{}", assets_dir.display()))?;

    let css = build_css(root)?;
    let js = build_js(root)?;
    let css_name = format!("main.{}.css", content_hash(&css));
    let js_name = format!("main.{}.js", content_hash(&js));
    write_atomic(&assets_dir.join(&css_name), css.as_bytes())?;
    write_atomic(&assets_dir.join(&js_name), js.as_bytes())?;

    let assets = PageAssets {
        css: format!("/assets/{css_name}"),
        js: format!("/assets/{js_name}"),
    };
    let manifest = Manifest {
        css: &assets.css,
        js: &assets.js,
    };
    let manifest_raw = format!("{}\n", serde_json::to_string_pretty(&manifest)?);
    write_atomic(&out.join("assets-manifest.json"), manifest_raw.as_bytes())?;

    copy_vendored_assets(root, &assets_dir)?;
    Ok(assets)
}

/// 按 STYLE_ORDER 拼接样式并压缩；默认再用 HTML/JS 使用情况做保守裁剪。
fn build_css(root: &Path) -> Result<String> {
    let mut raw = String::new();
    for name in STYLE_ORDER {
        let file = root.join(name);
        let text = fs::read_to_string(&file).with_context(|| {
            format!(
                "缺少样式文件：{}\n  提示：主题样式由 node scripts/fetch-theme-assets.mjs 入库（vendor/chirpy/main.css）",
                file.display()
            )
        })?;
        raw.push_str(&text);
        raw.push('\n');
    }
    let css = minify_css(&raw);
    if !css_prune_enabled() {
        return Ok(css);
    }
    let Some(usage) = collect_css_usage(root)? else {
        eprintln!("css prune: 没有可用的 public HTML 或 rust/src 源码，跳过裁剪（输出未裁剪 CSS）");
        return Ok(css);
    };
    let result = css_prune::prune_css(&css, &usage, false);
    if std::env::var_os("NOBELOG_CSS_REPORT").is_some() {
        let (classes, ids, tags) = usage.counts();
        eprintln!(
            "css prune: {} ; usage classes={classes} ids={ids} tags={tags}",
            result.report.summary()
        );
    }
    Ok(result.css)
}

/// CSS 裁剪开关：默认开启，`NOBELOG_CSS_PRUNE=0|off|false|no` 关闭。
fn css_prune_enabled() -> bool {
    match std::env::var("NOBELOG_CSS_PRUNE") {
        Ok(value) => !matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "off" | "false" | "no"
        ),
        Err(_) => true,
    }
}

/// 收集 CSS 裁剪的使用情况：优先 `public/**/*.html`（最准确），
/// 没有 HTML 时回退到 `rust/src/**/*.rs` 源码字面量。
///
/// 静态构建可能 `--out public`（先清空 public），动态部署也可能根本没有 public，
/// 这两种情况都靠源码兜底；两处都不可用时返回 None，调用方保守跳过裁剪。
fn collect_css_usage(root: &Path) -> Result<Option<css_prune::Usage>> {
    let mut usage = css_prune::Usage::new();
    let public = root.join("public");
    let mut html_files = 0usize;
    if public.is_dir() {
        for entry in walkdir::WalkDir::new(&public) {
            let entry = entry.with_context(|| format!("遍历 {} 失败", public.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "html") {
                let html = fs::read_to_string(path)
                    .with_context(|| format!("读取 HTML 失败：{}", path.display()))?;
                css_prune::collect_html_usage(&html, &mut usage);
                html_files += 1;
            }
        }
    }

    if html_files == 0 {
        let source_dir = root.join("rust/src");
        let mut source_files = 0usize;
        if source_dir.is_dir() {
            for entry in walkdir::WalkDir::new(&source_dir) {
                let entry = entry.with_context(|| format!("遍历 {} 失败", source_dir.display()))?;
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "rs") {
                    let source = fs::read_to_string(path)
                        .with_context(|| format!("读取源码失败：{}", path.display()))?;
                    css_prune::collect_rust_source_usage(&source, &mut usage);
                    source_files += 1;
                }
            }
        }
        if source_files == 0 {
            return Ok(None);
        }
    }

    let js_path = root.join("rust/client/main.js");
    let js = fs::read_to_string(&js_path)
        .with_context(|| format!("读取客户端脚本失败：{}", js_path.display()))?;
    css_prune::collect_js_usage(&js, &mut usage);
    css_prune::Usage::add_builtin_dynamic(&mut usage);
    Ok(Some(usage))
}

/// 读取直译后的客户端脚本并最小化。
fn build_js(root: &Path) -> Result<String> {
    let file = root.join("rust/client/main.js");
    let source = fs::read_to_string(&file).with_context(|| {
        format!(
            "缺少客户端脚本：{}\n  提示：源文件是 rust/client/main.js",
            file.display()
        )
    })?;
    Ok(minify_js(&source))
}

/// 8 位内容哈希：文件名即版本，配合一年强缓存。
fn content_hash(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    hex::encode(digest)[..8].to_string()
}

/// 原子写盘：先写 `<target>.tmp-<pid>` 再 rename，磁盘上不会出现半截文件。
fn write_atomic(target: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut tmp = target.as_os_str().to_os_string();
    tmp.push(format!(".tmp-{}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, data).with_context(|| format!("写入失败：{}", tmp.display()))?;
    fs::rename(&tmp, target).with_context(|| format!("重命名失败：{}", target.display()))?;
    Ok(())
}

/// 复制 vendored 字体：FA 四件套（vendor/chirpy/）+ 自托管拉丁字体（vendor/fonts/*.woff2）。
/// 这些文件名被 CSS 的 `url(/assets/...)` 写死，不能加内容哈希。
fn copy_vendored_assets(root: &Path, assets_dir: &Path) -> Result<()> {
    let vendor_dir = root.join("vendor/chirpy");
    for name in VENDORED_ASSETS {
        let file = vendor_dir.join(name);
        let data = fs::read(&file).with_context(|| {
            format!(
                "缺少 vendored 资产：{}\n  提示：先运行 node scripts/fetch-theme-assets.mjs",
                file.display()
            )
        })?;
        write_atomic(&assets_dir.join(name), &data)?;
    }

    let font_dir = root.join("vendor/fonts");
    if !font_dir.is_dir() {
        bail!(
            "缺少自托管字体目录：{}\n  提示：先运行 node scripts/fetch-fonts.mjs",
            font_dir.display()
        );
    }
    let mut fonts = Vec::new();
    for entry in fs::read_dir(&font_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".woff2") {
            fonts.push(name.into_owned());
        }
    }
    fonts.sort();
    if fonts.is_empty() {
        bail!(
            "{} 下没有 woff2 文件\n  提示：先运行 node scripts/fetch-fonts.mjs",
            font_dir.display()
        );
    }
    for name in fonts {
        let file = font_dir.join(&name);
        let data = fs::read(&file).with_context(|| format!("读取字体失败：{}", file.display()))?;
        write_atomic(&assets_dir.join(&name), &data)?;
    }
    Ok(())
}

/* -------------------------------------------------------------------------
 * CSS 压缩
 *
 * 保守策略：去掉注释（保留以 `!` 开头的版权注释）与多余空白、去掉块内最后一个分号；
 * 不改变选择器、声明顺序与任何 token 内容。字符串与 url(...) 原样保留，
 * `a :hover` 这类「后代 + 伪类」的空格必须保留，所以空格只在这些位置删除：
 *   - 紧邻 { } ; , 的前后
 *   - 冒号之后（声明值前；选择器里的冒号前空格不动）
 * calc() 里 + - 两侧的空格、选择器里的后代空格都保持原样。
 * ---------------------------------------------------------------------- */

fn minify_css(source: &str) -> String {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len);
    let mut i = 0usize;
    let mut pending_space = false;
    // 自定义属性（--foo: value）冒号后的空格是值 token 流的一部分，不能删。
    let mut custom_property = false;
    let mut at_decl_start = false;
    let mut protect_colon_space = false;

    while i < len {
        let b = bytes[i];

        // 注释：`/*!` 开头的是版权注释，与 esbuild 默认 legalComments=inline 一致地保留
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            let legal = i + 2 < len && bytes[i + 2] == b'!';
            let start = i;
            let mut j = i + 2;
            while j + 1 < len && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                j += 1;
            }
            let end = if j + 1 < len { j + 2 } else { len };
            if legal {
                flush_css_space(
                    &mut out,
                    &mut pending_space,
                    '/',
                    protect_colon_space,
                    custom_property,
                );
                out.push_str(&source[start..end]);
                protect_colon_space = false;
            } else {
                pending_space = true;
            }
            i = end;
            continue;
        }

        // 字符串：整段原样复制（content: "{"、属性选择器里的值等）
        if b == b'"' || b == b'\'' {
            flush_css_space(
                &mut out,
                &mut pending_space,
                b as char,
                protect_colon_space,
                custom_property,
            );
            let quote = b;
            let start = i;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let end = i.min(len);
            out.push_str(&source[start..end]);
            at_decl_start = false;
            protect_colon_space = false;
            continue;
        }

        // 反斜杠转义：`.\31 23` 里的空格是转义的一部分，整对原样保留
        if b == b'\\' {
            flush_css_space(
                &mut out,
                &mut pending_space,
                '\\',
                protect_colon_space,
                custom_property,
            );
            out.push('\\');
            i += 1;
            if i < len {
                let ch = source[i..].chars().next().unwrap_or('\u{fffd}');
                out.push(ch);
                i += ch.len_utf8();
            }
            at_decl_start = false;
            protect_colon_space = false;
            continue;
        }

        // url(...)：data URI 里可能有逗号、分号、空格、引号与括号转义，整体原样保留
        if (b == b'u' || b == b'U')
            && i + 3 < len
            && bytes[i + 1].eq_ignore_ascii_case(&b'r')
            && bytes[i + 2].eq_ignore_ascii_case(&b'l')
            && bytes[i + 3] == b'('
        {
            flush_css_space(
                &mut out,
                &mut pending_space,
                'u',
                protect_colon_space,
                custom_property,
            );
            let start = i;
            i += 4;
            while i < len {
                match bytes[i] {
                    b'\\' => i += 2,
                    quote @ (b'"' | b'\'') => {
                        i += 1;
                        while i < len {
                            if bytes[i] == b'\\' {
                                i += 2;
                                continue;
                            }
                            if bytes[i] == quote {
                                i += 1;
                                break;
                            }
                            i += 1;
                        }
                    }
                    b')' => {
                        i += 1;
                        break;
                    }
                    _ => i += 1,
                }
            }
            let end = i.min(len);
            out.push_str(&source[start..end]);
            at_decl_start = false;
            protect_colon_space = false;
            continue;
        }

        if b.is_ascii_whitespace() {
            pending_space = true;
            i += 1;
            continue;
        }

        let ch = source[i..].chars().next().unwrap_or('\u{fffd}');

        // 声明块里第一条 token 以 -- 开头 → 这是自定义属性，冒号后的空格要保留
        if at_decl_start && ch == '-' && i + 1 < len && bytes[i + 1] == b'-' {
            custom_property = true;
        }
        at_decl_start = false;

        // 块内最后一条声明的分号可以省掉：`...;}` → `...}`
        if ch == ';' {
            let mut look = i + 1;
            while look < len && bytes[look].is_ascii_whitespace() {
                look += 1;
            }
            if look < len && bytes[look] == b'}' {
                pending_space = false;
                custom_property = false;
                protect_colon_space = false;
                at_decl_start = true;
                i += 1;
                continue;
            }
        }

        flush_css_space(
            &mut out,
            &mut pending_space,
            ch,
            protect_colon_space,
            custom_property,
        );
        out.push(ch);
        if ch == ':' {
            protect_colon_space = custom_property;
        } else {
            protect_colon_space = false;
        }
        if matches!(ch, ';' | '{' | '}') {
            custom_property = false;
            at_decl_start = true;
        }
        i += ch.len_utf8();
    }

    out.trim_end().to_string()
}

/// 在输出下一个 token 前决定是否落一个空格。
fn flush_css_space(
    out: &mut String,
    pending: &mut bool,
    next: char,
    protect_colon_space: bool,
    custom_property: bool,
) {
    if !*pending {
        return;
    }
    *pending = false;
    let Some(last) = out.chars().next_back() else {
        return;
    };
    // 自定义属性的值是 token 流，内部空白（逗号后、值中间）保持原样，只把连续空白压成一个空格。
    let drop_after = !custom_property
        && (matches!(last, '{' | '}' | ';' | ',') || (last == ':' && !protect_colon_space));
    let drop_before = !custom_property && matches!(next, '{' | '}' | ';' | ',');
    if !drop_after && !drop_before && !last.is_whitespace() {
        out.push(' ');
    }
}

/* -------------------------------------------------------------------------
 * JS 最小化
 *
 * 只做安全变换：去掉注释与行首行尾缩进、合并空白、删除空行；换行保留（ASI 语义不变）。
 * 字符串 / 模板字符串原样保留；本项目源码没有正则字面量里的注释序列。
 * ---------------------------------------------------------------------- */

fn minify_js(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut pending_space = false;

    while let Some(ch) = chars.next() {
        if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            while let Some(&next) = chars.peek() {
                if next == '\n' {
                    break;
                }
                chars.next();
            }
            pending_space = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = '\0';
            let mut has_newline = false;
            for next in chars.by_ref() {
                if prev == '*' && next == '/' {
                    break;
                }
                if next == '\n' {
                    has_newline = true;
                }
                prev = next;
            }
            if has_newline {
                out.push('\n');
                pending_space = false;
            } else {
                pending_space = true;
            }
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            flush_js_space(&mut out, &mut pending_space, ch);
            out.push(ch);
            while let Some(next) = chars.next() {
                out.push(next);
                if next == '\\' {
                    if let Some(escaped) = chars.next() {
                        out.push(escaped);
                    }
                    continue;
                }
                if next == ch {
                    break;
                }
            }
            continue;
        }

        if ch.is_whitespace() {
            if ch == '\n' {
                out.push('\n');
                pending_space = false;
            } else {
                pending_space = true;
            }
            continue;
        }

        flush_js_space(&mut out, &mut pending_space, ch);
        out.push(ch);
    }

    out.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// 在 JS token 之间按需保留一个空格。
fn flush_js_space(out: &mut String, pending: &mut bool, next: char) {
    if !*pending {
        return;
    }
    *pending = false;
    let Some(last) = out.chars().next_back() else {
        return;
    };
    if !last.is_whitespace() && !matches!(next, ')' | ']' | '}' | ',' | ';' | '.') {
        out.push(' ');
    }
}
