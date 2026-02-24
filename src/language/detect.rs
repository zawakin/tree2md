use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct Lang {
    pub name: &'static str,
}

impl PartialEq for Lang {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub static LANG_BY_EXT: Lazy<HashMap<&'static str, Lang>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Programming languages
    m.insert("go", Lang { name: "go" });
    m.insert("py", Lang { name: "python" });
    m.insert("rs", Lang { name: "rust" });
    m.insert("js", Lang { name: "javascript" });
    m.insert("ts", Lang { name: "typescript" });
    m.insert("tsx", Lang { name: "tsx" });

    // Shell scripts
    m.insert("sh", Lang { name: "shell" });

    // Web technologies
    m.insert("html", Lang { name: "html" });
    m.insert("css", Lang { name: "css" });
    m.insert("scss", Lang { name: "scss" });
    m.insert("sass", Lang { name: "sass" });

    // Data/Config files
    m.insert("json", Lang { name: "json" });
    m.insert("toml", Lang { name: "toml" });
    m.insert("yaml", Lang { name: "yaml" });
    m.insert("yml", Lang { name: "yaml" });
    m.insert("sql", Lang { name: "sql" });
    m.insert("md", Lang { name: "markdown" });

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
