pub mod emoji;
pub mod file_type;
pub mod profiles;

use std::path::Path;

pub use emoji::EmojiMapper;
pub use file_type::FileType;

/// A profile defines how a particular file type should be displayed and processed
pub trait Profile {
    /// The file type this profile handles
    fn file_type(&self) -> FileType;

    /// The default emoji for this file type
    fn emoji(&self) -> &str;

    /// The name of this profile (e.g., "Rust", "Python")
    fn name(&self) -> &str;

    /// File extensions this profile matches (without the dot)
    fn extensions(&self) -> &[&str];

    /// Check if this profile matches a given path
    fn matches(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.extensions().contains(&ext_str);
            }
        }
        false
    }

    /// Whether to count lines of code for this file type
    fn should_count_lines(&self) -> bool {
        true
    }
}

/// Registry for all available profiles
pub struct ProfileRegistry {
    profiles: Vec<Box<dyn Profile>>,
}

impl ProfileRegistry {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    pub fn register(&mut self, profile: Box<dyn Profile>) {
        self.profiles.push(profile);
    }

    pub fn find_profile(&self, path: &Path) -> Option<&dyn Profile> {
        for profile in &self.profiles {
            if profile.matches(path) {
                return Some(profile.as_ref());
            }
        }
        None
    }

    pub fn classify_file(&self, path: &Path) -> FileType {
        if let Some(profile) = self.find_profile(path) {
            profile.file_type()
        } else {
            FileType::classify_path(path)
        }
    }
}

impl Default for ProfileRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        profiles::register_builtin_profiles(&mut registry);
        registry
    }
}
