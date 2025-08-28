use super::node::Node;
use crate::cli::Args;
use crate::content::read::print_file_content_with_display;
use crate::util::path::calculate_display_path;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn print_tree_with_options(node: &Node, prefix: &str, args: &Args, show_root: bool) {
    if show_root {
        // Show root with custom label if provided
        let root_name = args.root_label.as_deref().unwrap_or(&node.name);
        if !root_name.is_empty() {
            // Add trailing slash for root directory
            let display_name = if node.is_dir {
                format!("{}/", root_name)
            } else {
                root_name.to_string()
            };
            println!("{}- {}", prefix, display_name);
        }
        for child in &node.children {
            let child_prefix = format!("{}  ", prefix);
            print_tree(child, &child_prefix);
        }
    } else {
        // Skip root node, print children directly
        for child in &node.children {
            print_tree(child, prefix);
        }
    }
}

fn print_tree(node: &Node, prefix: &str) {
    if !node.name.is_empty() {
        // Add trailing slash for directories
        let display_name = if node.is_dir {
            format!("{}/", node.name)
        } else {
            node.name.clone()
        };
        println!("{}- {}", prefix, display_name);
    }

    for child in &node.children {
        let child_prefix = format!("{}  ", prefix);
        print_tree(child, &child_prefix);
    }
}

pub fn print_code_blocks(node: &Node, args: &Args) {
    if !node.is_dir && !node.path.as_os_str().is_empty() {
        print_file_content_with_display(&node.path, &node.display_path, args);
    }

    for child in &node.children {
        print_code_blocks(child, args);
    }
}

pub fn print_flat_structure(
    paths: &[PathBuf],
    args: &Args,
    display_root: &Path,
    original_inputs: &HashMap<PathBuf, String>,
) {
    println!("## File Structure");
    for path in paths {
        let original_input = original_inputs.get(path).map(|s| s.as_str());
        let display_path = calculate_display_path(
            path,
            &args.display_path,
            display_root,
            original_input,
            &args.strip_prefix,
        );
        println!("- {}", display_path.display());
    }

    if args.contents {
        for path in paths {
            if path.is_file() {
                let original_input = original_inputs.get(path).map(|s| s.as_str());
                let display_path = calculate_display_path(
                    path,
                    &args.display_path,
                    display_root,
                    original_input,
                    &args.strip_prefix,
                );
                print_file_content_with_display(path, &display_path, args);
            }
        }
    }
}