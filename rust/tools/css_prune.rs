// rust/tools/css_prune.rs
//
// CSS 选择器裁剪（assets 构建与离线报告工具共用，纯 std，无外部依赖）。
//
// 保守策略（宁可多留，不可误删）：
//   1. 从源码字面量（模板/渲染器）、正文原生 HTML、main.js 收集实际出现的 class / id / tag；
//      再叠加渲染器运行时才会生成、且只在特定页面状态出现的类/属性白名单。
//      刻意不依赖 `public/**/*.html`：静态构建会先清空 public，部署包也可能没有 public，
//      依赖它会让同一 commit 的静态构建与动态模式产出两套 CSS。
//   2. 选择器里只要有一个 class/id/tag 出现在收集结果中 → 保留整条选择器；
//      全部没出现 → 删除（任务要求"class/id 都未出现 → 删除"）。
//   3. 没有 class/id/tag token 的选择器（:root、*、::selection、[hidden] 等）→ 保留。
//   4. 含属性选择器 `[...]`、函数式伪类 `:not(...)` 等、或转义字符的选择器 → 保留。
//   5. @media/@supports/@layer/@container 递归裁剪，空了才丢；
//      @keyframes/@font-face/@page/@property 及未知 at-rule 整块保留。
//   6. :root 变量、伪类/伪元素都随所在规则保留。
//
// 该模块在 `rust/src/assets.rs` 中以 `#[path = "../tools/css_prune.rs"]` 引入，
// 也被 `rust/tools/css_report.rs` 独立编译使用。

use std::collections::HashSet;

#[derive(Default, Clone, Debug)]
pub struct Usage {
    classes: HashSet<String>,
    ids: HashSet<String>,
    tags: HashSet<String>,
    class_prefixes: Vec<String>,
}

impl Usage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_class(&mut self, name: &str) {
        let name = name.trim();
        if !name.is_empty() {
            self.classes.insert(name.to_string());
        }
    }

    /// 类名前缀白名单：用于 hljs-/rouge-/prompt- 这类由渲染器成批生成的类。
    pub fn add_class_prefix(&mut self, prefix: &str) {
        let prefix = prefix.trim();
        if !prefix.is_empty() {
            self.class_prefixes.push(prefix.to_string());
        }
    }

    pub fn add_id(&mut self, name: &str) {
        let name = name.trim();
        if !name.is_empty() {
            self.ids.insert(name.to_string());
        }
    }

    pub fn add_tag(&mut self, name: &str) {
        let name = name.trim().to_ascii_lowercase();
        if !name.is_empty() {
            self.tags.insert(name);
        }
    }

    pub fn has_class(&self, name: &str) -> bool {
        self.classes.contains(name)
            || self
                .class_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix))
    }

    pub fn has_id(&self, name: &str) -> bool {
        self.ids.contains(name)
    }

    pub fn has_tag(&self, name: &str) -> bool {
        self.tags.contains(&name.to_ascii_lowercase())
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        (self.classes.len(), self.ids.len(), self.tags.len())
    }

    /// 把一段"像选择器"的文本里的 class/id/tag 全部收进来（用于 main.js 字符串字面量）。
    pub fn add_selector_text(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b'.' => {
                    let (name, next) = read_ident(text, i + 1);
                    if !name.is_empty() {
                        self.add_class(&name);
                    }
                    i = next;
                }
                b'#' => {
                    let (name, next) = read_ident(text, i + 1);
                    if !name.is_empty() {
                        self.add_id(&name);
                    }
                    i = next;
                }
                b'[' => {
                    i = skip_balanced(text, i, b'[', b']');
                }
                b':' => {
                    i += 1;
                    if i < bytes.len() && bytes[i] == b':' {
                        i += 1;
                    }
                    let (_, next) = read_ident(text, i);
                    i = next;
                }
                c if c.is_ascii_alphabetic() || c == b'_' => {
                    let (name, next) = read_ident(text, i);
                    if !name.is_empty() {
                        self.add_tag(&name);
                    }
                    i = next;
                }
                b'*' | b'>' | b'+' | b'~' | b'|' | b',' | b'/' | b'-' | b'\\' => i += 1,
                _ => i += 1,
            }
        }
    }

    /// main.js 里运行时才出现的 class/id/tag（以及主题可能用到的状态类）。
    pub fn add_builtin_dynamic(&mut self) {
        for class in [
            "sidebar-display",
            "is-copied",
            "is-collapsed",
            "is-active-link",
            "is-active-li",
            "fa-fw",
            "fas",
            "far",
            "fab",
            "fa-check",
            "small",
            "loaded",
            "unloaded",
            "lazyload",
            "lazyloading",
            "lazyloaded",
            "prompt-tip",
            "prompt-info",
            "prompt-warning",
            "prompt-danger",
            "prompt-note",
            "mode-dark",
            "mode-light",
            "rouge-table",
            "rouge-code",
            "rouge-gutter",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "lead",
            "active",
            "disabled",
            "toc-list",
            "is-collapsible",
            "highlighter-rouge",
            "language-plaintext",
            "class_",
            "function_",
            "inherited__",
            // Markdown 渲染器按内容特性生成的类（任务列表 / 脚注 / 代码块 / 图片）：
            // 这些只在正文用到对应语法时出现，源码里没有字面量。
            "markdown-body",
            "task-list-item",
            "task-list-item-checkbox",
            "contains-task-list",
            "footnote-reference",
            "footnote-definition",
            "table-wrapper",
            "highlight",
            "code-block",
            "copy-button",
            "img-link",
            "popup",
            "anchor",
            "octicon",
            "octicon-link",
            "octicon-copy",
            "octicon-check",
            "me-2",
        ] {
            self.add_class(class);
        }
        // 渲染器成批生成的类：hljs token、rouge 代码表、prompt 变体、language-xxx，
        // 以及 TOC 的 node-name--H{level}（模板里是 format 占位符，字面量扫不到）。
        for prefix in ["hljs", "rouge", "prompt", "language", "node-name--"] {
            self.add_class_prefix(prefix);
        }
        // 正文由 markdown 渲染，标签集合不能只看当前内容：常见 markdown/HTML 标签全部放行。
        for tag in [
            "html",
            "body",
            "head",
            "script",
            "style",
            "link",
            "meta",
            "title",
            "textarea",
            "a",
            "button",
            "p",
            "blockquote",
            "pre",
            "code",
            "em",
            "strong",
            "del",
            "ins",
            "s",
            "u",
            "b",
            "i",
            "mark",
            "small",
            "sub",
            "sup",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "ul",
            "ol",
            "li",
            "dl",
            "dt",
            "dd",
            "table",
            "thead",
            "tbody",
            "tfoot",
            "tr",
            "th",
            "td",
            "hr",
            "br",
            "img",
            "figure",
            "figcaption",
            "details",
            "summary",
            "kbd",
            "samp",
            "var",
            "abbr",
            "cite",
            "q",
            "time",
            "address",
            "audio",
            "video",
            "picture",
            "source",
            "span",
            "div",
            "section",
            "article",
            "aside",
            "header",
            "footer",
            "main",
            "nav",
            "hgroup",
            "svg",
            "path",
            "circle",
            "g",
            "use",
            "chirpy-prompt",
            "col",
            "colgroup",
            // 正文原生 HTML / 渲染器输出里常见、但模板源码没有字面量的标签：
            "input",
            "label",
            "select",
            "option",
            "form",
            "iframe",
        ] {
            self.add_tag(tag);
        }
        for id in [
            "mode-toggle",
            "sidebar-trigger",
            "mask",
            "sidebar",
            "back-to-top",
            "toc",
            "page-category",
            "page-tag",
        ] {
            self.add_id(id);
        }
    }
}

/// 从 HTML 文本收集 class/id/tag。实现是宽松的（多收无妨，少收会误删）。
///
/// 正式构建不再读 `public/**/*.html`（见模块头），这个入口只留给离线报告工具
/// `rust/tools/css_report.rs` 和 `assets.rs` 的“源码使用集合覆盖真实页面”回归测试。
#[allow(dead_code)]
pub fn collect_html_usage(html: &str, usage: &mut Usage) {
    collect_html_tags(html, usage);
    collect_html_attr(html, "class", usage, true);
    collect_html_attr(html, "id", usage, false);
}

#[allow(dead_code)] // 仅被 collect_html_usage 使用（离线报告 / 回归测试）
fn collect_html_tags(html: &str, usage: &mut Usage) {
    let bytes = html.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let mut j = i + 1;
            if j < bytes.len() && bytes[j] == b'!' {
                i = j + 1;
                continue;
            }
            if j < bytes.len() && bytes[j] == b'/' {
                j += 1;
            }
            if j < bytes.len() && bytes[j].is_ascii_alphabetic() {
                let start = j;
                while j < bytes.len()
                    && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-' || bytes[j] == b'_')
                {
                    j += 1;
                }
                usage.add_tag(&html[start..j]);
                i = j;
                continue;
            }
        }
        i += 1;
    }
}

#[allow(dead_code)] // 仅被 collect_html_usage 使用（离线报告 / 回归测试）
fn collect_html_attr(html: &str, name: &str, usage: &mut Usage, is_class: bool) {
    let lower = html.to_ascii_lowercase();
    let hay = lower.as_bytes();
    let needle = name.as_bytes();
    let mut pos = 0usize;
    while pos + needle.len() <= hay.len() {
        let Some(rel) = hay[pos..].windows(needle.len()).position(|w| w == needle) else {
            break;
        };
        let at = pos + rel;
        pos = at + needle.len();
        // 属性名左侧必须是空白或 '<'，避免 data-class/data-id 之类误匹配
        if at > 0 {
            let prev = hay[at - 1];
            if !(prev.is_ascii_whitespace() || prev == b'<') {
                continue;
            }
        }
        let mut j = at + needle.len();
        while j < hay.len() && hay[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= hay.len() || hay[j] != b'=' {
            continue;
        }
        j += 1;
        while j < hay.len() && hay[j].is_ascii_whitespace() {
            j += 1;
        }
        let (value_start, value_end) = if j < hay.len() && (hay[j] == b'"' || hay[j] == b'\'') {
            let quote = hay[j];
            let start = j + 1;
            let mut k = start;
            while k < hay.len() && hay[k] != quote {
                k += 1;
            }
            (start, k)
        } else {
            let start = j;
            while j < hay.len() && !hay[j].is_ascii_whitespace() && hay[j] != b'>' && hay[j] != b'/'
            {
                j += 1;
            }
            (start, j)
        };
        // 用原始 HTML 的切片，保留大小写/非 ASCII（不包含引号）
        let value = &html[value_start..value_end.min(html.len())];
        if is_class {
            for class in value.split_ascii_whitespace() {
                usage.add_class(class);
            }
        } else {
            usage.add_id(value);
        }
    }
}

/// 从客户端 JS 收集字符串字面量里的选择器/类名（querySelector、classList、className 等）。
pub fn collect_js_usage(js: &str, usage: &mut Usage) {
    let bytes = js.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        if bytes[i] == b'"' || bytes[i] == b'\'' || bytes[i] == b'`' {
            let quote = bytes[i];
            let start = i + 1;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    break;
                }
                i += 1;
            }
            let end = i.min(bytes.len());
            let literal = &js[start..end];
            let prefix = js[..start].trim_end();
            if prefix.ends_with("createElement(") {
                usage.add_tag(literal);
            } else if literal.starts_with('#') && is_simple_ident(&literal[1..]) {
                usage.add_id(&literal[1..]);
            } else if literal.starts_with('.') && is_simple_ident(&literal[1..]) {
                usage.add_class(&literal[1..]);
            } else {
                usage.add_selector_text(literal);
            }
            i = (end + 1).min(bytes.len());
            continue;
        }
        i += 1;
    }
}

/// 扫描 Rust 源码里的 class/id 字面量（模板、Markdown 渲染器、站点配置里的图标类）。
///
/// 保守做法：
/// - `class="..."` 与 `class=\"...\"` 属性值（剥离 `{format}` 占位符）；
/// - `id="..."` 属性值；
/// - 普通字符串字面量里的 Font Awesome 类（`icon("fas fa-home")` 这类函数参数）。
///
/// 标签由 `add_builtin_dynamic` 的 markdown/HTML 标签白名单覆盖。
///
/// 该集合是静态构建与动态模式共用的**唯一**源码来源（源码在 assets.rs 里编译期内嵌），
/// 不再区分「有没有 public HTML」。
pub fn collect_rust_source_usage(source: &str, usage: &mut Usage) {
    collect_rust_attr(source, "class", true, usage);
    collect_rust_attr(source, "id", false, usage);
    collect_fa_icon_tokens(source, usage);
}

/// 扫描 Markdown 正文里直接写的原生 HTML 的 class/id 属性。
///
/// 正文由 `content/**/*.md` 决定，静态构建与动态模式读同一份内容；代码块里的示例 HTML
/// 会被一并收进来——多留样式无害，漏删才会出问题。
pub fn collect_markdown_source_usage(markdown: &str, usage: &mut Usage) {
    collect_rust_attr(markdown, "class", true, usage);
    collect_rust_attr(markdown, "id", false, usage);
}

fn collect_rust_attr(source: &str, name: &str, is_class: bool, usage: &mut Usage) {
    let bytes = source.as_bytes();
    let needle = name.as_bytes();
    let mut i = 0usize;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] != needle {
            i += 1;
            continue;
        }
        if i > 0 {
            let prev = bytes[i - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'-' {
                i += 1;
                continue;
            }
        }
        let mut j = i + needle.len();
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'=' {
            i += 1;
            continue;
        }
        j += 1;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        // Rust 普通字符串里 HTML 属性写作 class=\"...\"，raw string 里写作 class="..."
        let escaped = j < bytes.len() && bytes[j] == b'\\';
        if escaped {
            j += 1;
        }
        if j >= bytes.len() || (bytes[j] != b'"' && bytes[j] != b'\'') {
            i += 1;
            continue;
        }
        let quote = bytes[j];
        j += 1;
        let start = j;
        let mut end = start;
        while j < bytes.len() {
            if escaped {
                if bytes[j] == b'\\' && j + 1 < bytes.len() && bytes[j + 1] == quote {
                    end = j;
                    j += 2;
                    break;
                }
            } else if bytes[j] == quote {
                end = j;
                j += 1;
                break;
            }
            j += 1;
        }
        if end < start {
            end = start;
        }
        let value = &source[start..end.min(source.len())];
        let cleaned = strip_format_placeholders(value);
        if is_class {
            for class in cleaned.split_whitespace() {
                usage.add_class(class);
            }
        } else {
            let id = cleaned.trim();
            if !id.is_empty() {
                usage.add_id(id);
            }
        }
        i = j.max(i + needle.len());
    }
}

fn strip_format_placeholders(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut depth = 0usize;
    for ch in value.chars() {
        match ch {
            '{' => {
                depth += 1;
                out.push(' ');
            }
            '}' => {
                depth = depth.saturating_sub(1);
                out.push(' ');
            }
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

/// 从普通字符串字面量中提取 Font Awesome 类（图标名以函数参数形式出现，不在 class="..." 里）。
fn collect_fa_icon_tokens(source: &str, usage: &mut Usage) {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;
    while i < len {
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            i += 2;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            let mut depth = 1usize;
            i += 2;
            while i < len && depth > 0 {
                if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && i + 1 < len && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        if bytes[i] != b'"' {
            i += 1;
            continue;
        }
        // raw string（r"..." / r#"..."#）不扫：其中的图标都在 class="..." 里，已由属性扫描覆盖
        let mut k = i;
        let mut hashes = 0usize;
        while k > 0 && bytes[k - 1] == b'#' {
            hashes += 1;
            k -= 1;
        }
        if k > 0 && bytes[k - 1] == b'r' {
            let mut j = i + 1;
            let mut found = false;
            loop {
                if j >= len {
                    break;
                }
                if bytes[j] == b'"' {
                    let mut h = 0usize;
                    let mut q = j + 1;
                    while q < len && h < hashes && bytes[q] == b'#' {
                        h += 1;
                        q += 1;
                    }
                    if h == hashes {
                        i = q;
                        found = true;
                        break;
                    }
                }
                j += 1;
            }
            if !found {
                i = len;
            }
            continue;
        }
        let start = i + 1;
        let mut j = start;
        while j < len {
            if bytes[j] == b'\\' {
                j += 2;
                continue;
            }
            if bytes[j] == b'"' {
                break;
            }
            j += 1;
        }
        let literal = &source[start..j.min(len)];
        let lb = literal.as_bytes();
        let mut t = 0usize;
        while t < lb.len() {
            if lb[t].is_ascii_alphanumeric() || lb[t] == b'_' || lb[t] == b'-' {
                let begin = t;
                while t < lb.len()
                    && (lb[t].is_ascii_alphanumeric() || lb[t] == b'_' || lb[t] == b'-')
                {
                    t += 1;
                }
                let token = &literal[begin..t];
                if token == "fa"
                    || token == "fas"
                    || token == "far"
                    || token == "fab"
                    || token.starts_with("fa-")
                {
                    usage.add_class(token);
                }
            } else {
                t += 1;
            }
        }
        i = (j + 1).min(len);
    }
}

fn is_simple_ident(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn read_ident(text: &str, start: usize) -> (String, usize) {
    let bytes = text.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
            i += 1;
        } else {
            break;
        }
    }
    (text[start..i].to_string(), i)
}

fn skip_balanced(text: &str, start: usize, open: u8, close: u8) -> usize {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut i = start;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' => quote = Some(b),
            b if b == open => depth += 1,
            b if b == close => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// 仅离线报告工具读取这些字段；构建路径只统计数量，避免存储大清单。
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RemovedSelector {
    pub selector: String,
    pub reason: String,
    pub bytes: usize,
}

#[derive(Debug, Default, Clone)]
pub struct PruneReport {
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub selectors_total: usize,
    pub selectors_kept: usize,
    pub selectors_removed: usize,
    pub rules_total: usize,
    pub rules_kept: usize,
    pub removed: Vec<RemovedSelector>,
}

impl PruneReport {
    pub fn summary(&self) -> String {
        format!(
            "selectors {}/{} kept, rules {}/{}, bytes {} -> {} ({:+.1}%)",
            self.selectors_kept,
            self.selectors_total,
            self.rules_kept,
            self.rules_total,
            self.input_bytes,
            self.output_bytes,
            if self.input_bytes == 0 {
                0.0
            } else {
                (self.output_bytes as f64 - self.input_bytes as f64) * 100.0
                    / self.input_bytes as f64
            }
        )
    }
}

pub struct PruneResult {
    pub css: String,
    pub report: PruneReport,
}

/// 裁剪 CSS；`collect_details` 为 true 时记录每条被删选择器（离线报告用）。
pub fn prune_css(css: &str, usage: &Usage, collect_details: bool) -> PruneResult {
    let mut ctx = PruneCtx {
        usage,
        details: collect_details,
        report: PruneReport {
            input_bytes: css.len(),
            ..PruneReport::default()
        },
    };
    let out = ctx.prune_level(css);
    ctx.report.output_bytes = out.len();
    PruneResult {
        css: out,
        report: ctx.report,
    }
}

struct PruneCtx<'a> {
    usage: &'a Usage,
    details: bool,
    report: PruneReport,
}

impl PruneCtx<'_> {
    fn prune_level(&mut self, css: &str) -> String {
        let mut out = String::with_capacity(css.len());
        let bytes = css.as_bytes();
        let n = bytes.len();
        let mut i = 0usize;
        while i < n {
            // 顶部/块间的注释与空白原样保留（当前构建产物里只有版权注释）
            if bytes[i] == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
                let mut j = i + 2;
                while j + 1 < n && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                    j += 1;
                }
                let end = (j + 2).min(n);
                out.push_str(&css[i..end]);
                i = end;
                continue;
            }
            if bytes[i].is_ascii_whitespace() {
                out.push(bytes[i] as char);
                i += 1;
                continue;
            }

            // 找到 prelude 结束：'{' 或 ';'（跳过字符串）
            let mut j = i;
            let mut quote: Option<u8> = None;
            while j < n {
                let b = bytes[j];
                if let Some(q) = quote {
                    if b == b'\\' {
                        j += 2;
                        continue;
                    }
                    if b == q {
                        quote = None;
                    }
                    j += 1;
                    continue;
                }
                if b == b'"' || b == b'\'' {
                    quote = Some(b);
                    j += 1;
                    continue;
                }
                if b == b'{' || b == b';' {
                    break;
                }
                j += 1;
            }
            if j >= n {
                out.push_str(&css[i..]);
                break;
            }
            if bytes[j] == b';' {
                out.push_str(&css[i..=j]);
                i = j + 1;
                continue;
            }

            let end = find_block_end(css, j);
            let prelude = css[i..j].trim();
            let body = &css[j + 1..end];
            if prelude.starts_with('@') {
                out.push_str(&self.prune_at_rule(prelude, body));
            } else {
                out.push_str(&self.prune_style_rule(prelude, body));
            }
            i = end + 1;
        }
        out
    }

    fn prune_at_rule(&mut self, prelude: &str, body: &str) -> String {
        let lower = prelude.to_ascii_lowercase();
        let recursive = [
            "@media",
            "@supports",
            "@layer",
            "@container",
            "@scope",
            "@document",
        ]
        .iter()
        .any(|prefix| lower.starts_with(prefix));
        if recursive {
            let inner = self.prune_level(body);
            if inner.trim().is_empty() {
                String::new()
            } else {
                format!("{prelude}{{{inner}}}")
            }
        } else {
            // @keyframes/@font-face/@page/@property 以及未知 at-rule：整块保留
            format!("{prelude}{{{body}}}")
        }
    }

    fn prune_style_rule(&mut self, prelude: &str, body: &str) -> String {
        self.report.rules_total += 1;
        let selectors = split_selector_list(prelude);
        let mut kept = Vec::new();
        for selector in selectors {
            self.report.selectors_total += 1;
            let (keep, reason) = selector_kept(selector, self.usage);
            if keep {
                self.report.selectors_kept += 1;
                kept.push(selector.trim());
            } else {
                self.report.selectors_removed += 1;
                if self.details {
                    self.report.removed.push(RemovedSelector {
                        selector: selector.trim().to_string(),
                        reason,
                        bytes: selector.len(),
                    });
                }
            }
        }
        if kept.is_empty() {
            return String::new();
        }
        self.report.rules_kept += 1;
        format!("{}{{{body}}}", kept.join(","))
    }
}

fn split_selector_list(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut i = 0usize;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' => quote = Some(b),
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                out.push(&text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&text[start..]);
    out.into_iter().filter(|s| !s.trim().is_empty()).collect()
}

/// 选择器保留判断：任何 class/id/tag 命中即保留；全部未命中才删除。
fn selector_kept(selector: &str, usage: &Usage) -> (bool, String) {
    let sel = selector.trim();
    if sel.is_empty() {
        return (false, "empty".to_string());
    }
    // 已删除组件的选择器直接丢弃（含 :not() 等保守分支）
    if sel.contains(".post-navigation") {
        return (false, "removed-component".to_string());
    }
    // 转义标识符、属性选择器、函数式伪类一律保守保留
    if sel.contains('\\') {
        return (true, "escape".to_string());
    }
    if sel.contains('[') {
        return (true, "attribute".to_string());
    }
    if sel.contains('(') {
        return (true, "functional-pseudo".to_string());
    }

    let bytes = sel.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    let mut classes = Vec::new();
    let mut ids = Vec::new();
    let mut tags = Vec::new();
    while i < n {
        let b = bytes[i];
        if b.is_ascii_whitespace() || matches!(b, b'>' | b'+' | b'~' | b'|' | b'*' | b',') {
            i += 1;
            continue;
        }
        match b {
            b'.' => {
                let (name, next) = read_ident(sel, i + 1);
                if name.is_empty() {
                    return (true, "unknown".to_string());
                }
                classes.push(name);
                i = next;
            }
            b'#' => {
                let (name, next) = read_ident(sel, i + 1);
                if name.is_empty() {
                    return (true, "unknown".to_string());
                }
                ids.push(name);
                i = next;
            }
            b':' => {
                i += 1;
                if i < n && bytes[i] == b':' {
                    i += 1;
                }
                let (_, next) = read_ident(sel, i);
                i = next;
            }
            b if b.is_ascii_alphabetic() || b == b'_' || b == b'-' => {
                let (name, next) = read_ident(sel, i);
                if name.is_empty() {
                    return (true, "unknown".to_string());
                }
                tags.push(name);
                i = next;
            }
            _ => return (true, format!("unknown-char:{}", b as char)),
        }
    }

    if classes.is_empty() && ids.is_empty() && tags.is_empty() {
        return (true, "no-token".to_string());
    }
    // 已删除的组件：即使选择器里还有别的在用类（如 .btn），也整条丢弃
    if classes.iter().any(|name| is_removed_class(name)) {
        return (false, "removed-component".to_string());
    }
    if classes
        .iter()
        .any(|name| usage.has_class(name) || is_always_kept_class(name))
        || ids.iter().any(|name| usage.has_id(name))
        || tags.iter().any(|name| usage.has_tag(name))
    {
        return (true, "used".to_string());
    }

    let mut missing = Vec::new();
    missing.extend(classes.into_iter().map(|name| format!("class:{name}")));
    missing.extend(ids.into_iter().map(|name| format!("id:{name}")));
    missing.extend(tags.into_iter().map(|name| format!("tag:{name}")));
    (false, format!("unused({})", missing.join(",")))
}

/// 按选择器整体丢弃的已删除组件。
fn is_removed_class(name: &str) -> bool {
    name == "post-navigation"
}

/// 无条件保留的类：
///
/// - 栅格/布局类：模板里有些类是运行时拼出来的（如 `{main_col}`），使用情况收集可能漏掉；
///   漏掉这些类会让整页栅格塌掉，代价远大于多几 KB CSS。
/// - 条件渲染类：只在特定页面状态出现（翻页控件只在多页时渲染、`row-cols-*` 只在列表页、
///   `preview-img` 只在带封面的卡片），按“本次渲染的页面”去扫会时有时无，必须白名单化，
///   否则同一份 CSS 会随内容页数/封面增减而抖动。
pub fn is_always_kept_class(name: &str) -> bool {
    name == "row" || name == "container" || is_layout_class(name) || is_conditional_class(name)
}

fn is_layout_class(name: &str) -> bool {
    name.starts_with("lightbox")
        || name.starts_with("col-")
        || name.starts_with("g-")
        || name.starts_with("offset-")
        || name.starts_with("order-")
}

/// 条件渲染类白名单（宁可多留）：分页、列表栅格、封面图。
fn is_conditional_class(name: &str) -> bool {
    matches!(name, "pagination" | "preview-img" | "post-preview")
        || name.starts_with("page-")
        || name.starts_with("row-cols-")
        || name.starts_with("preview-")
}

fn find_block_end(src: &str, open: usize) -> usize {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' => quote = Some(b),
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len().saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage() -> Usage {
        let mut u = Usage::new();
        u.add_class("used");
        u.add_id("main");
        u.add_tag("div");
        u
    }

    #[test]
    fn keeps_used_and_drops_unused() {
        let u = usage();
        assert!(selector_kept(".used .child", &u).0);
        assert!(selector_kept("div .child", &u).0);
        assert!(!selector_kept(".other .child", &u).0);
        assert!(!selector_kept("span.foo", &u).0);
    }

    #[test]
    fn drops_removed_component_selectors() {
        let mut u = Usage::default();
        u.add_class("btn");
        assert!(!selector_kept(".post-navigation .btn", &u).0);
        assert!(!selector_kept(".post-navigation .btn:not(:hover)", &u).0);
        assert!(selector_kept(".btn", &u).0);
    }

    #[test]
    fn keeps_layout_classes_even_if_unused() {
        let u = Usage::default();
        for sel in [
            ".col-xl-9",
            ".row",
            ".container",
            ".g-4",
            ".offset-2",
            ".lightbox",
            ".lightbox img",
        ] {
            assert!(selector_kept(sel, &u).0, "{sel} 应保留");
        }
    }

    #[test]
    fn keeps_conditional_render_classes() {
        // 这些类只在特定页面状态出现（多页才有 pagination、列表页才有 row-cols-*、
        // 带封面才有 preview-img），不能按“本次渲染的页面”决定去留。
        let u = Usage::default();
        for sel in [
            ".pagination",
            ".pagination .page-item.active .page-link",
            ".page-item:first-child .page-link",
            ".page-index",
            ".row-cols-1",
            ".row-cols-md-2>*",
            ".row-cols-xl-3",
            ".preview-img img",
            ".preview-img:not(.no-bg) img.lazyloaded",
            ".post-preview",
        ] {
            assert!(selector_kept(sel, &u).0, "{sel} 应保留");
        }
        // 白名单之外、又确实没出现过的类仍然照常裁剪。
        assert!(!selector_kept(".totally-unused-widget", &u).0);
    }

    #[test]
    fn markdown_source_scan_collects_raw_html_attrs() {
        let mut u = Usage::new();
        let md = r#"正文
<div class="alert alert-note" id="custom-anchor">提示</div>
<span class='badge hot'>热</span>
```html
<pre class="code-sample">示例</pre>
```
"#;
        collect_markdown_source_usage(md, &mut u);
        for class in ["alert", "alert-note", "badge", "hot", "code-sample"] {
            assert!(u.has_class(class), "missing class {class}");
        }
        assert!(u.has_id("custom-anchor"));
        // 普通正文里的英文单词不会被误当成标签（只收 class/id 属性）。
        assert!(!u.has_tag("body"));
    }

    #[test]
    fn keeps_special_selectors() {
        let u = usage();
        assert!(selector_kept(":root", &u).0);
        assert!(selector_kept("*,*:before,*:after", &u).0);
        assert!(selector_kept("[hidden]", &u).0);
        assert!(selector_kept("html[data-mode=dark]", &u).0);
        assert!(selector_kept(":not(.unused)", &u).0);
        assert!(selector_kept("::-webkit-scrollbar", &u).0);
    }

    #[test]
    fn source_scan_collects_attrs_and_icons() {
        let mut u = Usage::new();
        let source = r##"
            let a = r#"<div class="foo bar active{active_class}" id="page-{kind}">"#;
            let b = "class=\"highlight language-plaintext\"";
            const ICON: &str = "fas fa-hashtag";
            let c = icon("fa-fw fa-solid fa-home");
        "##;
        collect_rust_source_usage(source, &mut u);
        for class in [
            "foo",
            "bar",
            "active",
            "highlight",
            "language-plaintext",
            "fas",
            "fa-hashtag",
            "fa-fw",
            "fa-solid",
            "fa-home",
        ] {
            assert!(u.has_class(class), "missing class {class}");
        }
    }

    #[test]
    fn source_scan_actual_files_cover_known_classes() {
        let mut u = Usage::new();
        for src in [
            include_str!("../src/markdown.rs"),
            include_str!("../src/templates.rs"),
            include_str!("../src/site.rs"),
        ] {
            collect_rust_source_usage(src, &mut u);
        }
        u.add_builtin_dynamic();
        for class in [
            "fa-hashtag",
            "fa-home",
            "fa-github",
            "highlight",
            "toc-list",
            "is-collapsible",
            "active",
        ] {
            assert!(u.has_class(class), "missing class {class}");
        }
    }

    #[test]
    fn prunes_media_but_keeps_variables_and_keyframes() {
        let u = usage();
        let css = ":root{--x:1}.unused{color:red}@media (min-width:1px){.other{color:red}}@keyframes spin{from{opacity:0}to{opacity:1}}";
        let result = prune_css(css, &u, true);
        assert!(result.css.contains(":root{--x:1}"));
        assert!(!result.css.contains(".unused{"));
        assert!(result.css.contains("@keyframes spin"));
        assert!(!result.css.contains("@media (min-width:1px)"));
        // 媒体查询里仍有使用中的规则时，wrapper 必须保留
        let css = "@media (min-width:1px){.used{color:blue}.other{color:red}}";
        let result = prune_css(css, &u, true);
        assert!(
            result
                .css
                .contains("@media (min-width:1px){.used{color:blue}}")
        );
        assert!(!result.css.contains(".other"));
    }
}
