// 代码高亮 token 层：syntect（TextMate 语法）→ highlight.js `hljs-*` HTML。
//
// `markdown.rs` 只负责 `<div class="highlight"><pre><code …>` 外壳，本模块只返回
// 转义后的 token HTML（不含 pre/code），接口保持：
//     pub fn highlight(lang: Option<&str>, code: &str) -> String
//
// 语言解析顺序：
//   1. 显式语言：大小写不敏感 + 常见别名（js/ts/sh/shell/sql/pg/pgsql…）；
//   2. 未标注 / `auto`：先看 shebang 等 first_line，再按首 token 查 syntax，
//      最后用 syntect 对常见语法做一次轻量打分（简化的 highlightAuto）；
//   3. 实在识别不了：按 highlight.js 的 escapeHTML 规则输出纯文本。
//
// 显式写了但 syntect 没有对应语法的语言（default-fancy 没有 powershell / prisma 等；
// typescript 会先退到 JavaScript）同样退化为纯文本，并汇总打印一条 warning。

#[path = "highlight_map.rs"]
mod highlight_map;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Once, OnceLock};
use syntect::parsing::{
    ParseState, ParsingError, Scope, ScopeStack, ScopeStackOp, SyntaxReference, SyntaxSet,
};
use syntect::util::LinesWithEndings;

static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
static HITS: AtomicU64 = AtomicU64::new(0);
static MISSES: AtomicU64 = AtomicU64::new(0);

/// 成功/退化为纯文本的代码块计数（examples/render.rs 的对照统计用）。
#[allow(dead_code)]
pub fn stats() -> (u64, u64) {
    (HITS.load(Ordering::Relaxed), MISSES.load(Ordering::Relaxed))
}

fn syntaxes() -> &'static SyntaxSet {
    SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// highlight.js 的 escapeHTML：`& < > " '`。
fn escape_into(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    escape_into(&mut out, text);
    out
}

fn push_text(out: &mut String, class: Option<&'static str>, text: &str) {
    if text.is_empty() {
        return;
    }
    match class {
        Some(class) => {
            out.push_str("<span class=\"");
            out.push_str(class);
            out.push_str("\">");
            escape_into(out, text);
            out.push_str("</span>");
        }
        None => escape_into(out, text),
    }
}

/// 从栈顶向下找第一个可映射的 scope。
///
/// 通用 `punctuation` 先记下不立即返回：这样字符串/注释内部的标点仍会继承
/// 父级 string/comment 的颜色，而不是被打回默认色。
fn class_for_stack(scopes: &[Scope]) -> Option<&'static str> {
    let mut punctuation = None;
    for scope in scopes.iter().rev() {
        let name = scope.build_string();
        match highlight_map::class_for_scope(&name) {
            Some("hljs-punctuation") => punctuation = Some("hljs-punctuation"),
            Some(class) => return Some(class),
            None => {}
        }
    }
    punctuation
}

/// 用给定语法把 `code` 渲染成 token HTML。换行原样保留。
fn render(syntax: &SyntaxReference, code: &str) -> Result<String, ParsingError> {
    let syntax_set = syntaxes();
    let mut state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut out = String::with_capacity(code.len() + code.len() / 2);

    for line in LinesWithEndings::from(code) {
        let ops = state.parse_line(line, syntax_set)?;
        let mut pos = 0usize;
        for (offset, op) in ops {
            let offset = offset.min(line.len());
            if offset > pos {
                push_text(
                    &mut out,
                    class_for_stack(stack.as_slice()),
                    &line[pos..offset],
                );
                pos = offset;
            }
            // ScopeStack 的 Pop 理论上可能与语法状态不一致；高亮层不应因此 panic。
            let _ = stack.apply(&op);
        }
        if pos < line.len() {
            push_text(&mut out, class_for_stack(stack.as_slice()), &line[pos..]);
        }
    }
    Ok(out)
}

fn normalize_lang(lang: &str) -> String {
    let mut lang = lang.trim();
    if let Some(rest) = lang.strip_prefix("language-") {
        lang = rest;
    }
    lang.split(|ch: char| ch.is_whitespace() || ch == ',' || ch == ';')
        .next()
        .unwrap_or("")
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

fn alias_tokens(lang: &str) -> &'static [&'static str] {
    match lang {
        "js" | "jsx" | "mjs" | "cjs" | "node" | "javascript" => &["javascript", "js"],
        // syntect 的 default-fancy 没有 TypeScript 语法；退到 JavaScript 仍能覆盖大部分 token。
        "ts" | "tsx" | "typescript" => &["typescript", "ts", "javascript", "js"],
        "sh" | "shell" | "bash" | "zsh" | "ksh" | "fish" | "console" | "terminal"
        | "shellsession" | "shell-script" | "shellscript" => &["bash", "sh", "shell"],
        "sql" | "pgsql" | "psql" | "postgres" | "postgresql" | "pg" | "mysql" | "mariadb"
        | "sqlite" | "sqlite3" => &["sql"],
        "powershell" | "ps1" | "pwsh" => &["powershell", "ps1", "pwsh"],
        "prisma" => &["prisma"],
        "yml" => &["yaml", "yml"],
        "md" | "mdx" => &["markdown", "md"],
        "htm" | "xhtml" => &["html", "htm"],
        "svg" => &["xml", "svg"],
        "c++" | "cpp" | "cxx" | "cc" | "hpp" | "hxx" => &["c++", "cpp", "cxx", "cc"],
        "c#" | "cs" | "csharp" => &["c#", "cs", "csharp"],
        "objective-c" | "objectivec" | "objc" | "m" => &["objective-c", "objc", "m"],
        "rs" => &["rust", "rs"],
        "py" | "python3" => &["python", "py"],
        "rb" => &["ruby", "rb"],
        "go" | "golang" => &["go", "golang"],
        "kt" | "kotlin" => &["kotlin", "kt"],
        "swift" => &["swift"],
        "make" | "makefile" | "gnumakefile" => &["makefile", "make"],
        "docker" | "dockerfile" => &["dockerfile", "docker"],
        "regex" => &["regexp", "regex"],
        "tex" | "latex" => &["latex", "tex"],
        "jsp" => &["jsp"],
        "bat" | "batch" | "cmd" | "batchfile" => &["bat", "batch"],
        "graphviz" | "dot" => &["dot", "graphviz (dot)"],
        "scss" | "sass" | "less" | "styl" => &["css", "scss", "sass", "less"],
        _ => &[],
    }
}

fn syntax_for_lang(lang: &str) -> Option<&'static SyntaxReference> {
    let syntax_set = syntaxes();
    for token in alias_tokens(lang) {
        if let Some(syntax) = syntax_set.find_syntax_by_token(token) {
            return Some(syntax);
        }
    }
    syntax_set
        .find_syntax_by_token(lang)
        .or_else(|| syntax_set.find_syntax_by_name(lang))
}

fn first_token(line: &str) -> Option<String> {
    let token = line.split_whitespace().next()?;
    let token = token.trim_matches(|ch: char| {
        !(ch.is_alphanumeric() || matches!(ch, '+' | '#' | '_' | '-' | '.'))
    });
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// 没有显式语言时，先用少量高置信度特征识别；避免把普通说明文字强行上色。
/// 特征都是“代码开头/行首”级别，不依赖猜测；识别不了的仍按纯文本处理。
fn detect_by_hints(code: &str) -> Option<&'static SyntaxReference> {
    let syntax_set = syntaxes();
    let trimmed = code.trim_start();
    let first_line = trimmed.lines().next().unwrap_or("").trim();
    let lower = trimmed.to_ascii_lowercase();
    let starts = |prefix: &str| first_line.starts_with(prefix);

    if starts("$ ")
        || first_line.contains("-bash-")
        || starts("export ")
        || starts("source ")
        || starts("sudo ")
        || starts("apt ")
        || starts("yum ")
        || starts("systemctl ")
        || trimmed.starts_with("#!/bin/")
    {
        return syntax_set.find_syntax_by_token("bash");
    }
    if starts("diff --git ") || starts("@@ ") || lower.contains("\n@@ ") {
        return syntax_set.find_syntax_by_token("diff");
    }
    if trimmed.starts_with("<!DOCTYPE")
        || trimmed.starts_with("<!doctype")
        || trimmed.starts_with("<html")
        || trimmed.starts_with("<div")
        || trimmed.starts_with("<span")
    {
        return syntax_set.find_syntax_by_token("html");
    }
    if (trimmed.starts_with('{') || trimmed.starts_with('[')) && trimmed.contains("\":") {
        return syntax_set.find_syntax_by_token("json");
    }
    if lower.starts_with("select ")
        || lower.starts_with("insert into ")
        || lower.starts_with("create table ")
        || lower.starts_with("alter table ")
        || lower.starts_with("delete from ")
        || lower.starts_with("update ")
    {
        return syntax_set.find_syntax_by_token("sql");
    }
    if starts("package main") && lower.contains("func ") {
        return syntax_set.find_syntax_by_token("go");
    }
    if (starts("package ") && first_line.ends_with(';'))
        || lower.contains("public class ")
        || lower.contains("import java.")
        || lower.contains("system.out.")
    {
        return syntax_set.find_syntax_by_token("java");
    }
    if starts("fn ")
        || starts("impl ")
        || starts("use ")
        || starts("let mut ")
        || lower.contains("\nfn ")
        || lower.contains("\nimpl ")
    {
        return syntax_set.find_syntax_by_token("rust");
    }
    if starts("def ")
        || starts("from ")
        || lower.contains("\ndef ")
        || lower.contains("\nfrom ")
        || lower.contains("if __name__")
    {
        return syntax_set.find_syntax_by_token("python");
    }
    if starts("const ")
        || starts("let ")
        || starts("var ")
        || starts("function ")
        || starts("import ")
        || lower.contains("console.log(")
        || lower.contains("=>")
    {
        return syntax_set.find_syntax_by_token("javascript");
    }
    if lower.starts_with("#include")
        || lower.contains("\n#include")
        || lower.contains("int main(")
        || lower.contains("std::")
    {
        return syntax_set.find_syntax_by_token("c++");
    }
    if lower.starts_with("<?php") {
        return syntax_set.find_syntax_by_token("php");
    }

    None
}

fn detect_syntax(code: &str) -> Option<&'static SyntaxReference> {
    let syntax_set = syntaxes();
    let first_line = code.lines().next().unwrap_or("");

    if let Some(syntax) = detect_by_hints(code) {
        return Some(syntax);
    }
    if let Some(syntax) = syntax_set.find_syntax_by_first_line(first_line) {
        return Some(syntax);
    }
    if let Some(token) = first_token(first_line)
        && let Some(syntax) = syntax_set.find_syntax_by_token(&token)
    {
        return Some(syntax);
    }

    detect_by_score(code)
}

/// syntect 没有 highlightAuto；对少量常见语法做轻量打分，模拟“未标注时自动识别”。
/// 只在没有显式语言时使用，得分门槛不高但要求至少命中一个强 token。
fn score_syntax(syntax: &SyntaxReference, code: &str) -> (u32, u32) {
    let syntax_set = syntaxes();
    let mut state = ParseState::new(syntax);
    let mut score = 0u32;
    let mut strong = 0u32;

    for line in LinesWithEndings::from(code).take(40) {
        let Ok(ops) = state.parse_line(line, syntax_set) else {
            return (0, 0);
        };
        for (_, op) in ops {
            let ScopeStackOp::Push(scope) = op else {
                continue;
            };
            let name = scope.build_string();
            let weight = match highlight_map::class_for_scope(&name) {
                Some("hljs-keyword" | "hljs-string" | "hljs-comment" | "hljs-number") => 3,
                Some(_) => 1,
                None => 0,
            };
            score += weight;
            if weight == 3 {
                strong += 1;
            }
        }
        if score >= 18 {
            break;
        }
    }
    (score, strong)
}

const AUTO_CANDIDATES: &[&str] = &[
    "bash",
    "java",
    "javascript",
    "python",
    "sql",
    "rust",
    "c",
    "c++",
    "php",
    "go",
];

fn detect_by_score(code: &str) -> Option<&'static SyntaxReference> {
    let syntax_set = syntaxes();
    let mut best: Option<(&SyntaxReference, u32)> = None;

    for token in AUTO_CANDIDATES {
        let Some(syntax) = syntax_set.find_syntax_by_token(token) else {
            continue;
        };
        let (score, strong) = score_syntax(syntax, code);
        if score < 4 || strong < 1 {
            continue;
        }
        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((syntax, score));
        }
    }

    best.map(|(syntax, _)| syntax)
}

fn warn_unknown_lang(lang: &str) {
    static WARN: Once = Once::new();
    WARN.call_once(|| {
        eprintln!(
            "warning: highlight: 未找到语言 {lang:?} 的 syntect syntax，已按纯文本转义；\
             构建中其他未知语言同样退化，不再逐块提示"
        );
    });
}

/// 返回 highlight.js 风格的 token HTML（已去掉内联 style、不含 pre/code 外壳）。
///
/// `lang` 为原始语言名（保留大小写）；`None` / 空串 / `auto` 表示自动识别。
pub fn highlight(lang: Option<&str>, code: &str) -> String {
    if code.is_empty() {
        return String::new();
    }

    let normalized = lang.map(normalize_lang).filter(|lang| !lang.is_empty());
    let syntax = match normalized.as_deref() {
        None | Some("auto" | "detect" | "guess") => detect_syntax(code),
        Some("plaintext" | "plain" | "text" | "txt" | "none") => {
            MISSES.fetch_add(1, Ordering::Relaxed);
            return escape_html(code);
        }
        Some(lang) => match syntax_for_lang(lang) {
            Some(syntax) => Some(syntax),
            None => {
                MISSES.fetch_add(1, Ordering::Relaxed);
                warn_unknown_lang(lang);
                return escape_html(code);
            }
        },
    };

    match syntax {
        Some(syntax) => match render(syntax, code) {
            Ok(html) => {
                HITS.fetch_add(1, Ordering::Relaxed);
                html
            }
            Err(_) => {
                MISSES.fetch_add(1, Ordering::Relaxed);
                escape_html(code)
            }
        },
        None => {
            MISSES.fetch_add(1, Ordering::Relaxed);
            escape_html(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::highlight;

    #[test]
    fn highlights_rust_tokens() {
        let html = highlight(Some("rust"), "fn main() {\n    let s = \"<&>\";\n}\n");
        assert!(html.contains("hljs-keyword"), "{html}");
        assert!(html.contains("hljs-title function_"), "{html}");
        assert!(html.contains("hljs-string"), "{html}");
        assert!(html.contains("&lt;&amp;&gt;"), "{html}");
        assert!(html.contains('\n'), "newlines must be preserved");
    }

    #[test]
    fn aliases_are_case_insensitive() {
        for (lang, marker) in [
            ("JS", "hljs-keyword"),
            ("Shell", "hljs-built_in"),
            ("SQL", "hljs-keyword"),
            ("Ts", "hljs-keyword"),
        ] {
            let html = highlight(Some(lang), "const value = 1; select * from t; echo ok");
            assert!(html.contains(marker), "lang={lang} html={html}");
        }
    }

    #[test]
    fn plaintext_is_escaped_without_spans() {
        let html = highlight(Some("plaintext"), "<&>\"'");
        assert_eq!(html, "&lt;&amp;&gt;&quot;&#x27;");
    }

    #[test]
    fn unknown_language_falls_back_to_plain_text() {
        let html = highlight(Some("prisma"), "model User { id Int @id }");
        assert!(!html.contains("<span"), "{html}");
        assert!(html.contains("model User"), "{html}");
    }

    #[test]
    fn detects_shebang_and_empty_language() {
        let html = highlight(None, "#!/bin/bash\necho \"$HOME\"\n");
        assert!(html.contains("hljs-comment"), "{html}");
        assert!(
            html.contains("hljs-built_in") || html.contains("hljs-title function_"),
            "{html}"
        );
    }

    #[test]
    fn empty_code_is_empty() {
        assert_eq!(highlight(None, ""), "");
        assert_eq!(highlight(Some("rust"), ""), "");
    }
}
