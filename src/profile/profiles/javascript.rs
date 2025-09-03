use crate::profile::{FileType, Profile};

pub struct JavaScriptProfile;

impl Profile for JavaScriptProfile {
    fn file_type(&self) -> FileType {
        FileType::JavaScript
    }

    fn emoji(&self) -> &str {
        "✨"
    }

    fn name(&self) -> &str {
        "JavaScript"
    }

    fn extensions(&self) -> &[&str] {
        &["js", "mjs", "cjs", "jsx"]
    }

    fn should_count_lines(&self) -> bool {
        true
    }
}

pub struct TypeScriptProfile;

impl Profile for TypeScriptProfile {
    fn file_type(&self) -> FileType {
        FileType::TypeScript
    }

    fn emoji(&self) -> &str {
        "✨"
    }

    fn name(&self) -> &str {
        "TypeScript"
    }

    fn extensions(&self) -> &[&str] {
        &["ts", "tsx", "mts", "cts"]
    }

    fn should_count_lines(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;
    use std::path::Path;

    #[test]
    fn test_javascript_profile() {
        let profile = JavaScriptProfile;
        assert!(profile.matches(Path::new("app.js")));
        assert!(profile.matches(Path::new("module.mjs")));
        assert!(profile.matches(Path::new("common.cjs")));
        assert!(!profile.matches(Path::new("component.ts"))); // TypeScript is separate
        assert!(!profile.matches(Path::new("main.rs")));
        assert_eq!(profile.name(), "JavaScript");
        assert_eq!(profile.emoji(), "✨");
    }
}
