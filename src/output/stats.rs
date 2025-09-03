use crate::cli::StatsMode;
use crate::profile::FileType;
use crate::terminal::capabilities::ProgressChars;
use std::collections::HashMap;
use std::path::Path;

// Type alias for backwards compatibility
#[allow(dead_code)]
pub type StatsCollector = Stats;

/// Unified statistics collector combining all stats functionality
pub struct Stats {
    file_types: HashMap<FileType, TypeStats>,
    extension_counts: HashMap<String, usize>,
    total_dirs: usize,
    total_files: usize,
    total_loc: Option<usize>,
}

#[derive(Default)]
struct TypeStats {
    count: usize,
    emoji: String,
    name: String,
    loc: Option<usize>,
}

impl Stats {
    /// Create a new stats collector
    pub fn new() -> Self {
        Self {
            file_types: HashMap::new(),
            extension_counts: HashMap::new(),
            total_dirs: 0,
            total_files: 0,
            total_loc: None,
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.file_types.clear();
        self.extension_counts.clear();
        self.total_dirs = 0;
        self.total_files = 0;
        self.total_loc = None;
    }

    /// Add a file with its type
    pub fn add_file(&mut self, file_type: FileType, emoji: String, path: &Path) {
        self.total_files += 1;

        // Track by file type
        let entry = self
            .file_types
            .entry(file_type)
            .or_insert_with(|| TypeStats {
                count: 0,
                emoji: emoji.clone(),
                name: file_type.display_name().to_string(),
                loc: None,
            });
        entry.count += 1;

        // Track by extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            *self.extension_counts.entry(ext_str).or_insert(0) += 1;
        } else {
            *self
                .extension_counts
                .entry(String::from("(no ext)"))
                .or_insert(0) += 1;
        }
    }

    /// Add a directory
    pub fn add_directory(&mut self) {
        self.total_dirs += 1;
    }

    /// Get the total number of directories
    #[allow(dead_code)]
    pub fn total_dirs(&self) -> usize {
        self.total_dirs
    }

    /// Add LOC count for a file type
    pub fn add_loc(&mut self, file_type: FileType, lines: usize) {
        if let Some(stats) = self.file_types.get_mut(&file_type) {
            stats.loc = Some(stats.loc.unwrap_or(0) + lines);
        }
        self.total_loc = Some(self.total_loc.unwrap_or(0) + lines);
    }

    /// Generate stats output based on mode
    pub fn generate_output(&self, mode: StatsMode, use_unicode: bool) -> String {
        match mode {
            StatsMode::Off => self.generate_footer(),
            StatsMode::Min => self.generate_minimal(),
            StatsMode::Full => self.generate_full(use_unicode),
        }
    }

    /// Generate the basic stats footer (similar to old StatsCollector)
    pub fn generate_footer(&self) -> String {
        let mut footer = String::new();

        footer.push_str("**Stats**\n");
        footer.push_str(&format!("- Dirs: {}\n", self.total_dirs));
        footer.push_str(&format!("- Files: {}\n", self.total_files));

        // Get top extensions by count
        if !self.extension_counts.is_empty() {
            let mut ext_vec: Vec<(&String, &usize)> = self.extension_counts.iter().collect();
            ext_vec.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

            // Show top 5 extensions
            let top_exts: Vec<String> = ext_vec
                .iter()
                .take(5)
                .map(|(ext, count)| {
                    if ext == &"(no ext)" {
                        format!("no-ext({})", count)
                    } else {
                        format!("{}({})", ext, count)
                    }
                })
                .collect();

            if !top_exts.is_empty() {
                footer.push_str(&format!("- Top by ext: {}\n", top_exts.join(", ")));
            }
        }

        footer
    }

    /// Generate minimal stats
    fn generate_minimal(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "**Stats**: 📂 {} dirs • 📄 {} files",
            self.total_dirs, self.total_files
        ));

        if let Some(loc) = self.total_loc {
            output.push_str(&format!(" • 🧾 ~{} LOC", format_count(loc)));
        }

        output.push('\n');
        output
    }

    /// Generate full stats with progress bars
    fn generate_full(&self, use_unicode: bool) -> String {
        let mut output = String::new();

        // Totals line
        output.push_str(&format!(
            "**Totals**: 📂 {} dirs • 📄 {} files",
            self.total_dirs, self.total_files
        ));

        if let Some(loc) = self.total_loc {
            output.push_str(&format!(" • 🧾 ~{} LOC", format_count(loc)));
        }

        output.push('\n');

        // File type breakdown with progress bars
        if !self.file_types.is_empty() {
            output.push_str("\n**By type**:\n");

            // Sort by count descending
            let mut types: Vec<_> = self.file_types.iter().collect();
            types.sort_by(|a, b| b.1.count.cmp(&a.1.count));

            let chars = if use_unicode {
                ProgressChars::unicode()
            } else {
                ProgressChars::ascii()
            };

            for (_file_type, stats) in types.iter().take(8) {
                let percentage = (stats.count as f32 / self.total_files as f32) * 100.0;
                let bar = self.render_bar(percentage, 15, chars.clone());

                let emoji = if !stats.emoji.is_empty() {
                    format!("{} ", stats.emoji)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "- {}{}: {} ({:.0}%) {}\n",
                    emoji, stats.name, stats.count, percentage, bar
                ));
            }
        }

        output
    }

    /// Render a progress bar
    fn render_bar(&self, percentage: f32, width: usize, chars: ProgressChars) -> String {
        let filled = ((percentage * width as f32 / 100.0).round() as usize).min(width);
        let empty = width - filled;

        format!(
            "{}{}",
            chars.filled_md.repeat(filled),
            chars.empty_md.repeat(empty)
        )
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

/// Format large numbers with K/M suffixes
fn format_count(count: usize) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_stats_collector() {
        let mut stats = Stats::new();

        // Add directories
        stats.add_directory();
        stats.add_directory();
        assert_eq!(stats.total_dirs, 2);

        // Add files
        stats.add_file(
            FileType::Rust,
            String::from("🦀"),
            &PathBuf::from("main.rs"),
        );
        stats.add_file(
            FileType::Python,
            String::from("🐍"),
            &PathBuf::from("app.py"),
        );
        stats.add_file(FileType::Text, String::new(), &PathBuf::from("README"));
        assert_eq!(stats.total_files, 3);

        // Check extensions
        assert_eq!(stats.extension_counts.get("rs"), Some(&1));
        assert_eq!(stats.extension_counts.get("py"), Some(&1));
        assert_eq!(stats.extension_counts.get("(no ext)"), Some(&1));

        // Generate footer
        let footer = stats.generate_footer();
        assert!(footer.contains("Dirs: 2"));
        assert!(footer.contains("Files: 3"));
    }

    #[test]
    fn test_stats_reset() {
        let mut stats = Stats::new();
        stats.add_directory();
        stats.add_file(FileType::Rust, String::new(), &PathBuf::from("test.rs"));

        stats.reset();
        assert_eq!(stats.total_dirs, 0);
        assert_eq!(stats.total_files, 0);
        assert!(stats.file_types.is_empty());
        assert!(stats.extension_counts.is_empty());
    }

    #[test]
    fn test_loc_tracking() {
        let mut stats = Stats::new();
        stats.add_file(FileType::Rust, String::new(), &PathBuf::from("main.rs"));
        stats.add_loc(FileType::Rust, 100);
        stats.add_loc(FileType::Rust, 50);

        assert_eq!(stats.total_loc, Some(150));
        assert_eq!(
            stats.file_types.get(&FileType::Rust).unwrap().loc,
            Some(150)
        );
    }
}
