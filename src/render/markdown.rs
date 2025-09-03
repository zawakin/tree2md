use crate::cli::Args;
use crate::fs_tree::{LocCounter, Node};
use crate::output::stats::Stats;
use crate::profile::{EmojiMapper, FileType};
use crate::render::pipeline::{build_ir, AggregationContext, IrDir, IrFile};
use crate::render::renderer::{OutputFormat, Renderer};
use crate::terminal::detect::TerminalDetector;
use std::path::Path;

/// Pure Markdown renderer with bullet lists
pub struct MarkdownRenderer<'a> {
    args: &'a Args,
    emoji_mapper: EmojiMapper,
    stats: Stats,
    loc_counter: LocCounter,
    output: String,
}

impl<'a> MarkdownRenderer<'a> {
    pub fn new(args: &'a Args) -> Self {
        let detector = TerminalDetector::new();

        // For Markdown output, use emojis if fun mode is enabled
        // (Markdown files can display emojis on GitHub and other platforms)
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
            emoji_mapper,
            stats: Stats::new(),
            loc_counter: LocCounter::new(args.loc.clone()),
            output: String::new(),
        }
    }

    fn combine_url(base: &str, path: &str) -> String {
        let base = base.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }

    fn render_ir_dir(&mut self, dir: &IrDir, indent: usize) {
        let indent_str = "  ".repeat(indent);

        // Render subdirectories first
        for subdir in &dir.dirs {
            // Get emoji for directory
            let dir_emoji = self
                .emoji_mapper
                .get_emoji(&subdir.display_path, FileType::Directory);
            let emoji_str = if !dir_emoji.is_empty() {
                format!("{} ", dir_emoji)
            } else {
                String::new()
            };

            self.output
                .push_str(&format!("{}- {}{}/\n", indent_str, emoji_str, subdir.name));

            self.render_ir_dir(subdir, indent + 1);
        }

        // Then render files
        for file in &dir.files {
            self.render_ir_file(file, indent);
        }
    }

    fn render_ir_file(&mut self, file: &IrFile, indent: usize) {
        let indent_str = "  ".repeat(indent);

        let emoji_str = if !file.emoji.is_empty() {
            format!("{} ", file.emoji)
        } else {
            String::new()
        };

        // Add Markdown link if links are enabled
        let filename = if self.args.links == crate::cli::LinksMode::On {
            // Generate Markdown-style link [name](url)
            let path_str = file.display_path.to_string_lossy().replace('\\', "/");
            if let Some(github_url) = &self.args.github {
                format!(
                    "[{}]({})",
                    file.name,
                    Self::combine_url(github_url, &path_str)
                )
            } else {
                // Use relative link when no GitHub URL provided
                format!("[{}]({})", file.name, path_str)
            }
        } else {
            file.name.clone()
        };

        // Add LOC count if available
        let loc_str = if let Some(loc) = file.loc {
            format!(" ({} lines)", loc)
        } else {
            String::new()
        };

        self.output.push_str(&format!(
            "{}- {}{}{}\n",
            indent_str, emoji_str, filename, loc_str
        ));
    }
}

impl<'a> Renderer for MarkdownRenderer<'a> {
    fn render_tree(&mut self, root: &Node) -> String {
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

        // Render the IR
        self.render_ir_dir(&ir, 0);

        if self.args.should_show_stats() {
            self.output.push('\n');
            self.output.push_str(&self.render_stats(&self.stats));
        }

        self.output.clone()
    }

    fn render_stats(&self, stats: &Stats) -> String {
        stats.generate_output(self.args.stats.clone(), false)
    }

    fn output_format(&self) -> OutputFormat {
        OutputFormat::Markdown
    }
}

// Removed old stats helper methods - no longer needed

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
            output: OutputMode::Md,
            preset: None,
            fun: FunMode::Off,
            no_anim: false,
            fold: crate::cli::FoldMode::Off,
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
    fn test_markdown_renderer_basic() {
        let args = create_test_args();
        let mut renderer = MarkdownRenderer::new(&args);

        let root = Node {
            name: "test".to_string(),
            path: PathBuf::from("test"),
            is_dir: true,
            display_path: PathBuf::from("."),
            children: vec![
                Node {
                    name: "src".to_string(),
                    path: PathBuf::from("test/src"),
                    is_dir: true,
                    display_path: PathBuf::from("src"),
                    children: vec![Node {
                        name: "main.rs".to_string(),
                        path: PathBuf::from("test/src/main.rs"),
                        is_dir: false,
                        display_path: PathBuf::from("src/main.rs"),
                        children: vec![],
                    }],
                },
                Node {
                    name: "README.md".to_string(),
                    path: PathBuf::from("test/README.md"),
                    is_dir: false,
                    display_path: PathBuf::from("README.md"),
                    children: vec![],
                },
            ],
        };

        let output = renderer.render_tree(&root);
        assert!(output.contains("- README.md"));
        assert!(output.contains("- src/"));
        assert!(output.contains("  - main.rs"));
    }

    #[test]
    fn test_markdown_renderer_with_links() {
        let mut args = create_test_args();
        args.links = LinksMode::On;
        args.github = Some("https://github.com/user/repo".to_string());
        let mut renderer = MarkdownRenderer::new(&args);

        let root = Node {
            name: "test".to_string(),
            path: PathBuf::from("test"),
            is_dir: true,
            display_path: PathBuf::from("."),
            children: vec![Node {
                name: "file.txt".to_string(),
                path: PathBuf::from("test/file.txt"),
                is_dir: false,
                display_path: PathBuf::from("file.txt"),
                children: vec![],
            }],
        };

        let output = renderer.render_tree(&root);
        assert!(output.contains("[file.txt](https://github.com/user/repo/file.txt)"));
    }

    #[test]
    fn test_markdown_renderer_output_format() {
        let args = create_test_args();
        let renderer = MarkdownRenderer::new(&args);

        assert_eq!(renderer.output_format(), OutputFormat::Markdown);
    }
}
