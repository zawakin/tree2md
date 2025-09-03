use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    // Directories
    Directory,

    // Programming Languages
    Rust,
    Python,
    Go,
    JavaScript,
    TypeScript,
    Java,
    CSharp,
    CPlusPlus,
    C,
    Swift,
    Kotlin,
    Ruby,
    Php,
    Shell,

    // Documentation
    Markdown,
    Text,

    // Configuration
    Json,
    Yaml,
    Toml,
    Xml,
    Ini,

    // Special files
    License,
    Ignore,
    Lock,
    Dockerfile,
    Makefile,

    // Testing
    Test,

    // Unknown
    Unknown,
}

impl FileType {
    /// Get the default emoji for this file type
    pub fn default_emoji(&self) -> &str {
        match self {
            // Directories
            FileType::Directory => "📁",

            // Programming Languages
            FileType::Rust => "🦀",
            FileType::Python => "🐍",
            FileType::Go => "🐹",
            FileType::JavaScript => "✨",
            FileType::TypeScript => "✨",
            FileType::Java => "☕",
            FileType::CSharp => "💜",
            FileType::CPlusPlus => "🔧",
            FileType::C => "🔧",
            FileType::Swift => "🐦",
            FileType::Kotlin => "🦊",
            FileType::Ruby => "💎",
            FileType::Php => "🐘",
            FileType::Shell => "🐚",

            // Documentation
            FileType::Markdown => "📘",
            FileType::Text => "📄",

            // Configuration
            FileType::Json => "⚙️",
            FileType::Yaml => "⚙️",
            FileType::Toml => "⚙️",
            FileType::Xml => "⚙️",
            FileType::Ini => "⚙️",

            // Special files
            FileType::License => "📜",
            FileType::Ignore => "🗂",
            FileType::Lock => "📦",
            FileType::Dockerfile => "🐳",
            FileType::Makefile => "🔨",

            // Testing
            FileType::Test => "🧪",

            // Unknown
            FileType::Unknown => "📄",
        }
    }

    /// Get a display name for this file type
    pub fn display_name(&self) -> &str {
        match self {
            FileType::Directory => "Directory",
            FileType::Rust => "Rust",
            FileType::Python => "Python",
            FileType::Go => "Go",
            FileType::JavaScript => "JavaScript",
            FileType::TypeScript => "TypeScript",
            FileType::Java => "Java",
            FileType::CSharp => "C#",
            FileType::CPlusPlus => "C++",
            FileType::C => "C",
            FileType::Swift => "Swift",
            FileType::Kotlin => "Kotlin",
            FileType::Ruby => "Ruby",
            FileType::Php => "PHP",
            FileType::Shell => "Shell",
            FileType::Markdown => "Markdown",
            FileType::Text => "Text",
            FileType::Json => "JSON",
            FileType::Yaml => "YAML",
            FileType::Toml => "TOML",
            FileType::Xml => "XML",
            FileType::Ini => "INI",
            FileType::License => "License",
            FileType::Ignore => "Ignore",
            FileType::Lock => "Lock",
            FileType::Dockerfile => "Docker",
            FileType::Makefile => "Make",
            FileType::Test => "Test",
            FileType::Unknown => "Unknown",
        }
    }

    /// Classify a file based on its path (fallback when no profile matches)
    pub fn classify_path(path: &Path) -> Self {
        if path.is_dir() {
            return FileType::Directory;
        }

        // Check file name for special files
        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy().to_lowercase();

            // Special file names
            if name == "dockerfile" || name.starts_with("dockerfile.") {
                return FileType::Dockerfile;
            }
            if name == "makefile" || name == "gnumakefile" {
                return FileType::Makefile;
            }
            if name == "license" || name == "licence" || name.starts_with("license.") {
                return FileType::License;
            }
            if name == ".gitignore" || name == ".dockerignore" || name.ends_with("ignore") {
                return FileType::Ignore;
            }
            if name.ends_with(".lock") || name == "package-lock.json" || name == "cargo.lock" {
                return FileType::Lock;
            }

            // Test files
            if name.contains("test") || name.contains("spec") {
                return FileType::Test;
            }
        }

        // Check extension
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return match ext_str.to_lowercase().as_str() {
                    // Programming languages
                    "rs" => FileType::Rust,
                    "py" | "pyw" => FileType::Python,
                    "go" => FileType::Go,
                    "js" | "mjs" | "cjs" => FileType::JavaScript,
                    "ts" | "tsx" => FileType::TypeScript,
                    "java" => FileType::Java,
                    "cs" => FileType::CSharp,
                    "cpp" | "cc" | "cxx" | "hpp" | "hxx" => FileType::CPlusPlus,
                    "c" | "h" => FileType::C,
                    "swift" => FileType::Swift,
                    "kt" | "kts" => FileType::Kotlin,
                    "rb" => FileType::Ruby,
                    "php" => FileType::Php,
                    "sh" | "bash" | "zsh" | "fish" => FileType::Shell,

                    // Documentation
                    "md" | "markdown" => FileType::Markdown,
                    "txt" | "text" => FileType::Text,

                    // Configuration
                    "json" => FileType::Json,
                    "yaml" | "yml" => FileType::Yaml,
                    "toml" => FileType::Toml,
                    "xml" => FileType::Xml,
                    "ini" | "cfg" | "conf" => FileType::Ini,

                    _ => FileType::Unknown,
                };
            }
        }

        FileType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_emoji() {
        assert_eq!(FileType::Rust.default_emoji(), "🦀");
        assert_eq!(FileType::Python.default_emoji(), "🐍");
        assert_eq!(FileType::Go.default_emoji(), "🐹");
        assert_eq!(FileType::JavaScript.default_emoji(), "✨");
        assert_eq!(FileType::Directory.default_emoji(), "📁");
        assert_eq!(FileType::Markdown.default_emoji(), "📘");
        assert_eq!(FileType::Test.default_emoji(), "🧪");
        assert_eq!(FileType::Unknown.default_emoji(), "📄");
    }

    #[test]
    fn test_display_name() {
        assert_eq!(FileType::Rust.display_name(), "Rust");
        assert_eq!(FileType::CPlusPlus.display_name(), "C++");
        assert_eq!(FileType::CSharp.display_name(), "C#");
        assert_eq!(FileType::Directory.display_name(), "Directory");
    }

    #[test]
    fn test_classify_rust_files() {
        assert_eq!(
            FileType::classify_path(Path::new("main.rs")),
            FileType::Rust
        );
        assert_eq!(FileType::classify_path(Path::new("lib.rs")), FileType::Rust);
        assert_eq!(
            FileType::classify_path(Path::new("src/module.rs")),
            FileType::Rust
        );
    }

    #[test]
    fn test_classify_python_files() {
        assert_eq!(
            FileType::classify_path(Path::new("script.py")),
            FileType::Python
        );
        assert_eq!(
            FileType::classify_path(Path::new("app.pyw")),
            FileType::Python
        );
    }

    #[test]
    fn test_classify_javascript_typescript() {
        assert_eq!(
            FileType::classify_path(Path::new("app.js")),
            FileType::JavaScript
        );
        assert_eq!(
            FileType::classify_path(Path::new("module.mjs")),
            FileType::JavaScript
        );
        assert_eq!(
            FileType::classify_path(Path::new("common.cjs")),
            FileType::JavaScript
        );
        assert_eq!(
            FileType::classify_path(Path::new("component.ts")),
            FileType::TypeScript
        );
        assert_eq!(
            FileType::classify_path(Path::new("component.tsx")),
            FileType::TypeScript
        );
    }

    #[test]
    fn test_classify_special_files() {
        assert_eq!(
            FileType::classify_path(Path::new("Dockerfile")),
            FileType::Dockerfile
        );
        assert_eq!(
            FileType::classify_path(Path::new("dockerfile")),
            FileType::Dockerfile
        );
        assert_eq!(
            FileType::classify_path(Path::new("Dockerfile.prod")),
            FileType::Dockerfile
        );
        assert_eq!(
            FileType::classify_path(Path::new("Makefile")),
            FileType::Makefile
        );
        assert_eq!(
            FileType::classify_path(Path::new("makefile")),
            FileType::Makefile
        );
        assert_eq!(
            FileType::classify_path(Path::new("LICENSE")),
            FileType::License
        );
        assert_eq!(
            FileType::classify_path(Path::new("LICENSE.txt")),
            FileType::License
        );
        assert_eq!(
            FileType::classify_path(Path::new(".gitignore")),
            FileType::Ignore
        );
        assert_eq!(
            FileType::classify_path(Path::new(".dockerignore")),
            FileType::Ignore
        );
    }

    #[test]
    fn test_classify_lock_files() {
        assert_eq!(
            FileType::classify_path(Path::new("Cargo.lock")),
            FileType::Lock
        );
        assert_eq!(
            FileType::classify_path(Path::new("package-lock.json")),
            FileType::Lock
        );
        assert_eq!(
            FileType::classify_path(Path::new("yarn.lock")),
            FileType::Lock
        );
    }

    #[test]
    fn test_classify_test_files() {
        assert_eq!(
            FileType::classify_path(Path::new("test_main.py")),
            FileType::Test
        );
        assert_eq!(
            FileType::classify_path(Path::new("main_test.go")),
            FileType::Test
        );
        assert_eq!(
            FileType::classify_path(Path::new("spec_helper.rb")),
            FileType::Test
        );
    }

    #[test]
    fn test_classify_config_files() {
        assert_eq!(
            FileType::classify_path(Path::new("config.json")),
            FileType::Json
        );
        assert_eq!(
            FileType::classify_path(Path::new("config.yaml")),
            FileType::Yaml
        );
        assert_eq!(
            FileType::classify_path(Path::new("config.yml")),
            FileType::Yaml
        );
        assert_eq!(
            FileType::classify_path(Path::new("Cargo.toml")),
            FileType::Toml
        );
        assert_eq!(FileType::classify_path(Path::new("pom.xml")), FileType::Xml);
        assert_eq!(
            FileType::classify_path(Path::new("config.ini")),
            FileType::Ini
        );
    }

    #[test]
    fn test_classify_documentation() {
        assert_eq!(
            FileType::classify_path(Path::new("README.md")),
            FileType::Markdown
        );
        assert_eq!(
            FileType::classify_path(Path::new("notes.txt")),
            FileType::Text
        );
    }

    #[test]
    fn test_classify_unknown() {
        assert_eq!(
            FileType::classify_path(Path::new("unknown.xyz")),
            FileType::Unknown
        );
        assert_eq!(
            FileType::classify_path(Path::new("file_without_extension")),
            FileType::Unknown
        );
    }

    #[test]
    fn test_case_insensitive_classification() {
        assert_eq!(
            FileType::classify_path(Path::new("MAIN.RS")),
            FileType::Rust
        );
        assert_eq!(
            FileType::classify_path(Path::new("Script.PY")),
            FileType::Python
        );
        assert_eq!(
            FileType::classify_path(Path::new("README.MD")),
            FileType::Markdown
        );
    }
}
