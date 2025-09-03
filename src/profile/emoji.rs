use crate::profile::FileType;
use std::collections::HashMap;
use std::path::Path;

/// Manages emoji assignments for files and directories
pub struct EmojiMapper {
    /// Custom emoji overrides by extension
    extension_overrides: HashMap<String, String>,

    /// Custom emoji overrides by file type
    type_overrides: HashMap<FileType, String>,

    /// Whether emojis are enabled
    enabled: bool,
}

impl EmojiMapper {
    pub fn new(enabled: bool) -> Self {
        Self {
            extension_overrides: HashMap::new(),
            type_overrides: HashMap::new(),
            enabled,
        }
    }

    /// Add a custom emoji override for a specific extension
    pub fn add_extension_override(&mut self, extension: String, emoji: String) {
        self.extension_overrides.insert(extension, emoji);
    }

    /// Add a custom emoji override for a file type
    pub fn add_type_override(&mut self, file_type: FileType, emoji: String) {
        self.type_overrides.insert(file_type, emoji);
    }

    /// Load emoji mappings from a TOML file
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mappings: toml::Value = toml::from_str(&content)?;

        if let Some(table) = mappings.as_table() {
            // Load extension mappings
            if let Some(extensions) = table.get("extensions").and_then(|v| v.as_table()) {
                for (ext, emoji) in extensions {
                    if let Some(emoji_str) = emoji.as_str() {
                        self.extension_overrides.insert(
                            ext.trim_start_matches('.').to_string(),
                            emoji_str.to_string(),
                        );
                    }
                }
            }

            // Load type mappings
            if let Some(types) = table.get("types").and_then(|v| v.as_table()) {
                for (type_name, emoji) in types {
                    if let Some(emoji_str) = emoji.as_str() {
                        // Convert string to FileType
                        let file_type = match type_name.to_lowercase().as_str() {
                            "rust" => FileType::Rust,
                            "python" => FileType::Python,
                            "go" => FileType::Go,
                            "javascript" | "js" => FileType::JavaScript,
                            "typescript" | "ts" => FileType::TypeScript,
                            "markdown" | "md" => FileType::Markdown,
                            "config" | "configuration" => FileType::Json, // Generic config
                            "test" | "tests" => FileType::Test,
                            _ => continue,
                        };
                        self.type_overrides.insert(file_type, emoji_str.to_string());
                    }
                }
            }
        }

        Ok(())
    }

    /// Get emoji for a file path
    pub fn get_emoji(&self, path: &Path, file_type: FileType) -> String {
        if !self.enabled {
            return String::new();
        }

        // Check extension overrides first
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if let Some(emoji) = self.extension_overrides.get(ext_str) {
                    return emoji.clone();
                }
            }
        }

        // Check type overrides
        if let Some(emoji) = self.type_overrides.get(&file_type) {
            return emoji.clone();
        }

        // Use default emoji for the file type
        file_type.default_emoji().to_string()
    }

    /// Parse CLI emoji arguments (format: ".ext=emoji" or "type=emoji")
    pub fn parse_cli_emoji(&mut self, arg: &str) {
        if let Some(eq_pos) = arg.find('=') {
            let (key, emoji) = arg.split_at(eq_pos);
            let emoji = &emoji[1..]; // Skip the '='

            if let Some(stripped) = key.strip_prefix('.') {
                // Extension override
                self.add_extension_override(stripped.to_string(), emoji.to_string());
            } else {
                // Try to parse as type
                let file_type = match key.to_lowercase().as_str() {
                    "rust" => FileType::Rust,
                    "python" | "py" => FileType::Python,
                    "go" => FileType::Go,
                    "javascript" | "js" => FileType::JavaScript,
                    "typescript" | "ts" => FileType::TypeScript,
                    "markdown" | "md" | "docs" => FileType::Markdown,
                    "test" | "tests" => FileType::Test,
                    "config" => FileType::Json,
                    _ => return, // Unknown type, skip
                };
                self.add_type_override(file_type, emoji.to_string());
            }
        }
    }
}

impl Default for EmojiMapper {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::FileType;
    use std::path::Path;

    #[test]
    fn test_emoji_mapper_disabled() {
        let mapper = EmojiMapper::new(false);
        assert_eq!(mapper.get_emoji(Path::new("main.rs"), FileType::Rust), "");
        assert_eq!(mapper.get_emoji(Path::new("test.py"), FileType::Python), "");
    }

    #[test]
    fn test_emoji_mapper_enabled() {
        let mapper = EmojiMapper::new(true);
        assert_eq!(mapper.get_emoji(Path::new("main.rs"), FileType::Rust), "🦀");
        assert_eq!(
            mapper.get_emoji(Path::new("test.py"), FileType::Python),
            "🐍"
        );
        assert_eq!(
            mapper.get_emoji(Path::new("app.js"), FileType::JavaScript),
            "✨"
        );
    }

    #[test]
    fn test_emoji_override_by_extension() {
        let mut mapper = EmojiMapper::new(true);
        mapper.add_extension_override("rs".to_string(), "🚀".to_string());

        assert_eq!(mapper.get_emoji(Path::new("main.rs"), FileType::Rust), "🚀");
        assert_eq!(
            mapper.get_emoji(Path::new("test.py"), FileType::Python),
            "🐍"
        );
    }

    #[test]
    fn test_emoji_override_by_type() {
        let mut mapper = EmojiMapper::new(true);
        mapper.add_type_override(FileType::Markdown, "📖".to_string());

        assert_eq!(
            mapper.get_emoji(Path::new("README.md"), FileType::Markdown),
            "📖"
        );
        assert_eq!(
            mapper.get_emoji(Path::new("script.py"), FileType::Python),
            "🐍"
        );
    }

    #[test]
    fn test_emoji_parse_cli() {
        let mut mapper = EmojiMapper::new(true);
        mapper.parse_cli_emoji(".rs=🚀");
        mapper.parse_cli_emoji(".py=🔥");

        assert_eq!(mapper.get_emoji(Path::new("main.rs"), FileType::Rust), "🚀");
        assert_eq!(
            mapper.get_emoji(Path::new("script.py"), FileType::Python),
            "🔥"
        );
    }

    #[test]
    fn test_parse_cli_emoji_invalid() {
        let mut mapper = EmojiMapper::new(true);

        // Invalid format should be ignored
        mapper.parse_cli_emoji("invalid");
        mapper.parse_cli_emoji("=emoji");
        mapper.parse_cli_emoji("pattern=");

        // Should still use default emojis
        assert_eq!(mapper.get_emoji(Path::new("main.rs"), FileType::Rust), "🦀");
    }
}
