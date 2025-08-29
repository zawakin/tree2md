mod cli;
mod content;
mod fs_tree;
mod input;
mod language;
mod matcher;
mod util;

use clap::Parser;
use cli::Args;
use fs_tree::{
    build_tree, insert_path_into_tree, print_code_blocks, print_flat_structure,
    print_tree_with_options, Node,
};
use input::{find_common_ancestor, process_stdin_input, StdinConfig, StdinError, StdinResult};
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Handle stdin mode
    if args.stdin {
        return handle_stdin_mode(&args);
    }

    // Determine display root
    let display_root = Path::new(&args.directory)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.directory));

    // Get the root path for pattern matching
    let root_path = Path::new(&args.directory)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(&args.directory).to_path_buf());

    // Create args with effective gitignore setting for directory scan
    // Build tree using unified WalkBuilder approach
    let root_node = build_tree(&args.directory, &args, &root_path, &display_root)?;

    // Print structure based on format preference
    println!("## File Structure");

    if args.flat {
        // Collect all file paths for flat output
        let mut all_paths = Vec::new();
        collect_paths_from_node(&root_node, &mut all_paths);
        all_paths.sort();

        // Print in flat format
        print_flat_structure(
            &all_paths,
            &args,
            &display_root,
            &std::collections::HashMap::new(),
        );

        // Print code blocks if requested
        if args.contents {
            print_code_blocks(&root_node, &args);
        }
    } else {
        // Print tree structure
        // Show root only if --root-label is specified
        let show_root = args.root_label.is_some();
        print_tree_with_options(&root_node, "", &args, show_root);

        // Print code blocks if requested
        if args.contents {
            print_code_blocks(&root_node, &args);
        }
    }

    Ok(())
}

fn handle_stdin_mode(args: &Args) -> io::Result<()> {
    // Use the positional directory argument as base for resolving relative paths from stdin
    let base_dir = Path::new(&args.directory)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.directory));

    let stdin_config = StdinConfig {
        base_dir,
        restrict_root: args.restrict_root.as_ref().map(PathBuf::from),
        expand_dirs: args.expand_dirs,
        // When expanding dirs, respect gitignore by default (unless --no-gitignore)
        respect_gitignore: if args.expand_dirs {
            !args.no_gitignore
        } else {
            false // Not expanding, so this field doesn't matter
        },
        exclude_hidden: args.exclude_hidden,
    };

    // Process stdin input and get both canonical paths and original inputs
    let StdinResult {
        canonical_paths: file_paths,
        original_map: original_inputs,
    } = match process_stdin_input(&stdin_config) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: {}", e);
            match e {
                StdinError::RestrictRootViolation(_, _) => std::process::exit(2),
                StdinError::DirectoriesNotAllowed(_) => std::process::exit(3),
                StdinError::NoValidFiles => std::process::exit(4),
                _ => std::process::exit(1),
            }
        }
    };

    // Always use authoritative mode (stdin only) and preserve input order
    let all_paths = file_paths;

    // Use current directory as display root for stdin mode
    let display_root = std::env::current_dir()?;

    // Extension filtering is already handled during stdin processing/expansion

    // Generate output
    if args.flat {
        print_flat_structure(&all_paths, args, &display_root, &original_inputs);
    } else {
        // Build tree from paths
        let common_ancestor = find_common_ancestor(&all_paths);
        let mut root = Node {
            name: common_ancestor
                .as_ref()
                .and_then(|p| p.file_name())
                .unwrap_or_else(|| std::ffi::OsStr::new("."))
                .to_string_lossy()
                .to_string(),
            path: common_ancestor
                .clone()
                .unwrap_or_else(|| PathBuf::from(".")),
            display_path: PathBuf::from("."),
            is_dir: true,
            children: Vec::new(),
            original_input: None,
        };

        for path in &all_paths {
            let original_input = original_inputs.get(path).cloned();
            insert_path_into_tree(
                &mut root,
                path,
                &common_ancestor,
                args,
                &display_root,
                original_input,
            );
        }

        println!("## File Structure");
        // Show root only if --root-label is specified
        let show_root = args.root_label.is_some();
        print_tree_with_options(&root, "", args, show_root);

        if args.contents {
            print_code_blocks(&root, args);
        }
    }

    Ok(())
}

fn collect_paths_from_node(node: &Node, paths: &mut Vec<PathBuf>) {
    if !node.is_dir && !node.path.as_os_str().is_empty() {
        paths.push(node.path.clone());
    }
    for child in &node.children {
        collect_paths_from_node(child, paths);
    }
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
