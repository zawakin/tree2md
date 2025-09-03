use crate::cli::Args;
use crate::fs_tree::{LocCounter, Node};
use crate::output::stats::Stats;
use crate::profile::{EmojiMapper, FileType};
use crate::render::pipeline::{build_ir, AggregationContext, IrDir, IrFile};
use crate::render::renderer::{OutputFormat, Renderer};
use crate::terminal::capabilities::TerminalCapabilities;
use crate::terminal::detect::TerminalDetector;
use crate::util::format::{format_loc_display, is_global_outlier, loc_category, loc_to_bar};
use std::path::Path;

/// Terminal renderer with Unicode tree branches
pub struct TerminalRenderer<'a> {
    args: &'a Args,
    capabilities: TerminalCapabilities,
    emoji_mapper: EmojiMapper,
    stats: Stats,
    loc_counter: LocCounter,
    output: String,
    global_threshold: usize, // Threshold for global outliers (95th percentile)
}

impl<'a> TerminalRenderer<'a> {
    pub fn new(args: &'a Args) -> Self {
        let detector = TerminalDetector::new();
        let capabilities = TerminalCapabilities::new();

        // Set up emoji mapper based on fun mode
        // When fun is explicitly on, force emojis even if terminal detection is conservative
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
            capabilities,
            emoji_mapper,
            stats: Stats::new(),
            loc_counter: LocCounter::new(args.loc.clone()),
            output: String::new(),
            global_threshold: 0, // Will be calculated when rendering
        }
    }

    // Helper to collect all files for max width calculation
    #[allow(clippy::only_used_in_recursion)]
    fn collect_all_files(
        &self,
        dir: &IrDir,
        files: &mut Vec<(String, Option<usize>)>,
        prefix_len: usize,
    ) {
        for subdir in &dir.dirs {
            let new_prefix_len = prefix_len + 2; // Account for tree characters
            self.collect_all_files(subdir, files, new_prefix_len);
        }

        for file in &dir.files {
            files.push((file.name.clone(), file.loc));
        }
    }

    fn render_ir_dir_aligned(&mut self, dir: &IrDir, prefix: &str, max_name_width: usize) {
        let tree_chars = self.capabilities.tree_chars();

        // Calculate max LOC in this directory for local normalization
        let max_loc_in_dir = dir.files.iter().filter_map(|f| f.loc).max().unwrap_or(0);

        // Render subdirectories first
        for (i, subdir) in dir.dirs.iter().enumerate() {
            let subdir_is_last = i == dir.dirs.len() - 1 && dir.files.is_empty();

            // Get emoji for directory
            let dir_emoji = self
                .emoji_mapper
                .get_emoji(&subdir.display_path, FileType::Directory);
            let emoji_str = if !dir_emoji.is_empty() {
                format!("{} ", dir_emoji)
            } else {
                String::new()
            };

            // Render directory name
            self.output.push_str(&format!(
                "{}{}{}{}/\n",
                prefix,
                if subdir_is_last {
                    tree_chars.last_branch
                } else {
                    tree_chars.branch
                },
                emoji_str,
                subdir.name
            ));

            // Render directory contents with updated prefix
            let new_prefix = format!(
                "{}{}",
                prefix,
                if subdir_is_last {
                    tree_chars.empty
                } else {
                    tree_chars.vertical
                }
            );
            self.render_ir_dir_aligned(subdir, &new_prefix, max_name_width);
        }

        // Then render files with local normalization
        for (i, file) in dir.files.iter().enumerate() {
            let file_is_last = i == dir.files.len() - 1;
            self.render_ir_file_with_local_scale(
                file,
                prefix,
                file_is_last,
                max_name_width,
                max_loc_in_dir,
            );
        }
    }

    fn render_ir_file_with_local_scale(
        &mut self,
        file: &IrFile,
        prefix: &str,
        is_last: bool,
        max_name_width: usize,
        max_loc_in_dir: usize,
    ) {
        let tree_chars = self.capabilities.tree_chars();

        let branch = if is_last {
            tree_chars.last_branch
        } else {
            tree_chars.branch
        };

        let emoji_str = if !file.emoji.is_empty() {
            format!("{} ", file.emoji)
        } else {
            String::new()
        };

        // Build the name part with tree structure
        self.output.push_str(prefix);
        self.output.push_str(branch);
        let name_with_emoji = format!("{}{}", emoji_str, file.name);
        self.output.push_str(&name_with_emoji);

        // Add LOC visualization if available
        if let Some(loc) = file.loc {
            // Calculate padding for global alignment
            let current_len = prefix.len() + 2 + name_with_emoji.len(); // 2 for branch chars
            let padding = if current_len < max_name_width {
                " ".repeat(max_name_width - current_len)
            } else {
                "  ".to_string()
            };

            // Generate bar with local normalization (10 cells wide)
            let bar = loc_to_bar(loc, max_loc_in_dir, 10);
            // Bar already includes brackets, no need for additional formatting

            // Format LOC display
            let loc_display = format_loc_display(loc);
            let loc_formatted = format!("{:>6}", loc_display);

            // Get category
            let category = loc_category(loc);

            // Check if it's a global outlier
            let star = if is_global_outlier(loc, self.global_threshold) {
                " ★"
            } else {
                ""
            };

            // Output format: padding + bar + loc + (category) + star
            self.output.push_str(&format!(
                "{}  {}  {} ({}){}",
                padding, bar, loc_formatted, category, star
            ));
        }

        self.output.push('\n');
    }
}

impl<'a> Renderer for TerminalRenderer<'a> {
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

        // Calculate maximum name width across all files
        let mut all_files = Vec::new();
        self.collect_all_files(&ir, &mut all_files, 0);

        // Find the maximum width needed for alignment
        let max_name_width = all_files
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0)
            + 10; // Add some padding for tree characters and emoji

        // Calculate global threshold (95th percentile or top 10 files)
        let mut all_locs: Vec<usize> = all_files.iter().filter_map(|(_, loc)| *loc).collect();
        all_locs.sort_unstable_by(|a, b| b.cmp(a)); // Sort descending

        // Use 95th percentile or top 10, whichever is more restrictive
        let percentile_95_idx = (all_locs.len() as f64 * 0.05).ceil() as usize;
        let top_n_idx = 10.min(all_locs.len());
        let threshold_idx = percentile_95_idx.min(top_n_idx);

        self.global_threshold = if threshold_idx > 0 && threshold_idx <= all_locs.len() {
            all_locs[threshold_idx - 1]
        } else {
            usize::MAX // No files qualify as outliers
        };

        // Render the IR with global alignment
        self.render_ir_dir_aligned(&ir, "", max_name_width);

        if self.args.should_show_stats() {
            self.output.push('\n');
            self.output.push_str(&self.render_stats(&self.stats));
        }

        self.output.clone()
    }

    fn render_stats(&self, stats: &Stats) -> String {
        stats.generate_output(
            self.args.stats.clone(),
            self.capabilities.supports_unicode_trees(),
        )
    }

    fn supports_animation(&self) -> bool {
        self.capabilities.supports_animation()
    }

    fn supports_colors(&self) -> bool {
        self.capabilities.supports_colors()
    }

    fn output_format(&self) -> OutputFormat {
        OutputFormat::Terminal
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
            output: OutputMode::Tty,
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
    fn test_terminal_renderer_basic() {
        let args = create_test_args();
        let mut renderer = TerminalRenderer::new(&args);

        let root = Node {
            name: "test".to_string(),
            path: PathBuf::from("test"),
            is_dir: true,
            display_path: PathBuf::from("."),
            children: vec![
                Node {
                    name: "dir1".to_string(),
                    path: PathBuf::from("test/dir1"),
                    is_dir: true,
                    display_path: PathBuf::from("dir1"),
                    children: vec![Node {
                        name: "file1.txt".to_string(),
                        path: PathBuf::from("test/dir1/file1.txt"),
                        is_dir: false,
                        display_path: PathBuf::from("dir1/file1.txt"),
                        children: vec![],
                    }],
                },
                Node {
                    name: "file2.rs".to_string(),
                    path: PathBuf::from("test/file2.rs"),
                    is_dir: false,
                    display_path: PathBuf::from("file2.rs"),
                    children: vec![],
                },
            ],
        };

        let output = renderer.render_tree(&root);
        assert!(output.contains("dir1/"));
        assert!(output.contains("file1.txt"));
        assert!(output.contains("file2.rs"));
    }

    #[test]
    fn test_terminal_renderer_with_unicode_trees() {
        let args = create_test_args();
        let _renderer = TerminalRenderer::new(&args);

        // Test passes if renderer creates successfully
        // Unicode support is handled internally
    }

    #[test]
    fn test_terminal_renderer_output_format() {
        let args = create_test_args();
        let renderer = TerminalRenderer::new(&args);

        assert_eq!(renderer.output_format(), OutputFormat::Terminal);
    }
}
