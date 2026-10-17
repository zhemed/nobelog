// Markdown -> HTML。接口冻结：
//   pub fn render(body: &str) -> (String, Vec<TocEntry>)
// 本文件是 src/markdown.ts 的逐字节移植：markdown-it 的正文结构、::: 提示块、
// upgradeHtml 的表格/图片/行内代码/标题锚点与 TOC 都在这里实现。
// 唯一例外是 highlight.js 的 token 层——由 syntect + hljs-* 映射实现（见 highlight.rs /
// highlight_map.rs），因此代码块内部 token 与 Node 版不保证逐字节一致。
use crate::model::TocEntry;
use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

#[path = "highlight.rs"]
pub mod highlight;

const INLINE_CODE_CLASS: &str = "language-plaintext highlighter-rouge";
const PROMPT_KINDS: [&str; 4] = ["tip", "info", "warning", "danger"];
const ANCHOR_ICON: &str = "fas fa-hashtag";
const COPY_SVG: &str = "<svg class=\"octicon octicon-copy\" viewBox=\"0 0 16 16\" width=\"16\" height=\"16\" aria-hidden=\"true\"><path d=\"M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z\"></path><path d=\"M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z\"></path></svg>";
const CHECK_SVG: &str = "<svg class=\"octicon octicon-check\" viewBox=\"0 0 16 16\" width=\"16\" height=\"16\" aria-hidden=\"true\"><path d=\"M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.060.751.751 0 0 1 1.06-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z\"></path></svg>";

fn is_toc_level(level: usize) -> bool {
    matches!(level, 2..=4)
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_attr(text: &str) -> String {
    escape_html(text)
}

/// 语言名只允许能安全出现在 class 里的字符；实时站保留作者写法（如 language-SQL）。
fn safe_lang_name(lang: &str) -> String {
    lang.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '#' | '.' | '-'))
        .collect()
}

/// 标题 id：保留中英文与数字，其余压成连字符（浏览器自己会给非 ASCII 做百分号编码）。
fn slugify(text: &str, counts: &mut HashMap<String, usize>) -> String {
    let lower = text.to_lowercase();
    let mut dashed = String::with_capacity(lower.len());
    let mut last_was_space = false;
    for ch in lower.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                dashed.push('-');
                last_was_space = true;
            }
        } else {
            dashed.push(ch);
            last_was_space = false;
        }
    }
    // JS 的 .slice(0, 80) 按 UTF-16 code unit 截断。
    let mut units = 0usize;
    let mut cut = String::new();
    for ch in dashed.chars() {
        let size = ch.len_utf16();
        if units + size > 80 {
            break;
        }
        units += size;
        cut.push(ch);
    }
    let mut base: String = cut
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    while base.starts_with('-') {
        base.remove(0);
    }
    while base.ends_with('-') {
        base.pop();
    }
    let safe = if base.is_empty() {
        "section".to_string()
    } else {
        base
    };
    let seen = *counts.get(&safe).unwrap_or(&0);
    counts.insert(safe.clone(), seen + 1);
    if seen == 0 {
        safe
    } else {
        format!("{safe}-{seen}")
    }
}

fn decode_entities(input: &str) -> String {
    static DEC: OnceLock<Regex> = OnceLock::new();
    static HEX: OnceLock<Regex> = OnceLock::new();
    let dec = DEC.get_or_init(|| Regex::new(r"&#(\d+);").unwrap());
    let hex = HEX.get_or_init(|| Regex::new(r"&#x([0-9a-fA-F]+);").unwrap());
    let step1 = dec.replace_all(input, |caps: &regex::Captures<'_>| {
        let code: u32 = caps[1].parse().unwrap_or(0xFFFD);
        char::from_u32(code).unwrap_or('\u{FFFD}').to_string()
    });
    let step2 = hex.replace_all(&step1, |caps: &regex::Captures<'_>| {
        let code = u32::from_str_radix(&caps[1], 16).unwrap_or(0xFFFD);
        char::from_u32(code).unwrap_or('\u{FFFD}').to_string()
    });
    step2
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

fn text_of(inner_html: &str) -> String {
    static TAGS: OnceLock<Regex> = OnceLock::new();
    let tags = TAGS.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());
    let stripped = tags.replace_all(inner_html, "");
    decode_entities(&stripped)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn anchor_html(id: &str) -> String {
    format!(
        "<a href=\"#{}\" class=\"anchor text-muted\"><i class=\"{}\"></i></a>",
        escape_attr(id),
        ANCHOR_ICON
    )
}

fn code_block_html(lang: &str, code: &str) -> String {
    let body = highlight::highlight(if lang.is_empty() { None } else { Some(lang) }, code);
    let name = safe_lang_name(lang);
    let lang_class = if name.is_empty() {
        "plaintext".to_string()
    } else {
        name
    };
    format!(
        "<div class=\"highlight\"><pre class=\"code-block\"><code class=\"language-{lang_class}\" data-highlighted=\"yes\">{body}</code></pre><button class=\"copy-button\" type=\"button\" aria-label=\"复制代码\" data-copy>{COPY_SVG}{CHECK_SVG}</button></div>"
    )
}

/// mdurl.encode（markdown-it 的 normalizeLink 主体，keepEscaped=true）。
fn normalize_link(url: &str) -> String {
    const DEFAULT_CHARS: &str = ";/?:@&=+$,-_.!~*'()#";
    let bytes = url.as_bytes();
    let mut out = String::with_capacity(url.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let ch = url[i..].chars().next().unwrap();
        let code = ch as u32;
        if ch == '%'
            && i + 2 < bytes.len()
            && bytes[i + 1].is_ascii_hexdigit()
            && bytes[i + 2].is_ascii_hexdigit()
        {
            out.push_str(&url[i..i + 3]);
            i += 3;
            continue;
        }
        if code < 128 {
            if ch.is_ascii_alphanumeric() || DEFAULT_CHARS.contains(ch) {
                out.push(ch);
            } else {
                out.push_str(&format!("%{code:02X}"));
            }
        } else {
            let mut buf = [0u8; 4];
            for byte in ch.encode_utf8(&mut buf).bytes() {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
        i += ch.len_utf8();
    }
    out
}

#[derive(Default)]
struct Ctx {
    toc: Vec<TocEntry>,
    slug_count: HashMap<String, usize>,
}

/// Node 版 loadContent 复用一个 renderer 渲染全部文章，slugCount 因此跨文章累积
/// （页面则是每页新建 renderer）。`render` 保持单文件无状态；构建文章列表时用
/// `render_shared` 复刻 Node 的跨文章去重（例如 plugin-uposs 的「删除」会让
/// vimGettingStarted 的同名标题变成「删除-1」）。
static SHARED_SLUG_COUNTS: std::sync::Mutex<Option<HashMap<String, usize>>> =
    std::sync::Mutex::new(None);

struct HeadingBuf {
    html: String,
    text: String,
}

struct ImgBuf {
    dest: String,
    title: String,
    alt: String,
}

#[derive(Default)]
struct TableState {
    in_head: bool,
    body_open: bool,
    align: Vec<Alignment>,
    cell_index: usize,
}

struct Renderer<'a> {
    ctx: &'a mut Ctx,
    out: String,
    heading: Option<HeadingBuf>,
    images: Vec<ImgBuf>,
    code: Option<(String, String)>,
    list_stack: Vec<bool>,
    table: TableState,
}

impl<'a> Renderer<'a> {
    fn new(ctx: &'a mut Ctx) -> Self {
        Self {
            ctx,
            out: String::new(),
            heading: None,
            images: Vec::new(),
            code: None,
            list_stack: Vec::new(),
            table: TableState::default(),
        }
    }

    fn push(&mut self, text: &str) {
        match self.heading.as_mut() {
            Some(h) => h.html.push_str(text),
            None => self.out.push_str(text),
        }
    }

    fn push_text(&mut self, text: &str) {
        if let Some(img) = self.images.last_mut() {
            img.alt.push_str(text);
        } else {
            self.push(&escape_html(text));
            if let Some(h) = self.heading.as_mut() {
                h.text.push_str(text);
            }
        }
    }

    fn push_code_text(&mut self, text: &str) {
        if let Some(img) = self.images.last_mut() {
            img.alt.push_str(text);
            return;
        }
        self.push("<code>");
        self.push(&escape_html(text));
        self.push("</code>");
        if let Some(h) = self.heading.as_mut() {
            h.text.push_str(text);
        }
    }

    fn ensure_block_start(&mut self) {
        if let Some(h) = self.heading.as_mut() {
            if !h.html.is_empty() && !h.html.ends_with('\n') {
                h.html.push('\n');
            }
            return;
        }
        if !self.out.is_empty() && !self.out.ends_with('\n') {
            self.out.push('\n');
        }
    }

    fn run(mut self, source: &str) -> String {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        let events: Vec<Event<'_>> = Parser::new_ext(source, opts).collect();
        for event in events {
            if let (Some((_, buffer)), Event::Text(text)) = (self.code.as_mut(), &event) {
                buffer.push_str(text);
                continue;
            }
            match event {
                Event::Start(tag) => self.start_tag(tag),
                Event::End(tag) => self.end_tag(tag),
                Event::Text(text) => self.push_text(&text),
                Event::Code(text) => self.push_code_text(&text),
                Event::Html(html) => {
                    if self.images.is_empty() {
                        self.ensure_block_start();
                        self.push(&html);
                    }
                }
                Event::InlineHtml(html) => {
                    if self.images.is_empty() {
                        self.push(&html);
                    }
                }
                Event::SoftBreak => {
                    if self.images.is_empty() {
                        self.push("\n");
                        if let Some(h) = self.heading.as_mut() {
                            h.text.push(' ');
                        }
                    }
                }
                Event::HardBreak => {
                    if self.images.is_empty() {
                        self.push("<br>\n");
                        if let Some(h) = self.heading.as_mut() {
                            h.text.push(' ');
                        }
                    }
                }
                Event::Rule => {
                    self.ensure_block_start();
                    self.push("<hr>\n");
                }
                _ => {}
            }
        }
        self.out
    }

    fn start_tag(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                self.ensure_block_start();
                self.push("<p>");
            }
            Tag::Heading { .. } => {
                self.ensure_block_start();
                self.heading = Some(HeadingBuf {
                    html: String::new(),
                    text: String::new(),
                });
            }
            Tag::BlockQuote(_) => {
                self.ensure_block_start();
                self.push("<blockquote>\n");
            }
            Tag::List(start) => {
                self.ensure_block_start();
                let ordered = start.is_some();
                match start {
                    Some(1) | None => self.push(if ordered { "<ol>\n" } else { "<ul>\n" }),
                    Some(n) => self.push(&format!("<ol start=\"{n}\">\n")),
                }
                self.list_stack.push(ordered);
            }
            Tag::Item => self.push("<li>"),
            Tag::CodeBlock(kind) => {
                let lang = match &kind {
                    CodeBlockKind::Fenced(info) => {
                        info.split_whitespace().next().unwrap_or("").to_string()
                    }
                    CodeBlockKind::Indented => String::new(),
                };
                self.code = Some((lang, String::new()));
            }
            Tag::Link {
                dest_url, title, ..
            } => {
                let href = normalize_link(&dest_url);
                self.push(&format!("<a href=\"{}\"", escape_html(&href)));
                if !title.is_empty() {
                    self.push(&format!(" title=\"{}\"", escape_html(&title)));
                }
                self.push(">");
            }
            Tag::Image {
                dest_url, title, ..
            } => {
                self.images.push(ImgBuf {
                    dest: dest_url.to_string(),
                    title: title.to_string(),
                    alt: String::new(),
                });
            }
            Tag::Emphasis => self.push("<em>"),
            Tag::Strong => self.push("<strong>"),
            Tag::Table(aligns) => {
                self.ensure_block_start();
                self.table = TableState {
                    align: aligns,
                    ..Default::default()
                };
                self.push("<table>\n");
            }
            Tag::TableHead => {
                self.table.in_head = true;
                self.table.cell_index = 0;
                self.push("<thead>\n<tr>\n");
            }
            Tag::TableRow => {
                if !self.table.in_head && !self.table.body_open {
                    self.table.body_open = true;
                    self.push("<tbody>\n");
                }
                self.table.cell_index = 0;
                self.push("<tr>\n");
            }
            Tag::TableCell => {
                let tag = if self.table.in_head { "th" } else { "td" };
                let style = match self.table.align.get(self.table.cell_index).copied() {
                    Some(Alignment::Left) => Some("text-align:left"),
                    Some(Alignment::Center) => Some("text-align:center"),
                    Some(Alignment::Right) => Some("text-align:right"),
                    _ => None,
                };
                self.table.cell_index += 1;
                match style {
                    Some(style) => self.push(&format!("<{tag} style=\"{style}\">")),
                    None => self.push(&format!("<{tag}>")),
                }
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.push("</p>\n"),
            TagEnd::Heading(level) => {
                if let Some(heading) = self.heading.take() {
                    let level = heading_number(level);
                    let id = slugify(heading.text.trim(), &mut self.ctx.slug_count);
                    self.out.push_str(&format!(
                        "<h{level} id=\"{}\">{}</h{level}>\n",
                        escape_attr(&id),
                        heading.html
                    ));
                }
            }
            TagEnd::BlockQuote(_) => self.push("</blockquote>\n"),
            TagEnd::List(_) => {
                let ordered = self.list_stack.pop().unwrap_or(false);
                self.push(if ordered { "</ol>\n" } else { "</ul>\n" });
            }
            TagEnd::Item => self.push("</li>\n"),
            TagEnd::CodeBlock => {
                if let Some((lang, code)) = self.code.take() {
                    self.ensure_block_start();
                    self.push(&code_block_html(&lang, &code));
                    self.push("\n");
                }
            }
            TagEnd::Link => self.push("</a>"),
            TagEnd::Image => {
                if let Some(img) = self.images.pop() {
                    if let Some(parent) = self.images.last_mut() {
                        parent.alt.push_str(&img.alt);
                        return;
                    }
                    let src = normalize_link(&img.dest);
                    let mut tag = format!(
                        "<img src=\"{}\" alt=\"{}\"",
                        escape_html(&src),
                        escape_html(&img.alt)
                    );
                    if !img.title.is_empty() {
                        tag.push_str(&format!(" title=\"{}\"", escape_html(&img.title)));
                    }
                    tag.push('>');
                    self.push(&tag);
                    if let Some(h) = self.heading.as_mut() {
                        h.text.push_str(&img.alt);
                    }
                }
            }
            TagEnd::Emphasis => self.push("</em>"),
            TagEnd::Strong => self.push("</strong>"),
            TagEnd::Table => {
                if self.table.body_open {
                    self.push("</tbody>\n");
                }
                self.push("</table>\n");
                self.table = TableState::default();
            }
            TagEnd::TableHead => {
                self.push("</tr>\n</thead>\n");
                self.table.in_head = false;
            }
            TagEnd::TableRow => self.push("</tr>\n"),
            TagEnd::TableCell => {
                if self.table.in_head {
                    self.push("</th>\n");
                } else {
                    self.push("</td>\n");
                }
            }
            _ => {}
        }
    }
}

fn heading_number(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

// ---------------------------------------------------------------------------
// upgradeHtml：主题在浏览器里做的那一套（codeClass.js + refactor-content.html）
// ---------------------------------------------------------------------------

fn upgrade_html(html: &str, ctx: &mut Ctx) -> String {
    let with_tables = wrap_tables(html);
    let with_images = wrap_images(&with_tables);
    let with_code = upgrade_code(&with_images);
    upgrade_headings(&with_code, ctx)
}

fn wrap_tables(html: &str) -> String {
    static TABLE: OnceLock<Regex> = OnceLock::new();
    let re = TABLE.get_or_init(|| Regex::new(r"(?s)<table\b[\s\S]*?</table>").unwrap());
    re.replace_all(html, |caps: &regex::Captures<'_>| {
        let table = &caps[0];
        let open_end = table.find('>').map(|i| i + 1).unwrap_or(0);
        let open_tag = &table[..open_end];
        if contains_word(open_tag, "rouge-table") {
            table.to_string()
        } else {
            format!("<div class=\"table-wrapper\">{table}</div>")
        }
    })
    .into_owned()
}

fn contains_word(haystack: &str, word: &str) -> bool {
    let bytes = haystack.as_bytes();
    let needle = word.as_bytes();
    if needle.is_empty() || bytes.len() < needle.len() {
        return false;
    }
    let mut i = 0usize;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let before_ok = i == 0 || !is_word_byte(bytes[i - 1]);
            let after = i + needle.len();
            let after_ok = after == bytes.len() || !is_word_byte(bytes[after]);
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn wrap_images(html: &str) -> String {
    static IMG: OnceLock<Regex> = OnceLock::new();
    static SRC: OnceLock<Regex> = OnceLock::new();
    let re = IMG.get_or_init(|| Regex::new(r#"(?s)(<a\b[^>]*>\s*)?<img\b([^>]*)>"#).unwrap());
    let src_re = SRC.get_or_init(|| Regex::new(r#"\bsrc="([^"]*)""#).unwrap());
    re.replace_all(html, |caps: &regex::Captures<'_>| {
        let whole = &caps[0];
        if caps.get(1).is_some() {
            return whole.to_string();
        }
        let attrs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        match src_re.captures(attrs) {
            Some(src) => format!(
                "<a href=\"{}\" class=\"popup img-link\">{}</a>",
                &src[1], whole
            ),
            None => whole.to_string(),
        }
    })
    .into_owned()
}

fn inline_code_open_tag(tag: &str) -> String {
    static CLASS: OnceLock<Regex> = OnceLock::new();
    let re = CLASS.get_or_init(|| Regex::new(r#"class="([^"]*)""#).unwrap());
    if let Some(caps) = re.captures(tag) {
        let existing = &caps[1];
        let mut merged: Vec<&str> = existing
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .collect();
        for class in INLINE_CODE_CLASS.split_whitespace() {
            if !merged.contains(&class) {
                merged.push(class);
            }
        }
        let replacement = format!("class=\"{}\"", merged.join(" "));
        return re.replace(tag, replacement.as_str()).into_owned();
    }
    format!(
        "<code class=\"{INLINE_CODE_CLASS}\"{}",
        &tag["<code".len()..]
    )
}

/// upgradeHtml 里的 CODE_PATTERN：<pre><code>…</code></pre> 整块，或散落的行内 <code>。
/// JS 版带 (?<!<pre>) 与 (?!\s+data-highlighted=)，这里用扫描器等价实现。
fn upgrade_code(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut i = 0usize;
    let bytes = html.as_bytes();
    while i < bytes.len() {
        let rest = &html[i..];
        let next_pre = rest.find("<pre><code");
        let next_code = rest.find("<code");
        let pos = match (next_pre, next_code) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => {
                out.push_str(rest);
                break;
            }
        };
        out.push_str(&html[i..i + pos]);
        i += pos;

        // 第一分支：<pre><code ATTRS>INNER</code></pre>
        if html[i..].starts_with("<pre><code") {
            let attrs_start = i + "<pre><code".len();
            if let Some(gt) = html[attrs_start..].find('>') {
                let inner_start = attrs_start + gt + 1;
                if let Some(end) = html[inner_start..].find("</code></pre>") {
                    let attrs = &html[attrs_start..attrs_start + gt];
                    let inner = &html[inner_start..inner_start + end];
                    let lang = lang_from_attrs(attrs);
                    out.push_str(&code_block_html(&lang, &decode_entities(inner)));
                    i = inner_start + end + "</code></pre>".len();
                    continue;
                }
            }
            out.push_str("<pre>");
            i += "<pre>".len();
            continue;
        }

        // 第二分支：<code ...>（(?<!<pre>) 用前 5 个字符判断）
        let preceded_by_pre = i >= 5 && &bytes[i - 5..i] == b"<pre>";
        let after = &html[i + "<code".len()..];
        let starts_with_space = after.starts_with(|c: char| c.is_whitespace());
        let starts_with_data =
            starts_with_space && after.trim_start().starts_with("data-highlighted=");
        if preceded_by_pre || starts_with_data {
            out.push_str("<code");
            i += "<code".len();
            continue;
        }
        if after.starts_with('>') {
            let end = i + "<code".len() + 1;
            out.push_str(&inline_code_open_tag(&html[i..end]));
            i = end;
            continue;
        }
        if starts_with_space && let Some(gt) = after.find('>') {
            let end = i + "<code".len() + gt + 1;
            out.push_str(&inline_code_open_tag(&html[i..end]));
            i = end;
            continue;
        }
        out.push_str("<code");
        i += "<code".len();
    }
    out
}

fn lang_from_attrs(attrs: &str) -> String {
    static CLASS: OnceLock<Regex> = OnceLock::new();
    static LANG: OnceLock<Regex> = OnceLock::new();
    let class_re = CLASS.get_or_init(|| Regex::new(r#"class="([^"]*)""#).unwrap());
    let lang_re = LANG.get_or_init(|| Regex::new(r#"\blanguage-([^"\s]+)"#).unwrap());
    let classes = class_re
        .captures(attrs)
        .map(|c| c[1].to_string())
        .unwrap_or_default();
    lang_re
        .captures(&classes)
        .map(|c| c[1].to_string())
        .unwrap_or_default()
}

fn upgrade_headings(html: &str, ctx: &mut Ctx) -> String {
    let bytes = html.as_bytes();
    let mut out = String::with_capacity(html.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let Some(rel) = html[i..].find("<h") else {
            out.push_str(&html[i..]);
            break;
        };
        let start = i + rel;
        out.push_str(&html[i..start]);
        let level_byte = bytes.get(start + 2).copied().unwrap_or(0);
        if !(b'2'..=b'4').contains(&level_byte) {
            out.push_str("<h");
            i = start + 2;
            continue;
        }
        let level = (level_byte - b'0') as usize;
        let Some(gt_rel) = html[start..].find('>') else {
            out.push_str(&html[start..]);
            break;
        };
        let attrs_start = start + 3;
        let attrs = &html[attrs_start..start + gt_rel];
        let inner_start = start + gt_rel + 1;
        let close = format!("</h{level}>");
        let Some(inner_end_rel) = html[inner_start..].find(&close) else {
            out.push_str(&html[start..]);
            break;
        };
        let inner = &html[inner_start..inner_start + inner_end_rel];
        let end = inner_start + inner_end_rel + close.len();

        let text = text_of(inner);
        let declared = declared_id(attrs);
        let needs_id = declared.is_none() && is_toc_level(level);
        let id = match declared {
            Some(id) => id,
            None => {
                if needs_id {
                    slugify(&text, &mut ctx.slug_count)
                } else {
                    String::new()
                }
            }
        };
        if id.is_empty() {
            out.push_str(&html[start..end]);
            i = end;
            continue;
        }
        if is_toc_level(level) {
            ctx.toc.push(TocEntry {
                level,
                id: id.clone(),
                text,
            });
        }
        out.push_str(&format!("<h{level}{attrs}"));
        if needs_id {
            out.push_str(&format!(" id=\"{}\"", escape_attr(&id)));
        }
        out.push('>');
        if is_toc_level(level) {
            out.push_str("<span class=\"me-2\">");
            out.push_str(inner);
            out.push_str("</span>");
            out.push_str(&anchor_html(&id));
            out.push_str(&close);
        } else {
            out.push_str(inner);
            out.push_str(&close);
        }
        i = end;
    }
    out
}

fn declared_id(attrs: &str) -> Option<String> {
    static ID: OnceLock<Regex> = OnceLock::new();
    let re = ID.get_or_init(|| Regex::new(r#"\bid="([^"]*)""#).unwrap());
    re.captures(attrs).map(|c| c[1].to_string())
}

// ---------------------------------------------------------------------------
// 迁移正文里的原始 <pre><code> 块：先抠成占位符，渲染后再换回主题代码块。
// ---------------------------------------------------------------------------

fn protect_raw_code_blocks(source: &str, base: usize) -> (String, Vec<String>) {
    static PRE: OnceLock<Regex> = OnceLock::new();
    static INNER: OnceLock<Regex> = OnceLock::new();
    static CLASS: OnceLock<Regex> = OnceLock::new();
    static LANG: OnceLock<Regex> = OnceLock::new();
    let pre_re = PRE.get_or_init(|| Regex::new(r"(?is)<pre\b[^>]*>[\s\S]*?</pre>").unwrap());
    let inner_re = INNER.get_or_init(|| {
        Regex::new(r"(?is)^<pre\b[^>]*>\s*<code\b([^>]*)>([\s\S]*?)</code>\s*</pre>$").unwrap()
    });
    let class_re = CLASS.get_or_init(|| Regex::new(r#"class="([^"]*)""#).unwrap());
    let lang_re = LANG.get_or_init(|| Regex::new(r#"\blanguage-([^"\s]+)"#).unwrap());

    let lines: Vec<&str> = source.split('\n').collect();
    let mut fences: Vec<(usize, usize)> = Vec::new();
    let mut opened: Option<usize> = None;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("```") || trimmed.starts_with("~~~")) {
            continue;
        }
        match opened {
            None => opened = Some(index),
            Some(from) => {
                fences.push((from, index));
                opened = None;
            }
        }
    }
    if let Some(from) = opened {
        fences.push((from, lines.len().saturating_sub(1)));
    }

    let mut blocks: Vec<String> = Vec::new();
    let text = pre_re
        .replace_all(source, |caps: &regex::Captures<'_>| {
            let whole = &caps[0];
            let offset = caps.get(0).unwrap().start();
            let line_index = source[..offset].bytes().filter(|b| *b == b'\n').count();
            if fences
                .iter()
                .any(|(from, to)| line_index >= *from && line_index <= *to)
            {
                return whole.to_string();
            }
            let Some(parsed) = inner_re.captures(whole) else {
                return whole.to_string();
            };
            let attrs = &parsed[1];
            let inner = &parsed[2];
            let classes = class_re
                .captures(attrs)
                .map(|c| c[1].to_string())
                .unwrap_or_default();
            let lang = lang_re
                .captures(&classes)
                .map(|c| c[1].to_string())
                .unwrap_or_default();
            let rendered = code_block_html(&lang, &decode_entities(inner));
            let placeholder = format!("<div data-nb-code=\"{}\"></div>", base + blocks.len());
            blocks.push(rendered);
            placeholder
        })
        .into_owned();
    (text, blocks)
}

fn restore_raw_code_blocks(html: &str, blocks: &[String]) -> String {
    static PLACEHOLDER: OnceLock<Regex> = OnceLock::new();
    let re = PLACEHOLDER.get_or_init(|| Regex::new(r#"<div data-nb-code="(\d+)"></div>"#).unwrap());
    re.replace_all(html, |caps: &regex::Captures<'_>| {
        let index: usize = caps[1].parse().unwrap_or(usize::MAX);
        blocks.get(index).cloned().unwrap_or_default()
    })
    .into_owned()
}

// ---------------------------------------------------------------------------
// ::: tip/info/warning/danger 提示块（markdown-it block rule 的等价实现）
// ---------------------------------------------------------------------------

enum Segment {
    Markdown(String),
    Prompt {
        kind: String,
        title: String,
        body: String,
    },
}

fn split_prompts(source: &str) -> Vec<Segment> {
    let lines: Vec<&str> = source.split('\n').collect();
    let mut segments: Vec<Segment> = Vec::new();
    let mut markdown: Vec<&str> = Vec::new();
    let mut in_fence = false;
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            markdown.push(line);
            i += 1;
            continue;
        }
        let at_block_start = i == 0 || lines[i - 1].trim().is_empty();
        let parsed = if in_fence || !at_block_start {
            None
        } else {
            parse_prompt_open(line)
        };
        let Some((kind, title)) = parsed else {
            markdown.push(line);
            i += 1;
            continue;
        };
        let mut body_end = i + 1;
        while body_end < lines.len() && !is_prompt_close(lines[body_end]) {
            body_end += 1;
        }
        if body_end >= lines.len() {
            markdown.push(line);
            i += 1;
            continue;
        }
        if !markdown.is_empty() {
            segments.push(Segment::Markdown(markdown.join("\n")));
            markdown.clear();
        }
        segments.push(Segment::Prompt {
            kind,
            title,
            body: lines[i + 1..body_end].join("\n"),
        });
        i = body_end + 1;
    }
    if !markdown.is_empty() {
        segments.push(Segment::Markdown(markdown.join("\n")));
    }
    segments
}

fn parse_prompt_open(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix(":::")?;
    let rest = rest.trim_start();
    let mut end = rest.len();
    for (idx, ch) in rest.char_indices() {
        if !ch.is_ascii_alphabetic() {
            end = idx;
            break;
        }
    }
    let kind = rest[..end].to_lowercase();
    if !PROMPT_KINDS.contains(&kind.as_str()) {
        return None;
    }
    let title = rest[end..].trim().to_string();
    Some((kind, title))
}

fn is_prompt_close(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with(":::") && trimmed[3..].trim().is_empty()
}

fn render_markdown_segment(
    text: &str,
    ctx: &mut Ctx,
    raw_blocks: &mut Vec<String>,
    out: &mut String,
) {
    let (protected, mut blocks) = protect_raw_code_blocks(text, raw_blocks.len());
    raw_blocks.append(&mut blocks);
    let renderer = Renderer::new(ctx);
    out.push_str(&renderer.run(&protected));
}

fn render_with(body: &str, mut ctx: Ctx) -> (String, Vec<TocEntry>, HashMap<String, usize>) {
    let mut html = String::new();
    let mut raw_blocks: Vec<String> = Vec::new();
    for segment in split_prompts(body) {
        match segment {
            Segment::Markdown(text) => {
                render_markdown_segment(&text, &mut ctx, &mut raw_blocks, &mut html);
            }
            Segment::Prompt { kind, title, body } => {
                html.push_str(&format!(
                    "<blockquote type=\"{kind}\" class=\"prompt-{kind}\">\n"
                ));
                if !title.is_empty() {
                    html.push_str(&format!("<p><strong>{}</strong></p>", escape_html(&title)));
                }
                render_markdown_segment(&body, &mut ctx, &mut raw_blocks, &mut html);
                html.push_str("</blockquote>\n");
            }
        }
    }
    let upgraded = upgrade_html(&html, &mut ctx);
    let html = restore_raw_code_blocks(&upgraded, &raw_blocks);
    (html, ctx.toc, ctx.slug_count)
}

/// 单文件无状态渲染：与任务里的 Node oracle（每文件 `createRenderer()`）对齐。
pub fn render(body: &str) -> (String, Vec<TocEntry>) {
    let (html, toc, _) = render_with(body, Ctx::default());
    (html, toc)
}

/// 跨文章共享标题去重计数：与 Node loadContent 复用一个 renderer 的行为对齐。
/// content.rs 的文章循环应使用本函数；页面循环继续用无状态的 `render`。
pub fn render_shared(body: &str) -> (String, Vec<TocEntry>) {
    let mut guard = SHARED_SLUG_COUNTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let counts = guard.take().unwrap_or_default();
    let (html, toc, counts) = render_with(
        body,
        Ctx {
            toc: Vec::new(),
            slug_count: counts,
        },
    );
    *guard = Some(counts);
    (html, toc)
}

/// 清空 `render_shared` 的跨文章计数（测试/多次构建时用）。
#[allow(dead_code)]
pub fn reset_shared() {
    *SHARED_SLUG_COUNTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(HashMap::new());
}
