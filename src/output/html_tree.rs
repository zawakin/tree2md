use crate::cli::{Args, FoldMode};
use crate::fs_tree::{LocCounter, Node};
use crate::output::links::LinkGenerator;
use crate::output::stats::Stats;
use crate::profile::{EmojiMapper, FileType};
use crate::render::pipeline::{build_ir, AggregationContext, IrDir, IrFile};
use crate::terminal::detect::TerminalDetector;
use std::fmt::Write;
use std::path::Path;

/// Formatter for HTML-style tree output with collapsible folders
pub struct HtmlTreeFormatter<'a> {
    args: &'a Args,
    link_generator: LinkGenerator<'a>,
    emoji_mapper: EmojiMapper,
    stats: Stats,
    loc_counter: LocCounter,
    output: String,
}

impl<'a> HtmlTreeFormatter<'a> {
    /// Create a new HTML tree formatter
    pub fn new(args: &'a Args) -> Self {
        let detector = TerminalDetector::new();
        let use_emoji = args.is_fun_enabled(detector.is_tty());
        let mut emoji_mapper = EmojiMapper::new(use_emoji);

        // Load custom emoji mappings from file if provided
        if let Some(emoji_map_path) = &args.emoji_map {
            if let Err(e) = emoji_mapper.load_from_file(Path::new(emoji_map_path)) {
                eprintln!(
                    "Warning: Failed to load emoji map from {}: {}",
                    emoji_map_path, e
                );
            }
        }

        // Apply CLI emoji overrides
        for emoji_arg in &args.emoji {
            emoji_mapper.parse_cli_emoji(emoji_arg);
        }

        Self {
            args,
            link_generator: LinkGenerator::new(args),
            emoji_mapper,
            stats: Stats::new(),
            loc_counter: LocCounter::new(args.loc.clone()),
            output: String::new(),
        }
    }

    /// Format the tree and return the complete output
    pub fn format_tree(&mut self, root: &Node) -> String {
        self.output.clear();
        self.stats.reset();

        // Count root directory in stats if it has children
        if !root.children.is_empty() {
            self.stats.add_directory();
        }

        // Build IR with aggregation
        let mut ctx = AggregationContext {
            emoji_mapper: &self.emoji_mapper,
            stats: &mut self.stats,
            loc_counter: &self.loc_counter,
        };

        let ir = build_ir(root, &mut ctx);

        // Start with unordered list
        self.output.push_str("<ul>\n");

        // Format IR
        self.format_ir_dir(&ir, 1);

        self.output.push_str("</ul>\n");

        // Add stats footer if not suppressed
        if !self.args.no_stats {
            self.output.push_str("\n---\n\n");
            self.output.push_str(&self.stats.generate_footer());
        }

        self.output.clone()
    }

    /// Format IR directory and its contents
    fn format_ir_dir(&mut self, dir: &IrDir, depth: usize) {
        let indent = "  ".repeat(depth);

        // Format subdirectories first
        for subdir in &dir.dirs {
            self.format_ir_directory(subdir, &indent, depth);
        }

        // Then format files
        for file in &dir.files {
            self.format_ir_file(file, &indent);
        }
    }

    /// Format a directory node with optional collapsible details
    fn format_ir_directory(&mut self, dir: &IrDir, indent: &str, depth: usize) {
        // Count children for the directory stats
        let (file_count, dir_count) = (dir.files.len(), dir.dirs.len());

        // Determine if we should use details tag
        let use_details = match self.args.fold {
            FoldMode::On => true,
            FoldMode::Off => false,
            FoldMode::Auto => {
                // In auto mode, fold all directories (per spec OUTPUT_SAMPLE.md)
                // This provides better navigation for any project structure
                true
            }
        };

        self.output.push_str(indent);
        self.output.push_str("<li>\n");

        if use_details && (!dir.files.is_empty() || !dir.dirs.is_empty()) {
            // Use collapsible details
            let open = if depth <= 1 { " open" } else { "" };
            writeln!(
                &mut self.output,
                "{indent}  <details{open}>",
                indent = indent,
                open = open
            )
            .unwrap();

            // Get emoji for directory
            let dir_emoji = self
                .emoji_mapper
                .get_emoji(&dir.display_path, FileType::Directory);
            let emoji_str = if !dir_emoji.is_empty() {
                format!("{} ", dir_emoji)
            } else {
                String::new()
            };

            write!(
                &mut self.output,
                "{indent}    <summary>{emoji}<code>{name}/</code>",
                indent = indent,
                emoji = emoji_str,
                name = dir.name
            )
            .unwrap();

            // Add counts in summary
            writeln!(
                &mut self.output,
                " (files: {}, dirs: {})</summary>",
                file_count, dir_count
            )
            .unwrap();

            // Children in nested list
            if !dir.files.is_empty() || !dir.dirs.is_empty() {
                self.output.push_str(&format!("{}    <ul>\n", indent));
                self.format_ir_dir(dir, depth + 2);
                self.output.push_str(&format!("{}    </ul>\n", indent));
            }

            self.output.push_str(&format!("{}  </details>\n", indent));
        } else {
            // Simple directory without details
            // Get emoji for directory
            let dir_emoji = self
                .emoji_mapper
                .get_emoji(&dir.display_path, FileType::Directory);
            let emoji_str = if !dir_emoji.is_empty() {
                format!("{} ", dir_emoji)
            } else {
                String::new()
            };

            let dir_link = self.link_generator.generate_link(&dir.display_path, true);
            write!(
                &mut self.output,
                "{indent}  {emoji}{link}",
                indent = indent,
                emoji = emoji_str,
                link = dir_link
            )
            .unwrap();

            if !dir.files.is_empty() || !dir.dirs.is_empty() {
                self.output.push('\n');
                self.output.push_str(&format!("{}  <ul>\n", indent));
                self.format_ir_dir(dir, depth + 1);
                self.output.push_str(&format!("{}  </ul>\n", indent));
            }
        }

        self.output.push_str(&format!("{}</li>\n", indent));
    }

    /// Format a file node with a link
    fn format_ir_file(&mut self, file: &IrFile, indent: &str) {
        let emoji_str = if !file.emoji.is_empty() {
            format!("{} ", file.emoji)
        } else {
            String::new()
        };

        let file_link = self.link_generator.generate_link(&file.display_path, false);
        writeln!(
            &mut self.output,
            "{indent}<li>{emoji}{link}</li>",
            indent = indent,
            emoji = emoji_str,
            link = file_link
        )
        .unwrap();
    }

    // Removed old stats helper methods - no longer needed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{FunMode, LinksMode, LocMode, OutputMode, StatsMode};
    use std::path::PathBuf;

    fn create_test_args() -> Args {
        Args {
            target: ".".to_string(),
            level: None,
            include: vec![],
            exclude: vec![],
            use_gitignore: crate::cli::UseGitignoreMode::Auto,
            links: LinksMode::Off,
            github: None,
            emoji: vec![],
            emoji_map: None,
            output: OutputMode::Html,
            preset: None,
            fun: FunMode::Off,
            no_anim: false,
            fold: FoldMode::Off,
            stats: StatsMode::Off,
            no_stats: false,
            loc: LocMode::Off,
            contents: false,
            safe: true,
            unsafe_mode: false,
            restrict_root: None,
            stamp: vec![],
            stamp_date_format: "%Y-%m-%d".to_string(),
            inject: None,
            tag_start: "<!-- tree2md:start -->".to_string(),
            tag_end: "<!-- tree2md:end -->".to_string(),
            dry_run: false,
        }
    }

    #[test]
    fn test_html_tree_formatter_basic() {
        let args = create_test_args();
        let mut formatter = HtmlTreeFormatter::new(&args);

        let root = Node {
            name: "test".to_string(),
            path: PathBuf::from("test"),
            is_dir: true,
            display_path: PathBuf::from("."),
            children: vec![
                Node {
                    name: "file1.txt".to_string(),
                    path: PathBuf::from("test/file1.txt"),
                    is_dir: false,
                    children: vec![],
                    display_path: PathBuf::from("file1.txt"),
                },
                Node {
                    name: "dir1".to_string(),
                    path: PathBuf::from("test/dir1"),
                    is_dir: true,
                    display_path: PathBuf::from("dir1"),
                    children: vec![Node {
                        name: "file2.txt".to_string(),
                        path: PathBuf::from("test/dir1/file2.txt"),
                        is_dir: false,
                        children: vec![],
                        display_path: PathBuf::from("dir1/file2.txt"),
                    }],
                },
            ],
        };

        let output = formatter.format_tree(&root);

        assert!(output.contains("<ul>"), "Should have opening ul tag");
        assert!(output.contains("</ul>"), "Should have closing ul tag");
        assert!(output.contains("file1.txt"), "Should contain file1");
        assert!(output.contains("dir1"), "Should contain directory");
    }

    #[test]
    fn test_html_tree_formatter_with_fold() {
        let mut args = create_test_args();
        args.fold = FoldMode::On;
        let mut formatter = HtmlTreeFormatter::new(&args);

        let root = Node {
            name: "test".to_string(),
            path: PathBuf::from("test"),
            is_dir: true,
            display_path: PathBuf::from("."),
            children: vec![Node {
                name: "dir1".to_string(),
                path: PathBuf::from("test/dir1"),
                is_dir: true,
                display_path: PathBuf::from("dir1"),
                children: vec![Node {
                    name: "file1.txt".to_string(),
                    path: PathBuf::from("test/dir1/file1.txt"),
                    is_dir: false,
                    children: vec![],
                    display_path: PathBuf::from("dir1/file1.txt"),
                }],
            }],
        };

        let output = formatter.format_tree(&root);

        assert!(
            output.contains("<details"),
            "Should use details tag for folding"
        );
        assert!(output.contains("<summary>"), "Should have summary tag");
        assert!(
            output.contains("(files: 1, dirs: 0)"),
            "Should show file/dir counts"
        );
    }

    #[test]
    fn test_html_tree_formatter_with_links() {
        let mut args = create_test_args();
        args.links = LinksMode::On;
        args.github = Some("https://github.com/user/repo".to_string());
        let mut formatter = HtmlTreeFormatter::new(&args);

        let root = Node {
            name: "test".to_string(),
            path: PathBuf::from("test"),
            is_dir: true,
            display_path: PathBuf::from("."),
            children: vec![Node {
                name: "file.txt".to_string(),
                path: PathBuf::from("test/file.txt"),
                is_dir: false,
                children: vec![],
                display_path: PathBuf::from("file.txt"),
            }],
        };

        let output = formatter.format_tree(&root);

        assert!(
            output.contains("href=\"https://github.com/user/repo/file.txt\""),
            "Should generate GitHub link for file"
        );
    }
}
