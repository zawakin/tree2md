mod cli;
mod content;
mod filter;
mod fs_tree;
mod input;
mod language;
mod util;

use clap::Parser;
use cli::{Args, StdinMode};
use filter::{compile_patterns, filter_by_extension, parse_ext_list};
use fs_tree::{build_tree, insert_path_into_tree, print_code_blocks, print_flat_structure, print_tree_with_options, Node};
use input::{find_common_ancestor, process_stdin_input, StdinConfig, StdinError, StdinResult};
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Handle stdin mode
    if args.stdin || args.stdin0 {
        return handle_stdin_mode(&args);
    }

    // Determine display root
    let display_root = determine_display_root(&args, &[PathBuf::from(&args.directory)])?;

    // Show root if requested
    if args.show_root {
        println!("Display root: {}\n", display_root.display());
    }

    // Compile wildcard patterns
    let patterns = compile_patterns(&args.find_patterns)?;

    // Get the root path for pattern matching
    let root_path = Path::new(&args.directory)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(&args.directory).to_path_buf());

    // Build tree using unified WalkBuilder approach
    let root_node = build_tree(&args.directory, &args, &patterns, &root_path, &display_root)?;

    // Filter by extensions if specified
    let mut root_node = root_node;
    if let Some(ref ext_str) = args.include_ext {
        let exts = parse_ext_list(ext_str);
        filter_by_extension(&mut root_node, &exts);
    }

    // Print tree structure
    println!("## File Structure");
    // For non-stdin mode, show root by default unless --no-root is specified
    let show_root = !args.no_root;
    print_tree_with_options(&root_node, "", &args, show_root);

    // Print code blocks if requested
    if args.contents {
        print_code_blocks(&root_node, &args);
    }

    Ok(())
}

fn handle_stdin_mode(args: &Args) -> io::Result<()> {
    // Respect gitignore by default in stdin merge mode
    let _respect_gitignore = match args.stdin_mode {
        StdinMode::Merge => args.respect_gitignore,
        StdinMode::Authoritative => args.respect_gitignore,
    };

    let stdin_config = StdinConfig {
        null_delimited: args.stdin0,
        base_dir: PathBuf::from(&args.base),
        restrict_root: args.restrict_root.as_ref().map(PathBuf::from),
        expand_dirs: args.expand_dirs,
        keep_order: args.keep_order,
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

    let mut all_paths = file_paths;

    // Handle merge mode
    if matches!(args.stdin_mode, StdinMode::Merge) {
        let patterns = compile_patterns(&args.find_patterns)?;
        let root_path = Path::new(&args.directory)
            .canonicalize()
            .unwrap_or_else(|_| Path::new(&args.directory).to_path_buf());

        // Determine display root for merge mode
        let display_root = determine_display_root(args, &all_paths)?;

        let dir_node = build_tree(
            &args.directory,
            args,
            &patterns,
            &root_path,
            &display_root,
        )?;

        // Collect files from directory tree
        let mut dir_files = Vec::new();
        collect_files_from_tree(&dir_node, &mut dir_files);

        // Merge with stdin files
        for file in dir_files {
            if !all_paths.contains(&file) {
                all_paths.push(file);
            }
        }

        // Sort if not keeping order
        if !args.keep_order {
            all_paths.sort();
        }
    }

    // Determine display root
    let display_root = determine_display_root(args, &all_paths)?;

    // Show root if requested
    if args.show_root {
        println!("Display root: {}\n", display_root.display());
    }

    // Filter by extensions if specified
    if let Some(ref ext_str) = args.include_ext {
        let exts = parse_ext_list(ext_str);
        all_paths.retain(|path| {
            if let Some(ext) = path.extension() {
                let ext_str = format!(".{}", ext.to_string_lossy().to_lowercase());
                exts.contains(&ext_str)
            } else {
                false
            }
        });
    }

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
        // For stdin mode, default to no root unless explicitly set
        let show_root = !args.no_root && (args.root_label.is_some() || !args.stdin);
        print_tree_with_options(&root, "", args, show_root);

        if args.contents {
            print_code_blocks(&root, args);
        }
    }

    Ok(())
}

fn determine_display_root(args: &Args, paths: &[PathBuf]) -> io::Result<PathBuf> {
    if let Some(ref display_root_str) = args.display_root {
        // User specified display root
        let display_root = Path::new(display_root_str);
        if !display_root.exists() {
            eprintln!(
                "Warning: Display root '{}' does not exist, using current directory",
                display_root_str
            );
            Ok(std::env::current_dir()?)
        } else {
            Ok(display_root.canonicalize()?)
        }
    } else {
        // Auto-detect display root
        if args.stdin || args.stdin0 {
            // For stdin mode, use LCA of all paths
            if let Some(lca) = find_common_ancestor(paths) {
                Ok(lca)
            } else {
                Ok(std::env::current_dir()?)
            }
        } else {
            // For directory scan mode, use the scan directory
            Path::new(&args.directory)
                .canonicalize()
                .or_else(|_| std::env::current_dir())
        }
    }
}

fn collect_files_from_tree(node: &Node, files: &mut Vec<PathBuf>) {
    if !node.is_dir && !node.path.as_os_str().is_empty() {
        files.push(node.path.clone());
    }
    for child in &node.children {
        collect_files_from_tree(child, files);
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
        let patterns = Vec::new();
        let tree = build_tree(
            temp_path.to_str().unwrap(),
            &args,
            &patterns,
            temp_path,
            &display_root,
        )
        .unwrap();

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