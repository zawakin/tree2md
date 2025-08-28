use super::detect::Lang;

pub fn to_comment(lang: &Lang, msg: &str) -> String {
    if let Some((open, close)) = lang.comment_wrap {
        format!("{}{}{}", open, msg, close)
    } else if let Some(prefix) = lang.comment_prefix {
        format!("{}{}", prefix, msg)
    } else {
        // Fallback for languages without comment syntax (like JSON)
        // This should not be called for JSON in practice, as we handle it specially
        format!("// {}", msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::detect::LANG_BY_EXT;

    #[test]
    fn test_to_comment() {
        let rust_lang = &LANG_BY_EXT["rs"];
        assert_eq!(to_comment(rust_lang, "test"), "// test");
        
        let python_lang = &LANG_BY_EXT["py"];
        assert_eq!(to_comment(python_lang, "test"), "# test");
        
        let html_lang = &LANG_BY_EXT["html"];
        assert_eq!(to_comment(html_lang, "test"), "<!-- test -->");
        
        let css_lang = &LANG_BY_EXT["css"];
        assert_eq!(to_comment(css_lang, "test"), "/* test */");
    }
}