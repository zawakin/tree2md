pub mod html;
pub mod markdown;
pub mod pipeline;
pub mod renderer;
pub mod terminal;

pub use html::HtmlRenderer;
pub use markdown::MarkdownRenderer;
pub use renderer::Renderer;
pub use terminal::TerminalRenderer;

use crate::cli::{Args, OutputMode};
use crate::terminal::capabilities::TerminalCapabilities;
use crate::terminal::detect::TerminalDetector;

/// Create the appropriate renderer based on configuration
pub fn create_renderer<'a>(
    args: &'a Args,
    _capabilities: &TerminalCapabilities,
) -> Box<dyn Renderer + 'a> {
    let detector = TerminalDetector::new();
    let is_tty = detector.is_tty();
    let output_format = args.output_format(is_tty);

    match output_format {
        OutputMode::Tty => Box::new(TerminalRenderer::new(args)),
        OutputMode::Md => Box::new(MarkdownRenderer::new(args)),
        OutputMode::Html => Box::new(HtmlRenderer::new(args)),
        OutputMode::Auto => {
            // This shouldn't happen since output_format resolves Auto
            // But fallback to Markdown for safety
            Box::new(MarkdownRenderer::new(args))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{FunMode, LinksMode, LocMode, StatsMode};
    use crate::render::renderer::OutputFormat;

    fn create_test_args(output: OutputMode) -> Args {
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
            output,
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
    fn test_create_terminal_renderer() {
        let args = create_test_args(OutputMode::Tty);
        let capabilities = TerminalCapabilities::new();
        let renderer = create_renderer(&args, &capabilities);

        assert_eq!(renderer.output_format(), OutputFormat::Terminal);
    }

    #[test]
    fn test_create_markdown_renderer() {
        let args = create_test_args(OutputMode::Md);
        let capabilities = TerminalCapabilities::new();
        let renderer = create_renderer(&args, &capabilities);

        assert_eq!(renderer.output_format(), OutputFormat::Markdown);
    }

    #[test]
    fn test_create_auto_renderer() {
        // When OutputMode::Auto is passed, it gets resolved based on is_tty
        // In test environment (not TTY), it resolves to Markdown
        let args = create_test_args(OutputMode::Auto);
        let capabilities = TerminalCapabilities::new();
        let renderer = create_renderer(&args, &capabilities);

        // In test environment, Auto resolves to Markdown (since it's not a TTY)
        assert_eq!(renderer.output_format(), OutputFormat::Markdown);
    }
}
