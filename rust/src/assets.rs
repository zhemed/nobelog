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
// CSS 裁剪（阶段三）：压缩后按「源码字面量 + 正文原生 HTML + main.js + 动态白名单」的
// 使用情况保守删除未使用选择器（实现见 rust/tools/css_prune.rs）。默认开启；
// `NOBELOG_CSS_PRUNE=0`（或 off/false/no）可回退到未裁剪版本，
// `NOBELOG_CSS_REPORT=1` 会在构建时打印一行裁剪统计。
//
// 使用集合**不读 `public/**/*.html`**：静态构建（`build --self-assets --out public`）会先清空
// public，部署包也可能根本没有 public，而动态模式（`build_in_memory`）必须与静态构建产出同一份
// CSS。因此这里只用与产物位置无关、两种模式都能拿到的确定输入：
//   1. `rust/src` 里模板/渲染器的 class/id 字面量（编译期内嵌，部署包缺源码也一致）；
//   2. `content/**/*.md` 正文里直接写的原生 HTML class/id（两模式都读得到）；
//   3. `rust/client/main.js` 动态使用的选择器（缺文件时用编译期内嵌副本）；
//   4. 渲染器运行时才生成、且只在特定页面状态出现的类（hljs-*/prompt-*/分页/栅格等）白名单。
// 渲染出的路由 HTML 完全由上述源码与内容决定，所以资源层不需要自己加载页面内容。
// 回归测试（本文件 `tests`）用真实仓库断言：静态 build_assets 与动态 build_in_memory 的 CSS
// 内容与哈希一致，且源码使用集合覆盖真实渲染页面用到的选择器。

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

/// 按 STYLE_ORDER 拼接样式并压缩；默认再用源码/内容使用情况做保守裁剪。
fn build_css(root: &Path) -> Result<String> {
    let css = load_styles(root)?;
    if !css_prune_enabled() {
        return Ok(css);
    }
    let usage = collect_css_usage(root)?;
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

/// 按 STYLE_ORDER 拼接并压缩样式，不裁剪。
fn load_styles(root: &Path) -> Result<String> {
    let mut raw = String::new();
    for name in STYLE_ORDER {
        let file = root.join(name);
        let text = fs::read_to_string(&file).with_context(|| {
            format!(
                "缺少样式文件：{}\n  提示：仓库不完整或 vendor 资产未入库，请检查 vendor/ 是否完整（如 vendor/chirpy/main.css）",
                file.display()
            )
        })?;
        raw.push_str(&text);
        raw.push('\n');
    }
    Ok(minify_css(&raw))
}

/// CSS 裁剪开关：默认开启，`NOBELOG_CSS_PRUNE=0|off|false|no` 关闭。
fn css_prune_enabled() -> bool {
    css_prune_enabled_from(std::env::var("NOBELOG_CSS_PRUNE").ok().as_deref())
}

fn css_prune_enabled_from(value: Option<&str>) -> bool {
    match value {
        Some(value) => !matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "off" | "false" | "no"
        ),
        None => true,
    }
}

/// 编译期内嵌的源码字面量。
///
/// 部署包只带 `content/`、`vendor/`、`public/` 与 `rust/client/main.js`，**不含 `rust/src`**；
/// 动态模式如果扫描不到源码就会退回未裁剪 CSS，与静态构建的产物又不一致。把含 class/id 的
/// 源码在编译期内嵌，保证「同一 commit 的二进制」在仓库和部署包里得到同一份使用集合。
///
/// 新增含 class/id 字面量的源码文件时要同步加进来，
/// `embedded_sources_cover_class_literals` 测试会提醒。
const EMBEDDED_SOURCES: &[&str] = &[
    include_str!("templates.rs"),
    include_str!("markdown.rs"),
    include_str!("site.rs"),
    include_str!("highlight.rs"),
    include_str!("highlight_map.rs"),
];

/// 客户端脚本兜底副本（部署包需要它，本地开发时同名文件优先）。
const EMBEDDED_CLIENT_JS: &str = include_str!("../client/main.js");

/// 收集 CSS 裁剪的使用情况：源码字面量 + 正文原生 HTML + main.js + 动态白名单。
///
/// 这里刻意**不读 `public/**/*.html`**：public 在 `build --out public` 时会先被清空，
/// 部署包也可能没有 public；一旦依赖它，静态构建与动态模式就会产出两套 CSS。
/// 渲染页面由源码 + 内容唯一决定，所以源码/内容层面的使用集合足以覆盖页面实际用类
/// （回归测试 `source_usage_covers_rendered_pages` 会拿真实路由 HTML 交叉验证）。
fn collect_css_usage(root: &Path) -> Result<css_prune::Usage> {
    let mut usage = css_prune::Usage::new();
    for source in EMBEDDED_SOURCES {
        css_prune::collect_rust_source_usage(source, &mut usage);
    }
    collect_content_usage(root, &mut usage)?;

    let js_path = root.join("rust/client/main.js");
    match fs::read_to_string(&js_path) {
        Ok(js) => css_prune::collect_js_usage(&js, &mut usage),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            css_prune::collect_js_usage(EMBEDDED_CLIENT_JS, &mut usage);
        }
        Err(err) => {
            return Err(err).with_context(|| format!("读取客户端脚本失败：{}", js_path.display()));
        }
    }
    css_prune::Usage::add_builtin_dynamic(&mut usage);
    Ok(usage)
}

/// 正文里的原生 HTML（`<div class="...">`）也算使用情况。
///
/// Markdown 渲染器自己生成的类由 `add_builtin_dynamic` 的白名单覆盖；正文手写的
/// class/id 只有内容层知道，静态构建与动态模式读同一份 `content/`，所以不会破坏确定性。
/// 文章扩展名与 `content::read_posts` 保持一致：`.md` 与 `.mdx` 都要扫
/// （漏掉 `.mdx` 会让正文里的 class 被当成未使用而裁掉）。
/// 代码块里的示例 HTML 会被一并收进来——多留样式无害，漏删才会出问题。
fn collect_content_usage(root: &Path, usage: &mut css_prune::Usage) -> Result<()> {
    let content_dir = root.join("content");
    if !content_dir.is_dir() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(&content_dir) {
        let entry = entry.with_context(|| format!("遍历 {} 失败", content_dir.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("md" | "mdx")
        ) {
            let markdown = fs::read_to_string(path)
                .with_context(|| format!("读取 Markdown 失败：{}", path.display()))?;
            css_prune::collect_markdown_source_usage(&markdown, usage);
        }
    }
    Ok(())
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
                "缺少 vendored 资产：{}\n  提示：仓库不完整或 vendor 资产未入库，请检查 vendor/ 是否完整",
                file.display()
            )
        })?;
        write_atomic(&assets_dir.join(name), &data)?;
    }

    let font_dir = root.join("vendor/fonts");
    if !font_dir.is_dir() {
        bail!(
            "缺少自托管字体目录：{}\n  提示：仓库不完整或 vendor 资产未入库，请检查 vendor/fonts/ 是否完整",
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
            "{} 下没有 woff2 文件\n  提示：仓库不完整或 vendor 资产未入库，请检查 vendor/fonts/ 是否完整",
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    /// 仓库根（`rust/` 的上一级），与 `main.rs::default_root` 一致。
    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("rust/ 应有上一级目录")
            .to_path_buf()
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "nobelog-assets-test-{}-{tag}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn prune_toggle_keeps_env_semantics() {
        for off in ["0", "off", "false", "no", " OFF ", "No"] {
            assert!(!css_prune_enabled_from(Some(off)), "{off:?} 应关闭裁剪");
        }
        for on in ["1", "on", "true", "yes", "", "anything"] {
            assert!(css_prune_enabled_from(Some(on)), "{on:?} 应保持裁剪");
        }
        assert!(css_prune_enabled_from(None));
    }

    /// 使用集合只由源码/内容决定：`public/` 里有什么都不影响结果。
    #[test]
    fn css_usage_ignores_public_html() {
        let root = temp_dir("public-only");
        fs::create_dir_all(root.join("public")).unwrap();
        fs::write(
            root.join("public/index.html"),
            r##"<div class="only-in-public" id="only-in-public-id"></div>"##,
        )
        .unwrap();

        let usage = collect_css_usage(&root).unwrap();
        assert!(
            !usage.has_class("only-in-public"),
            "public HTML 不应再被当作使用情况来源"
        );
        assert!(!usage.has_id("only-in-public-id"));
        // 没有 rust/src、没有 main.js 时走编译期内嵌副本，仍然能拿到模板里的类。
        assert!(usage.has_class("pagination"));
        assert!(usage.has_class("preview-img"));
        let _ = fs::remove_dir_all(&root);
    }

    /// 文章扩展名是 `.md` / `.mdx`（见 `content::read_posts`），正文扫描必须都覆盖：
    /// `.mdx` 里的原生 HTML class 要进入 usage，不能被 CSS 裁剪丢掉。
    #[test]
    fn content_source_scan_includes_mdx() {
        let root = temp_dir("mdx");
        fs::create_dir_all(root.join("content/posts")).unwrap();
        fs::write(
            root.join("content/posts/with-html.md"),
            r##"<div class="md-only-widget">md</div>"##,
        )
        .unwrap();
        fs::write(
            root.join("content/posts/with-html.mdx"),
            r##"<div class="mdx-only-widget" id="mdx-anchor">mdx</div>"##,
        )
        .unwrap();

        let usage = collect_css_usage(&root).unwrap();
        assert!(
            usage.has_class("md-only-widget"),
            ".md 正文的 class 应进入 usage"
        );
        assert!(
            usage.has_class("mdx-only-widget"),
            ".mdx 正文的 class 应进入 usage"
        );
        assert!(usage.has_id("mdx-anchor"));

        let css = ".mdx-only-widget{color:red}.totally-unused-xyz{color:red}";
        let pruned = css_prune::prune_css(css, &usage, false).css;
        assert!(
            pruned.contains(".mdx-only-widget{"),
            ".mdx 里的 class 不应被裁掉：{pruned}"
        );
        assert!(!pruned.contains("totally-unused-xyz"));
        let _ = fs::remove_dir_all(&root);
    }

    /// 静态构建（build_assets）与动态模式（build_in_memory）必须产出同一份 CSS。
    #[test]
    fn static_and_dynamic_build_share_css() {
        let root = repo_root();
        let out = temp_dir("static-out");

        let dynamic = build_in_memory(&root).expect("动态模式资源应能构建");
        let static_assets = build_assets(&root, &out).expect("静态资源应能构建");

        assert_eq!(
            dynamic.assets.css, static_assets.css,
            "两条路径的 CSS 文件名（内容哈希）必须一致"
        );
        assert_eq!(dynamic.assets.js, static_assets.js);
        let css_name = dynamic
            .assets
            .css
            .strip_prefix("/assets/")
            .expect("CSS URL 应以 /assets/ 开头");
        assert_eq!(
            css_name,
            format!("main.{}.css", content_hash(&dynamic.css)),
            "文件名必须是内容哈希"
        );
        let on_disk = fs::read_to_string(out.join("assets").join(css_name)).unwrap();
        assert_eq!(on_disk, dynamic.css, "静态写盘内容应与内存产物一致");
        let manifest = fs::read_to_string(out.join("assets-manifest.json")).unwrap();
        assert!(manifest.contains(&dynamic.assets.css));
        assert!(manifest.contains(&dynamic.assets.js));

        let _ = fs::remove_dir_all(&out);
    }

    /// 条件渲染类必须被白名单保住，否则会随页面状态时有时无。
    #[test]
    fn conditional_classes_survive_pruning() {
        let root = repo_root();
        let usage = collect_css_usage(&root).unwrap();
        let css = concat!(
            ".pagination{color:red}",
            ".page-item{color:red}",
            ".page-link{color:red}",
            ".row-cols-md-2>*{color:red}",
            ".preview-img{color:red}",
            ".totally-unused-xyz{color:red}",
        );
        let pruned = css_prune::prune_css(css, &usage, false).css;
        for kept in [
            ".pagination{",
            ".page-item{",
            ".page-link{",
            ".row-cols-md-2>*{",
            ".preview-img{",
        ] {
            assert!(pruned.contains(kept), "{kept} 不应被裁掉：{pruned}");
        }
        assert!(
            !pruned.contains("totally-unused-xyz"),
            "真正没用的选择器仍应被裁掉：{pruned}"
        );
    }

    /// 源码使用集合必须覆盖真实渲染出的页面：页面里用到的选择器一条都不能被误裁。
    ///
    /// 这是 `collect_css_usage` 不读 public HTML 的依据——路由 HTML 完全由源码 + 内容决定。
    #[test]
    fn source_usage_covers_rendered_pages() {
        let root = repo_root();
        let site = crate::routes::Site::new(&root).expect("仓库内容应能加载");
        let provisional = PageAssets {
            css: "/assets/main.provisional.css".to_string(),
            js: "/assets/main.provisional.js".to_string(),
        };
        let mut rendered = css_prune::Usage::new();
        let mut pages = 0usize;
        for route in site.all_routes(&provisional) {
            if route.status != 200 {
                continue;
            }
            if !(route.path.ends_with('/') || route.path.ends_with(".html")) {
                continue;
            }
            css_prune::collect_html_usage(&route.html, &mut rendered);
            pages += 1;
        }
        assert!(pages >= 2, "至少应渲染首页与文章页，实际 {pages} 个");
        assert!(rendered.has_class("post-preview") || rendered.has_class("post"));

        let source = collect_css_usage(&root).unwrap();
        let css = load_styles(&root).unwrap();
        let removed_by_source: HashSet<String> = css_prune::prune_css(&css, &source, true)
            .report
            .removed
            .into_iter()
            .map(|removed| removed.selector)
            .collect();
        let removed_by_pages: HashSet<String> = css_prune::prune_css(&css, &rendered, true)
            .report
            .removed
            .into_iter()
            .map(|removed| removed.selector)
            .collect();
        let mut missed: Vec<&String> = removed_by_source.difference(&removed_by_pages).collect();
        missed.sort();
        assert!(
            missed.is_empty(),
            "源码使用集合漏掉真实页面在用的选择器（会被误裁）：{missed:?}"
        );
    }

    /// 仓库里直接扫 `rust/src/**/*.rs` 得到的源码集合。
    fn runtime_source_usage() -> css_prune::Usage {
        let src = repo_root().join("rust/src");
        let mut usage = css_prune::Usage::new();
        for entry in fs::read_dir(&src).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_file() {
                continue;
            }
            if !entry.file_name().to_string_lossy().ends_with(".rs") {
                continue;
            }
            let text = fs::read_to_string(entry.path()).unwrap();
            css_prune::collect_rust_source_usage(&text, &mut usage);
        }
        usage
    }

    /// 部署包不含 `rust/src`，编译期内嵌集合必须与仓库源码集合裁出同一份 CSS。
    ///
    /// 断言的是裁剪结果而不是文件清单：新增源码文件里只出现在测试中的 class 字面量
    /// 不会误报；但真正被样式引用、却没加进 EMBEDDED_SOURCES 的类会立刻暴露。
    #[test]
    fn embedded_sources_match_runtime_sources() {
        let root = repo_root();
        let mut embedded = css_prune::Usage::new();
        for source in EMBEDDED_SOURCES {
            css_prune::collect_rust_source_usage(source, &mut embedded);
        }
        let runtime = runtime_source_usage();
        let css = load_styles(&root).unwrap();
        let embedded_css = css_prune::prune_css(&css, &embedded, false).css;
        let runtime_css = css_prune::prune_css(&css, &runtime, false).css;
        assert_eq!(
            embedded_css, runtime_css,
            "内嵌源码集合与仓库 rust/src 的裁剪结果不一致：新增的 class 字面量需要加入 EMBEDDED_SOURCES"
        );
    }
}
