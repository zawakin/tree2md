use crate::cli::Args;
use crate::content::io::is_binary_extension;
use crate::fs_tree::{LocCounter, Node};
use crate::language::detect_lang;
use crate::output::stats::Stats;
use crate::profile::EmojiMapper;
use crate::render::pipeline::{build_ir, AggregationContext, IrDir, IrFile};
use crate::render::renderer::{OutputFormat, Renderer};

/// Pipe renderer for non-TTY output.
/// Produces plain tree characters with optional line counts and file contents.
pub struct PipeRenderer<'a> {
    args: &'a Args,
    emoji_mapper: EmojiMapper,
    stats: Stats,
    loc_counter: LocCounter,
    output: String,
}

impl<'a> PipeRenderer<'a> {
    pub fn new(args: &'a Args) -> Self {
        Self {
            args,
            emoji_mapper: EmojiMapper::new(false), // no emoji in pipe mode
            stats: Stats::new(),
            loc_counter: LocCounter::new(args.loc.clone()),
            output: String::new(),
        }
    }

    fn render_ir_dir(&mut self, dir: &IrDir, prefix: &str) {
        let total = dir.dirs.len() + dir.files.len();
        let mut idx = 0;

        // Render subdirectories first
        for subdir in &dir.dirs {
            idx += 1;
            let is_last = idx == total;
            let branch = if is_last { "└── " } else { "├── " };
            let continuation = if is_last { "    " } else { "│   " };

            self.output
                .push_str(&format!("{}{}{}/\n", prefix, branch, subdir.name));

            let new_prefix = format!("{}{}", prefix, continuation);
            self.render_ir_dir(subdir, &new_prefix);
        }

        // Then render files
        for file in &dir.files {
            idx += 1;
            let is_last = idx == total;
            let branch = if is_last { "└── " } else { "├── " };

            self.output.push_str(prefix);
            self.output.push_str(branch);
            self.output.push_str(&file.name);

            if let Some(loc) = file.loc {
                self.output.push_str(&format!("  ({} lines)", loc));
            }

            self.output.push('\n');
        }
    }

    fn render_contents(&mut self, dir: &IrDir) {
        // Render file contents in DFS order
        for subdir in &dir.dirs {
            self.render_contents(subdir);
        }

        for file in &dir.files {
            self.render_file_content(file);
        }
    }

    fn render_file_content(&mut self, file: &IrFile) {
        // Skip binary files by extension
        if is_binary_extension(&file.path) {
            return;
        }

        let file_name = file
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let lang_hint = detect_lang(&file_name).map(|l| l.name).unwrap_or("");

        // Read from the actual filesystem path, display using display_path
        match std::fs::read_to_string(&file.path) {
            Ok(content) => {
                self.output.push_str(&format!(
                    "\n## {}\n\n```{}\n",
                    file.display_path.display(),
                    lang_hint
                ));
                self.output.push_str(&content);
                if !content.ends_with('\n') {
                    self.output.push('\n');
                }
                self.output.push_str("```\n");
            }
            Err(_) => {
                // Skip files that can't be read
            }
        }
    }
}

impl<'a> Renderer for PipeRenderer<'a> {
    fn render_tree(&mut self, root: &Node) -> String {
        self.output.clear();
        self.stats.reset();

        if !root.children.is_empty() {
            self.stats.add_directory();
        }

        let mut ctx = AggregationContext {
            emoji_mapper: &self.emoji_mapper,
            stats: &mut self.stats,
            loc_counter: &self.loc_counter,
        };

        let ir = build_ir(root, &mut ctx);

        // Render tree structure
        self.output.push_str(".\n");
        self.render_ir_dir(&ir, "");

        // Append stats if enabled
        if self.args.should_show_stats() {
            self.output.push('\n');
            self.output.push_str(&self.render_stats(&self.stats));
        }

        // Append file contents if -c is enabled
        if self.args.contents {
            self.render_contents(&ir);
        }

        self.output.clone()
    }

    fn render_stats(&self, stats: &Stats) -> String {
        stats.generate_output(self.args.stats.clone(), false)
    }

    fn output_format(&self) -> OutputFormat {
        OutputFormat::Pipe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{FunMode, LocMode, StatsMode};
    use crate::fs_tree::Node;
    use std::path::PathBuf;

    fn create_test_args() -> Args {
        Args {
            target: ".".to_string(),
            level: None,
            include: vec![],
            exclude: vec![],
            use_gitignore: crate::cli::UseGitignoreMode::Auto,
            emoji: vec![],
            emoji_map: None,
            fun: FunMode::Off,
            no_anim: false,
            stats: StatsMode::Off,
            loc: LocMode::Off,
            contents: false,
            safe: true,
            unsafe_mode: false,
        }
    }

    #[test]
    fn test_pipe_renderer_basic_tree() {
        let args = create_test_args();
        let mut renderer = PipeRenderer::new(&args);

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
                    name: "Cargo.toml".to_string(),
                    path: PathBuf::from("test/Cargo.toml"),
                    is_dir: false,
                    display_path: PathBuf::from("Cargo.toml"),
                    children: vec![],
                },
            ],
        };

        let output = renderer.render_tree(&root);
        assert!(output.starts_with(".\n"));
        assert!(output.contains("src/"));
        assert!(output.contains("main.rs"));
        assert!(output.contains("Cargo.toml"));
        assert!(output.contains("├── ") || output.contains("└── "));
    }

    #[test]
    fn test_pipe_renderer_output_format() {
        let args = create_test_args();
        let renderer = PipeRenderer::new(&args);
        assert_eq!(renderer.output_format(), OutputFormat::Pipe);
    }
}
