use crate::profile::{FileType, Profile};

pub struct PythonProfile;

impl Profile for PythonProfile {
    fn file_type(&self) -> FileType {
        FileType::Python
    }

    fn emoji(&self) -> &str {
        "🐍"
    }

    fn name(&self) -> &str {
        "Python"
    }

    fn extensions(&self) -> &[&str] {
        &["py", "pyw", "pyi", "pyc", "pyo"]
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
    fn test_python_profile() {
        let profile = PythonProfile;
        assert!(profile.matches(Path::new("script.py")));
        assert!(profile.matches(Path::new("app.pyw")));
        assert!(!profile.matches(Path::new("main.rs")));
        assert_eq!(profile.name(), "Python");
        assert_eq!(profile.emoji(), "🐍");
    }
}
