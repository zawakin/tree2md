use std::fs;
use std::io;
use std::path::Path;

/// Inject or update tree2md output in README.md
pub struct ReadmeInjector {
    content: String,
    tag_start: String,
    tag_end: String,
}

impl ReadmeInjector {
    /// Create a new injector with tree2md output content and custom tags
    pub fn new(tree_content: String, tag_start: String, tag_end: String) -> Self {
        Self {
            content: tree_content,
            tag_start,
            tag_end,
        }
    }

    /// Inject into README file, updating existing section if found
    pub fn inject(&self, readme_path: &Path) -> io::Result<()> {
        // Check if file exists
        if !readme_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", readme_path.display()),
            ));
        }

        // Read existing README
        let existing_content = fs::read_to_string(readme_path)?;

        // Check for existing tags - both must be present
        if !existing_content.contains(&self.tag_start) || !existing_content.contains(&self.tag_end) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Injection tags not found in {}. Please add '{}' and '{}' to the file where you want the tree to be inserted.",
                    readme_path.display(),
                    self.tag_start,
                    self.tag_end
                ),
            ));
        }

        // Update existing content
        let updated_content = self.update_existing(&existing_content)?;

        // Write updated content
        fs::write(readme_path, updated_content)?;
        Ok(())
    }

    /// Update existing tree2md section between tags
    fn update_existing(&self, content: &str) -> io::Result<String> {
        let start_pos = content
            .find(&self.tag_start)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Start tag not found"))?;

        let end_pos = content
            .find(&self.tag_end)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "End tag not found"))?;

        if end_pos < start_pos {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "End tag appears before start tag",
            ));
        }

        let before = &content[..start_pos];
        let after = &content[end_pos + self.tag_end.len()..];

        Ok(format!(
            "{}{}\n{}\n{}{}",
            before, self.tag_start, self.content, self.tag_end, after
        ))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_inject_nonexistent_file_fails() {
        let tag_start = "<!-- tree2md:start -->";
        let tag_end = "<!-- tree2md:end -->";
        let injector = ReadmeInjector::new(
            "- file1.rs".to_string(),
            tag_start.to_string(),
            tag_end.to_string(),
        );

        let result = injector.inject(Path::new("/nonexistent/file.md"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_inject_without_tags_fails() {
        let tag_start = "<!-- tree2md:start -->";
        let tag_end = "<!-- tree2md:end -->";
        let injector = ReadmeInjector::new(
            "- file1.rs\n- file2.rs".to_string(),
            tag_start.to_string(),
            tag_end.to_string(),
        );
        let mut tmp = NamedTempFile::new().unwrap();

        write!(tmp, "# My Project\n\nDescription").unwrap();

        // Should fail because tags are not present
        let result = injector.inject(tmp.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Injection tags not found"));
    }

    #[test]
    fn test_inject_with_tags_succeeds() {
        let tag_start = "<!-- tree2md:start -->";
        let tag_end = "<!-- tree2md:end -->";
        let injector = ReadmeInjector::new(
            "- file1.rs\n- file2.rs".to_string(),
            tag_start.to_string(),
            tag_end.to_string(),
        );
        let mut tmp = NamedTempFile::new().unwrap();

        write!(
            tmp,
            "# My Project\n\n{}\n{}\n\nDescription",
            tag_start, tag_end
        )
        .unwrap();

        injector.inject(tmp.path()).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        assert!(result.contains("# My Project"));
        assert!(result.contains(tag_start));
        assert!(result.contains("- file1.rs"));
        assert!(result.contains(tag_end));
    }

    #[test]
    fn test_update_existing() {
        let tag_start = "<!-- tree2md:start -->";
        let tag_end = "<!-- tree2md:end -->";
        let injector = ReadmeInjector::new(
            "- new1.rs\n- new2.rs".to_string(),
            tag_start.to_string(),
            tag_end.to_string(),
        );
        let mut tmp = NamedTempFile::new().unwrap();

        write!(
            tmp,
            "# Project\n\n{}\nold content\n{}\n\nMore content",
            tag_start, tag_end
        )
        .unwrap();

        injector.inject(tmp.path()).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        assert!(result.contains("# Project"));
        assert!(result.contains("- new1.rs"));
        assert!(!result.contains("old content"));
        assert!(result.contains("More content"));
    }

    #[test]
    fn test_idempotent() {
        let tag_start = "<!-- tree2md:start -->";
        let tag_end = "<!-- tree2md:end -->";
        let injector = ReadmeInjector::new(
            "- file.rs".to_string(),
            tag_start.to_string(),
            tag_end.to_string(),
        );
        let mut tmp = NamedTempFile::new().unwrap();

        // Create file with tags already present
        write!(tmp, "# Project\n\n{}\n{}\n", tag_start, tag_end).unwrap();

        // First injection
        injector.inject(tmp.path()).unwrap();
        let result1 = fs::read_to_string(tmp.path()).unwrap();
        assert!(result1.contains("- file.rs"));

        // Second injection (should update, not duplicate)
        let injector2 = ReadmeInjector::new(
            "- updated.rs".to_string(),
            tag_start.to_string(),
            tag_end.to_string(),
        );
        injector2.inject(tmp.path()).unwrap();
        let result2 = fs::read_to_string(tmp.path()).unwrap();

        // Check only one set of tags
        assert_eq!(result2.matches(tag_start).count(), 1);
        assert_eq!(result2.matches(tag_end).count(), 1);

        // Check content was updated
        assert!(result2.contains("- updated.rs"));
        assert!(!result2.contains("- file.rs"));
    }
}
