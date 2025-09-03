use crate::cli::{Args, LinksMode};
use std::path::Path;

/// Generator for Markdown links with optional GitHub URL rewriting
pub struct LinkGenerator<'a> {
    args: &'a Args,
    github_base: Option<String>,
}

impl<'a> LinkGenerator<'a> {
    /// Create a new link generator
    pub fn new(args: &'a Args) -> Self {
        // Process GitHub URL if provided
        let github_base = args.github.as_ref().map(|url| {
            // Ensure URL ends without trailing slash for clean joining
            url.trim_end_matches('/').to_string()
        });

        Self { args, github_base }
    }

    /// Generate a link for a path
    pub fn generate_link(&self, path: &Path, is_dir: bool) -> String {
        // Check if links are disabled
        if matches!(self.args.links, LinksMode::Off) {
            return self.format_plain_text(path, is_dir);
        }

        // Generate the link URL
        let url = self.generate_url(path);

        // Format as Markdown link
        self.format_markdown_link(path, &url, is_dir)
    }

    /// Generate the URL for a path
    fn generate_url(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy().replace('\\', "/");

        if let Some(ref github_base) = self.github_base {
            // Rewrite to GitHub browse URL
            format!("{}/{}", github_base, path_str)
        } else {
            // Use relative path
            path_str
        }
    }

    /// Format as a Markdown link
    fn format_markdown_link(&self, path: &Path, url: &str, is_dir: bool) -> String {
        let display_name = if is_dir {
            format!("<code>{}/</code>", self.get_display_name(path))
        } else {
            format!("<a href=\"{}\">{}</a>", url, self.get_display_name(path))
        };

        display_name
    }

    /// Format as plain text (when links are off)
    fn format_plain_text(&self, path: &Path, is_dir: bool) -> String {
        if is_dir {
            format!("<code>{}/</code>", self.get_display_name(path))
        } else {
            self.get_display_name(path)
        }
    }

    /// Get the display name for a path
    fn get_display_name(&self, path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| path.to_str().unwrap_or(""))
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use std::path::PathBuf;

    #[test]
    fn test_link_generator_relative() {
        use clap::Parser;
        let args = Args::parse_from(vec!["tree2md", "."]);
        let generator = LinkGenerator::new(&args);

        let path = PathBuf::from("src/main.rs");
        let link = generator.generate_link(&path, false);

        assert!(link.contains("href=\"src/main.rs\""));
        assert!(link.contains("main.rs"));
    }

    #[test]
    fn test_link_generator_github() {
        use clap::Parser;
        let args = Args::parse_from(vec![
            "tree2md",
            ".",
            "--github",
            "https://github.com/user/repo/tree/main",
        ]);
        let generator = LinkGenerator::new(&args);

        let path = PathBuf::from("src/main.rs");
        let link = generator.generate_link(&path, false);

        assert!(link.contains("href=\"https://github.com/user/repo/tree/main/src/main.rs\""));
    }

    #[test]
    fn test_link_generator_directory() {
        use clap::Parser;
        let args = Args::parse_from(vec!["tree2md", "."]);
        let generator = LinkGenerator::new(&args);

        let path = PathBuf::from("src");
        let link = generator.generate_link(&path, true);

        assert!(link.contains("<code>"));
        assert!(link.contains("src/"));
    }

    #[test]
    fn test_link_generator_disabled() {
        use clap::Parser;
        let args = Args::parse_from(vec!["tree2md", ".", "--links", "off"]);
        let generator = LinkGenerator::new(&args);

        let path = PathBuf::from("src/main.rs");
        let link = generator.generate_link(&path, false);

        // Should not contain href when links are off
        assert!(!link.contains("href"));
        assert_eq!(link, "main.rs");
    }
}
