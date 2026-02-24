mod cli;
mod content;
mod fs_tree;
mod language;
mod matcher;
mod output;
mod profile;
mod render;
mod safety;
mod terminal;
mod util;

use clap::Parser;
use cli::Args;
use fs_tree::{build_tree, ProgressTracker};
use std::io;
use std::path::Path;
use terminal::animation::AnimationRunner;
use terminal::capabilities::TerminalCapabilities;
use terminal::detect::TerminalDetector;

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Determine display root
    let display_root = Path::new(&args.target)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(&args.target));

    // Get the root path for pattern matching
    let root_path = Path::new(&args.target)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(&args.target).to_path_buf());

    // Set up progress tracking and animation
    let detector = TerminalDetector::new();
    let is_tty = detector.is_tty();
    let show_animation = args.is_fun_enabled(is_tty) && !args.no_anim && is_tty;

    let progress_tracker = if show_animation {
        Some(ProgressTracker::new())
    } else {
        None
    };

    let mut animation_runner = AnimationRunner::new(show_animation, progress_tracker.clone());

    // Build tree using unified WalkBuilder approach
    let root_node = build_tree(&args.target, &args, &root_path, &display_root)?;

    // Stop animation once tree is built
    animation_runner.complete();

    // Create terminal capabilities and renderer
    let capabilities = TerminalCapabilities::new();
    let mut renderer = render::create_renderer(&args, &capabilities);
    let output = renderer.render_tree(&root_node);

    // Print to stdout
    print!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use language::detect_lang;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_lang() {
        assert_eq!(detect_lang("test.rs").map(|l| l.name), Some("rust"));
        assert_eq!(detect_lang("test.go").map(|l| l.name), Some("go"));
        assert_eq!(detect_lang("test.py").map(|l| l.name), Some("python"));
        assert_eq!(detect_lang("test.unknown"), None);
    }

    #[test]
    fn test_build_tree() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::create_dir(temp_path.join("src")).unwrap();
        fs::write(temp_path.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(temp_path.join("README.md"), "# Test").unwrap();

        let args = Args::parse_from(&["tree2md", temp_path.to_str().unwrap()]);
        let display_root = temp_path.to_path_buf();
        let tree =
            build_tree(temp_path.to_str().unwrap(), &args, temp_path, &display_root).unwrap();

        assert!(tree.is_dir);
        assert!(tree.children.len() >= 2);
    }
}
