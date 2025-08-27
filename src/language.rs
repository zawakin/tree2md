use std::path::Path;

pub struct Lang {
    pub ext: &'static str,
    pub name: &'static str,
    pub comment_fn: fn(&str) -> String,
}

impl Lang {
    pub fn to_comment(&self, s: &str) -> String {
        (self.comment_fn)(s)
    }
}

pub const LANGS: &[Lang] = &[
    Lang {
        ext: ".go",
        name: "go",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".py",
        name: "python",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".sh",
        name: "shell",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".js",
        name: "javascript",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".ts",
        name: "typescript",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".tsx",
        name: "tsx",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".html",
        name: "html",
        comment_fn: |s| format!("<!-- {} -->", s),
    },
    Lang {
        ext: ".css",
        name: "css",
        comment_fn: |s| format!("/* {} */", s),
    },
    Lang {
        ext: ".scss",
        name: "scss",
        comment_fn: |s| format!("/* {} */", s),
    },
    Lang {
        ext: ".sass",
        name: "sass",
        comment_fn: |s| format!("/* {} */", s),
    },
    Lang {
        ext: ".sql",
        name: "sql",
        comment_fn: |s| format!("-- {}", s),
    },
    Lang {
        ext: ".rs",
        name: "rust",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".toml",
        name: "toml",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".yaml",
        name: "yaml",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".yml",
        name: "yaml",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".json",
        name: "json",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".md",
        name: "markdown",
        comment_fn: |s| format!("<!-- {} -->", s),
    },
];

pub fn detect_lang(filename: &str) -> Option<&'static Lang> {
    let path = Path::new(filename);
    let ext = path.extension()?.to_str()?;
    let ext_with_dot = format!(".{}", ext.to_lowercase());

    LANGS.iter().find(|lang| lang.ext == ext_with_dot)
}
