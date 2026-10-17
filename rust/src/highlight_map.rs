//! syntect（TextMate scope）→ highlight.js 类名映射。
//!
//! 主题 `vendor/chirpy/main.css` 的配色选择器全部是 `.hljs-*`，所以这里把
//! syntect 的 scope 归一到 highlight.js 11 的 token 名称。返回的字符串直接
//! 作为 `class` 属性值，例如 `hljs-title function_`（主题用
//! `.hljs-title.function_` 选择器）。
//!
//! 匹配规则：scope 按 `.` 分段，前缀必须落在边界上（`string` 命中
//! `string.quoted.double`，但 `stringly` 不命中 `string`）。调用方从 scope
//! 栈顶向下查找，因此更具体的子 scope 优先。

fn has_prefix(scope: &str, prefix: &str) -> bool {
    scope == prefix
        || scope
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('.'))
}

fn has_suffix(scope: &str, suffix: &str) -> bool {
    scope == suffix
        || scope
            .strip_suffix(suffix)
            .is_some_and(|head| head.ends_with('.'))
}

fn is_css_scope(scope: &str) -> bool {
    ["css", "scss", "sass", "less", "styl"]
        .iter()
        .any(|lang| has_suffix(scope, lang))
}

/// 把一个 TextMate scope 映射成 highlight.js 类名；没有合适映射时返回 `None`。
pub fn class_for_scope(scope: &str) -> Option<&'static str> {
    let css = is_css_scope(scope);

    // ---- CSS / SCSS / Less -------------------------------------------------
    if css {
        if has_prefix(scope, "entity.other.attribute-name.class") {
            return Some("hljs-selector-class");
        }
        if has_prefix(scope, "entity.other.attribute-name.id") {
            return Some("hljs-selector-id");
        }
        if has_prefix(scope, "entity.other.attribute-name.pseudo-class")
            || has_prefix(scope, "entity.other.attribute-name.pseudo-element")
        {
            return Some("hljs-selector-pseudo");
        }
        if has_prefix(scope, "entity.other.attribute-name.attribute") {
            return Some("hljs-selector-attr");
        }
        if has_prefix(scope, "entity.name.tag") {
            return Some("hljs-selector-tag");
        }
        if has_prefix(scope, "support.type.property-name")
            || has_prefix(scope, "meta.property-name")
        {
            return Some("hljs-attribute");
        }
        if has_prefix(scope, "support.constant.property-value") {
            return Some("hljs-literal");
        }
        if has_prefix(scope, "keyword.other.unit") {
            return Some("hljs-number");
        }
    }

    // ---- YAML / JSON 键 ----------------------------------------------------
    // YAML 的键在 syntect 里是 entity.name.tag.yaml；主题按 hljs-attr 上色。
    if has_suffix(scope, "yaml") && has_prefix(scope, "entity.name.tag") {
        return Some("hljs-attr");
    }
    if has_prefix(scope, "support.type.property-name") {
        return Some("hljs-attr");
    }

    // ---- HTML / XML 标签与属性 --------------------------------------------
    if has_prefix(scope, "punctuation.definition.tag") {
        return Some("hljs-tag");
    }
    if has_prefix(scope, "entity.name.tag") {
        return Some("hljs-name");
    }
    if has_prefix(scope, "entity.other.attribute-name") {
        return Some("hljs-attr");
    }

    // ---- 注解 / 预处理 / 装饰器 -------------------------------------------
    if has_prefix(scope, "entity.name.type.annotation")
        || has_prefix(scope, "storage.type.annotation")
        || has_prefix(scope, "punctuation.definition.annotation")
        || has_prefix(scope, "meta.annotation")
        || has_prefix(scope, "meta.decorator")
        || has_prefix(scope, "meta.preprocessor")
        || has_prefix(scope, "meta.directive")
        || has_prefix(scope, "meta.diff.header")
        || has_prefix(scope, "meta.diff.range")
    {
        return Some("hljs-meta");
    }

    // ---- Markdown / Diff ---------------------------------------------------
    if has_prefix(scope, "markup.heading") || has_prefix(scope, "punctuation.definition.heading") {
        return Some("hljs-section");
    }
    if has_prefix(scope, "markup.bold") {
        return Some("hljs-strong");
    }
    if has_prefix(scope, "markup.italic") {
        return Some("hljs-emphasis");
    }
    if has_prefix(scope, "markup.inserted") {
        return Some("hljs-addition");
    }
    if has_prefix(scope, "markup.deleted") {
        return Some("hljs-deletion");
    }
    if has_prefix(scope, "markup.changed") {
        return Some("hljs-meta");
    }
    if has_prefix(scope, "markup.quote") {
        return Some("hljs-quote");
    }
    if has_prefix(scope, "markup.list") {
        return Some("hljs-bullet");
    }
    if has_prefix(scope, "markup.raw") {
        return Some("hljs-code");
    }
    if has_prefix(scope, "string.other.link") || has_prefix(scope, "markup.underline.link") {
        return Some("hljs-string");
    }

    // ---- 名字 / 类型 / 函数 ------------------------------------------------
    if has_prefix(scope, "entity.other.inherited-class") {
        return Some("hljs-title class_ inherited__");
    }
    if has_prefix(scope, "entity.name.function") {
        return Some("hljs-title function_");
    }
    if has_prefix(scope, "entity.name.type.class") || has_prefix(scope, "entity.name.class") {
        return Some("hljs-title class_");
    }
    if has_prefix(scope, "entity.name.type") || has_prefix(scope, "entity.name.namespace") {
        return Some("hljs-type");
    }
    if has_prefix(scope, "entity.name.section") {
        return Some("hljs-section");
    }

    // ---- 内建 / 库 ---------------------------------------------------------
    if has_prefix(scope, "support.function.construct") {
        // PHP 的 echo/isset 等在高亮里按 keyword 处理。
        return Some("hljs-keyword");
    }
    if has_prefix(scope, "support.function") {
        return Some("hljs-built_in");
    }
    if has_prefix(scope, "support.class") || has_prefix(scope, "support.type") {
        return Some("hljs-type");
    }
    if has_prefix(scope, "support.constant") {
        return Some("hljs-literal");
    }
    if has_prefix(scope, "support.variable") {
        return Some("hljs-variable");
    }

    // ---- 变量 / 参数 -------------------------------------------------------
    if has_prefix(scope, "variable.parameter") {
        return Some("hljs-params");
    }
    if has_prefix(scope, "variable.language") {
        return Some("hljs-variable language_");
    }
    if has_prefix(scope, "variable") {
        return Some("hljs-variable");
    }

    // ---- 常量 / 数字 / 符号 -----------------------------------------------
    if has_prefix(scope, "constant.numeric") {
        return Some("hljs-number");
    }
    if has_prefix(scope, "constant.character.escape") {
        return Some("hljs-string");
    }
    if has_prefix(scope, "constant.other.symbol") || has_prefix(scope, "constant.other.reference") {
        return Some("hljs-symbol");
    }
    if has_prefix(scope, "constant.other.color") {
        return Some("hljs-number");
    }
    if has_prefix(scope, "constant.language") {
        return Some("hljs-literal");
    }
    if has_prefix(scope, "constant") {
        return Some("hljs-literal");
    }

    // ---- 关键字 / 存储类型 -------------------------------------------------
    if has_prefix(scope, "keyword.operator") {
        return Some("hljs-operator");
    }
    if has_prefix(scope, "punctuation.operator") {
        return Some("hljs-operator");
    }
    if has_prefix(scope, "keyword") {
        return Some("hljs-keyword");
    }
    // TextMate 的 storage.type 在 Rust/Java 等语法里也覆盖 fn/let/int/class 这类
    // 关键字（highlight.js 统一算 keyword）；真正的类型名走 entity.name.type /
    // support.type / support.class。
    if has_prefix(scope, "storage.type") {
        return Some("hljs-keyword");
    }
    if has_prefix(scope, "storage.modifier") {
        return Some("hljs-keyword");
    }
    if has_prefix(scope, "storage") {
        return Some("hljs-keyword");
    }

    // ---- 字符串 / 注释 / 正则 ---------------------------------------------
    if has_prefix(scope, "string.regexp") || has_prefix(scope, "regexp") {
        return Some("hljs-regexp");
    }
    if has_prefix(scope, "string") {
        return Some("hljs-string");
    }
    if has_prefix(scope, "comment") {
        return Some("hljs-comment");
    }

    // ---- 插值 / 标点 -------------------------------------------------------
    if has_prefix(scope, "meta.interpolation")
        || has_prefix(scope, "punctuation.section.interpolation")
    {
        return Some("hljs-subst");
    }
    if has_prefix(scope, "punctuation.section.embedded.begin")
        || has_prefix(scope, "punctuation.section.embedded.end")
    {
        // PHP / ERB 的 <?php ?> 之类。
        return Some("hljs-meta");
    }
    if has_prefix(scope, "punctuation.section.embedded") {
        return Some("hljs-subst");
    }
    if has_prefix(scope, "punctuation.definition.string") {
        return Some("hljs-string");
    }
    if has_prefix(scope, "punctuation.definition.comment") {
        return Some("hljs-comment");
    }
    if has_prefix(scope, "punctuation.definition.variable") {
        return Some("hljs-variable");
    }
    if has_prefix(scope, "punctuation") {
        return Some("hljs-punctuation");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::class_for_scope;

    #[test]
    fn maps_required_hljs_classes() {
        let cases = [
            ("keyword.control.rust", "hljs-keyword"),
            ("string.quoted.double.java", "hljs-string"),
            ("constant.numeric.java", "hljs-number"),
            ("comment.line.double-slash.rust", "hljs-comment"),
            ("constant.language.python", "hljs-literal"),
            ("support.function.builtin.python", "hljs-built_in"),
            ("support.type.java", "hljs-type"),
            ("storage.type.java", "hljs-keyword"),
            ("entity.name.function.java", "hljs-title function_"),
            ("entity.name.type.class.java", "hljs-title class_"),
            ("entity.other.attribute-name.html", "hljs-attr"),
            ("variable.other.rust", "hljs-variable"),
            ("variable.parameter.java", "hljs-params"),
            ("meta.annotation.java", "hljs-meta"),
            ("entity.name.tag.html", "hljs-name"),
            ("entity.name.tag.css", "hljs-selector-tag"),
            (
                "entity.other.attribute-name.class.css",
                "hljs-selector-class",
            ),
            ("constant.other.symbol.ruby", "hljs-symbol"),
            ("meta.interpolation.js", "hljs-subst"),
            ("string.regexp.js", "hljs-regexp"),
            ("markup.italic.markdown", "hljs-emphasis"),
            ("markup.bold.markdown", "hljs-strong"),
            ("markup.inserted.diff", "hljs-addition"),
            ("markup.deleted.diff", "hljs-deletion"),
        ];
        for (scope, expected) in cases {
            assert_eq!(class_for_scope(scope), Some(expected), "scope={scope}");
        }
    }

    #[test]
    fn disambiguates_css_html_and_yaml_tags() {
        assert_eq!(
            class_for_scope("entity.name.tag.css"),
            Some("hljs-selector-tag")
        );
        assert_eq!(class_for_scope("entity.name.tag.html"), Some("hljs-name"));
        assert_eq!(class_for_scope("entity.name.tag.yaml"), Some("hljs-attr"));
        assert_eq!(
            class_for_scope("support.type.property-name.css"),
            Some("hljs-attribute")
        );
        assert_eq!(
            class_for_scope("support.type.property-name.json"),
            Some("hljs-attr")
        );
    }

    #[test]
    fn leaves_structural_scopes_unmapped() {
        for scope in [
            "source.rust",
            "text.html.basic",
            "meta.function.java",
            "meta.class.java",
        ] {
            assert_eq!(class_for_scope(scope), None, "scope={scope}");
        }
    }
}
