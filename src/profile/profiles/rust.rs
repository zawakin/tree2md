use crate::profile::{FileType, Profile};

pub struct RustProfile;

impl Profile for RustProfile {
    fn file_type(&self) -> FileType {
        FileType::Rust
    }

    fn emoji(&self) -> &str {
        "🦀"
    }

    fn name(&self) -> &str {
        "Rust"
    }

    fn extensions(&self) -> &[&str] {
        &["rs"]
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
    fn test_rust_profile() {
        let profile = RustProfile;
        assert!(profile.matches(Path::new("main.rs")));
        assert!(profile.matches(Path::new("lib.rs")));
        assert!(!profile.matches(Path::new("main.py")));
        assert_eq!(profile.name(), "Rust");
        assert_eq!(profile.emoji(), "🦀");
    }
}
