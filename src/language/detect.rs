use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Lang {
    pub ext: &'static str,
    pub name: &'static str,
    pub comment_prefix: Option<&'static str>,
    pub comment_wrap: Option<(&'static str, &'static str)>,
}

impl PartialEq for Lang {
    fn eq(&self, other: &Self) -> bool {
        self.ext == other.ext && self.name == other.name
    }
}

pub static LANG_BY_EXT: Lazy<HashMap<&'static str, Lang>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // Programming languages
    m.insert("go", Lang {
        ext: "go",
        name: "go",
        comment_prefix: Some("// "),
        comment_wrap: None,
    });
    m.insert("py", Lang {
        ext: "py",
        name: "python",
        comment_prefix: Some("# "),
        comment_wrap: None,
    });
    m.insert("rs", Lang {
        ext: "rs",
        name: "rust",
        comment_prefix: Some("// "),
        comment_wrap: None,
    });
    m.insert("js", Lang {
        ext: "js",
        name: "javascript",
        comment_prefix: Some("// "),
        comment_wrap: None,
    });
    m.insert("ts", Lang {
        ext: "ts",
        name: "typescript",
        comment_prefix: Some("// "),
        comment_wrap: None,
    });
    m.insert("tsx", Lang {
        ext: "tsx",
        name: "tsx",
        comment_prefix: Some("// "),
        comment_wrap: None,
    });
    
    // Shell scripts
    m.insert("sh", Lang {
        ext: "sh",
        name: "shell",
        comment_prefix: Some("# "),
        comment_wrap: None,
    });
    
    // Web technologies
    m.insert("html", Lang {
        ext: "html",
        name: "html",
        comment_prefix: None,
        comment_wrap: Some(("<!-- ", " -->")),
    });
    m.insert("css", Lang {
        ext: "css",
        name: "css",
        comment_prefix: None,
        comment_wrap: Some(("/* ", " */")),
    });
    m.insert("scss", Lang {
        ext: "scss",
        name: "scss",
        comment_prefix: None,
        comment_wrap: Some(("/* ", " */")),
    });
    m.insert("sass", Lang {
        ext: "sass",
        name: "sass",
        comment_prefix: None,
        comment_wrap: Some(("/* ", " */")),
    });
    
    // Data/Config files
    m.insert("json", Lang {
        ext: "json",
        name: "json",
        comment_prefix: None,
        comment_wrap: None,
    });
    m.insert("toml", Lang {
        ext: "toml",
        name: "toml",
        comment_prefix: Some("# "),
        comment_wrap: None,
    });
    m.insert("yaml", Lang {
        ext: "yaml",
        name: "yaml",
        comment_prefix: Some("# "),
        comment_wrap: None,
    });
    m.insert("yml", Lang {
        ext: "yml",
        name: "yaml",
        comment_prefix: Some("# "),
        comment_wrap: None,
    });
    m.insert("sql", Lang {
        ext: "sql",
        name: "sql",
        comment_prefix: Some("-- "),
        comment_wrap: None,
    });
    m.insert("md", Lang {
        ext: "md",
        name: "markdown",
        comment_prefix: None,
        comment_wrap: Some(("<!-- ", " -->")),
    });
    
    m
});

pub fn detect_lang(filename: &str) -> Option<&'static Lang> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())?;
    
    LANG_BY_EXT.get(ext.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_lang() {
        assert_eq!(detect_lang("test.rs").map(|l| l.name), Some("rust"));
        assert_eq!(detect_lang("test.go").map(|l| l.name), Some("go"));
        assert_eq!(detect_lang("test.py").map(|l| l.name), Some("python"));
        assert_eq!(detect_lang("test.unknown"), None);
        assert_eq!(detect_lang("test.JSON").map(|l| l.name), Some("json"));
        assert_eq!(detect_lang("TEST.RS").map(|l| l.name), Some("rust"));
    }
    
    #[test]
    fn test_lang_equality() {
        let lang1 = &LANG_BY_EXT["rs"];
        let lang2 = &LANG_BY_EXT["rs"];
        assert_eq!(lang1, lang2);
    }
}