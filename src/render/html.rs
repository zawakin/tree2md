use crate::cli::Args;
use crate::fs_tree::Node;
use crate::output::html_tree::HtmlTreeFormatter;
use crate::output::stats::Stats;
use crate::render::renderer::{OutputFormat, Renderer};

/// HTML renderer that delegates to HtmlTreeFormatter
pub struct HtmlRenderer<'a> {
    args: &'a Args,
}

impl<'a> HtmlRenderer<'a> {
    pub fn new(args: &'a Args) -> Self {
        Self { args }
    }
}

impl<'a> Renderer for HtmlRenderer<'a> {
    fn render_tree(&mut self, root: &Node) -> String {
        let mut formatter = HtmlTreeFormatter::new(self.args);
        formatter.format_tree(root)
    }

    fn render_stats(&self, stats: &Stats) -> String {
        stats.generate_footer()
    }

    fn output_format(&self) -> OutputFormat {
        OutputFormat::Html
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{FoldMode, FunMode, LinksMode, LocMode, OutputMode, StatsMode};
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
    fn test_html_renderer_basic() {
        let args = create_test_args();
        let mut renderer = HtmlRenderer::new(&args);

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

        let output = renderer.render_tree(&root);

        assert!(output.contains("<ul>"), "Should have opening ul tag");
        assert!(output.contains("</ul>"), "Should have closing ul tag");
        assert!(
            output.contains("<li>file.txt</li>"),
            "Should render file as list item"
        );
    }

    #[test]
    fn test_html_renderer_with_details() {
        let mut args = create_test_args();
        args.fold = FoldMode::On;

        let mut renderer = HtmlRenderer::new(&args);

        let root = Node {
            name: "root".to_string(),
            path: PathBuf::from("."),
            is_dir: true,
            display_path: PathBuf::from("."),
            children: vec![Node {
                name: "src".to_string(),
                path: PathBuf::from("src"),
                is_dir: true,
                display_path: PathBuf::from("src"),
                children: vec![Node {
                    name: "main.rs".to_string(),
                    path: PathBuf::from("src/main.rs"),
                    is_dir: false,
                    children: vec![],
                    display_path: PathBuf::from("src/main.rs"),
                }],
            }],
        };

        let output = renderer.render_tree(&root);

        assert!(
            output.contains("<details"),
            "Should have details tag when fold is on"
        );
        assert!(output.contains("<summary>"), "Should have summary tag");
        assert!(output.contains("</details>"), "Should close details tag");
        assert!(output.contains("src/"), "Should show directory name");
    }

    #[test]
    fn test_html_output_format() {
        let args = create_test_args();
        let renderer = HtmlRenderer::new(&args);

        assert_eq!(renderer.output_format(), OutputFormat::Html);
    }
}
