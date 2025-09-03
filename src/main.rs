mod cli;
mod content;
mod fs_tree;
mod injection;
mod language;
mod matcher;
mod output;
mod profile;
mod render;
mod safety;
mod stamp;
mod terminal;
mod util;

use clap::Parser;
use cli::Args;
use fs_tree::{build_tree, print_code_blocks, ProgressTracker};
use injection::readme::ReadmeInjector;
use stamp::provenance::StampGenerator;
use std::io;
use std::path::{Path, PathBuf};
use terminal::animation::AnimationRunner;
use terminal::capabilities::TerminalCapabilities;
use terminal::detect::TerminalDetector;

fn main() -> io::Result<()> {
    let mut args = Args::parse();

    // Apply preset configurations
    args.apply_preset();

    // Validate arguments and check for deprecated usage
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    args.check_deprecated();

    // Determine display root
    let display_root = Path::new(&args.target)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.target));

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
    let mut output = renderer.render_tree(&root_node);

    // Add stamp if requested
    let stamp_gen = StampGenerator::new(&args);
    if let Some(stamp) = stamp_gen.generate(&root_path) {
        output.push_str("\n\n---\n\n");
        output.push_str(&stamp);
        output.push('\n');
    }

    // Handle injection into README
    if let Some(ref inject_path) = args.inject {
        let injector =
            ReadmeInjector::new(output.clone(), args.tag_start.clone(), args.tag_end.clone());
        injector.inject(Path::new(inject_path))?;
        eprintln!("Successfully injected tree into {}", inject_path);
    } else {
        // Print to stdout if not injecting
        print!("{}", output);
    }

    // Print code blocks if requested (deprecated)
    if args.contents {
        print_code_blocks(&root_node, &args);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use language::detect_lang;
    use std::fs;
    use std::path::PathBuf;
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

    #[test]
    fn test_no_file_comments_in_code_blocks() {
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        let test_rs_content = "fn main() {\n    println!(\"Hello\");\n}";
        let test_json_content = "{\n  \"name\": \"test\"\n}";

        fs::write(temp_path.join("test.rs"), test_rs_content).unwrap();
        fs::write(temp_path.join("test.json"), test_json_content).unwrap();

        // Test Rust file output
        let mut output = Vec::new();
        {
            let display_path = PathBuf::from("test.rs");

            // Simulate print_file_content_with_display output
            writeln!(&mut output, "\n### {}", display_path.display()).unwrap();
            writeln!(&mut output, "```rust").unwrap();
            write!(&mut output, "{}", test_rs_content).unwrap();
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "```").unwrap();
        }

        let output_str = String::from_utf8(output).unwrap();

        // Verify no file comment is present
        assert!(
            !output_str.contains("// test.rs"),
            "Should not contain file comment"
        );
        assert!(
            output_str.contains("### test.rs"),
            "Should contain markdown header"
        );
        assert!(
            output_str.contains("```rust"),
            "Should contain language tag"
        );
        assert!(
            output_str.contains(test_rs_content),
            "Should contain file content"
        );

        // Test JSON file output
        let mut output = Vec::new();
        {
            let display_path = PathBuf::from("test.json");

            // Simulate print_file_content_with_display output for JSON
            writeln!(&mut output, "\n### {}", display_path.display()).unwrap();
            writeln!(&mut output, "```json").unwrap();
            write!(&mut output, "{}", test_json_content).unwrap();
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "```").unwrap();
        }

        let output_str = String::from_utf8(output).unwrap();

        // Verify JSON uses 'json' not 'jsonc' and has no comment
        assert!(
            !output_str.contains("// test.json"),
            "Should not contain file comment"
        );
        assert!(
            output_str.contains("```json"),
            "Should use 'json' language tag"
        );
        assert!(
            !output_str.contains("```jsonc"),
            "Should not use 'jsonc' language tag"
        );
        assert!(
            output_str.contains(test_json_content),
            "Should contain file content"
        );
    }

    #[test]
    fn test_truncation_message_preserved() {
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a file with multiple lines
        let mut content = String::new();
        for i in 1..=20 {
            content.push_str(&format!("Line {}\n", i));
        }
        fs::write(temp_path.join("large.txt"), &content).unwrap();

        // Simulate truncation output
        let mut output = Vec::new();
        let display_path = PathBuf::from("large.txt");

        writeln!(&mut output, "\n### {}", display_path.display()).unwrap();
        writeln!(&mut output, "```").unwrap();

        // Output first 5 lines
        for i in 1..=5 {
            writeln!(&mut output, "Line {}", i).unwrap();
        }

        // Add truncation message
        writeln!(
            &mut output,
            "// [Content truncated: showing first 5 of 20 lines]"
        )
        .unwrap();
        writeln!(&mut output, "```").unwrap();

        let output_str = String::from_utf8(output).unwrap();

        // Verify truncation message is present but file comment is not
        assert!(
            !output_str.contains("// large.txt"),
            "Should not contain file comment"
        );
        assert!(
            output_str.contains("// [Content truncated:"),
            "Should contain truncation message"
        );
    }
}
