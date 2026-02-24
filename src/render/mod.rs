pub mod pipe;
pub mod pipeline;
pub mod renderer;
pub mod terminal;

pub use pipe::PipeRenderer;
pub use renderer::Renderer;
pub use terminal::TerminalRenderer;

use crate::cli::Args;
use crate::terminal::capabilities::TerminalCapabilities;
use crate::terminal::detect::TerminalDetector;

/// Create the appropriate renderer based on TTY detection
pub fn create_renderer<'a>(
    args: &'a Args,
    _capabilities: &TerminalCapabilities,
) -> Box<dyn Renderer + 'a> {
    let detector = TerminalDetector::new();
    let is_tty = detector.is_tty();

    if is_tty {
        Box::new(TerminalRenderer::new(args))
    } else {
        Box::new(PipeRenderer::new(args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{FunMode, LocMode, StatsMode};
    use crate::render::renderer::OutputFormat;

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
    fn test_create_renderer_in_test_env() {
        // In test environment (not TTY), it resolves to Pipe
        let args = create_test_args();
        let capabilities = TerminalCapabilities::new();
        let renderer = create_renderer(&args, &capabilities);

        assert_eq!(renderer.output_format(), OutputFormat::Pipe);
    }
}
