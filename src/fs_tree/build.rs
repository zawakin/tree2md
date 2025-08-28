use super::node::Node;
use crate::cli::Args;
use crate::util::path::calculate_display_path;
use glob::Pattern;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Build tree using WalkBuilder for unified gitignore support
pub fn build_tree(
    path: &str,
    args: &Args,
    patterns: &[Pattern],
    root_path: &Path,
    display_root: &Path,
) -> io::Result<Node> {
    let path_buf = Path::new(path);
    let metadata = fs::metadata(path_buf)?;
    let name = path_buf
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("."))
        .to_string_lossy()
        .to_string();

    let resolved_path = path_buf
        .canonicalize()
        .unwrap_or_else(|_| path_buf.to_path_buf());

    let display_path = calculate_display_path(
        &resolved_path,
        &args.display_path,
        display_root,
        None,
        &args.strip_prefix,
    );

    let mut root_node = Node::new(name, resolved_path.clone(), metadata.is_dir())
        .with_display_path(display_path);

    if metadata.is_dir() {
        // Use WalkBuilder for recursive directory traversal with gitignore support
        let mut walker = WalkBuilder::new(path);
        walker
            .hidden(!args.all)
            .git_ignore(args.respect_gitignore)
            .git_global(args.respect_gitignore)
            .git_exclude(args.respect_gitignore)
            .parents(args.respect_gitignore)
            .ignore(args.respect_gitignore)
            .max_depth(None);

        // Build a map of paths to nodes for efficient tree construction
        let mut nodes_map: HashMap<PathBuf, Node> = HashMap::new();

        for entry in walker.build() {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let entry_path = entry.path();

            // Skip the root directory itself
            if entry_path == path_buf {
                continue;
            }

            // Skip if path cannot be converted to string (non-UTF8 paths)
            if entry_path.to_str().is_none() {
                eprintln!("Warning: Skipping non-UTF8 path: {:?}", entry_path);
                continue;
            }

            let entry_metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Apply pattern filtering only to files
            // Directories are always kept so children can attach properly in the tree
            if entry_metadata.is_file()
                && !patterns.is_empty()
                && !path_matches_patterns(entry_path, patterns, root_path)
            {
                continue;
            }

            let entry_name = entry_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("."))
                .to_string_lossy()
                .to_string();

            let resolved_entry_path = entry_path
                .canonicalize()
                .unwrap_or_else(|_| entry_path.to_path_buf());

            let entry_display_path = calculate_display_path(
                &resolved_entry_path,
                &args.display_path,
                display_root,
                None,
                &args.strip_prefix,
            );

            let node = Node::new(entry_name, resolved_entry_path, entry_metadata.is_dir())
                .with_display_path(entry_display_path);

            nodes_map.insert(entry_path.to_path_buf(), node);
        }

        // Build the tree structure from the flat map
        build_tree_from_map(&mut root_node, &nodes_map, path_buf)?;
        
        // Remove empty directories when patterns are specified
        if !patterns.is_empty() {
            remove_empty_directories(&mut root_node);
        }
    }

    Ok(root_node)
}

pub fn build_tree_from_map(
    parent: &mut Node,
    nodes_map: &HashMap<PathBuf, Node>,
    base_path: &Path,
) -> io::Result<()> {
    let mut direct_children: Vec<PathBuf> = Vec::new();

    // Find direct children of the parent
    for path in nodes_map.keys() {
        if let Some(parent_path) = path.parent() {
            if parent_path == base_path {
                direct_children.push(path.clone());
            }
        }
    }

    // Sort children: directories first, then files, alphabetically within each group
    direct_children.sort_by(|a, b| {
        let a_node = nodes_map.get(a).unwrap();
        let b_node = nodes_map.get(b).unwrap();

        match (a_node.is_dir, b_node.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a_node.name.cmp(&b_node.name),
        }
    });

    // Add children to parent and recursively build their subtrees
    for child_path in direct_children {
        if let Some(child_node) = nodes_map.get(&child_path) {
            let mut child = child_node.clone();
            if child.is_dir {
                build_tree_from_map(&mut child, nodes_map, &child_path)?;
            }
            parent.children.push(child);
        }
    }

    Ok(())
}

fn path_matches_patterns(path: &Path, patterns: &[Pattern], root_path: &Path) -> bool {
    // Try to get relative path without canonicalize for performance
    if let Ok(relative_path) = path.strip_prefix(root_path) {
        // Convert path to use forward slashes for consistent pattern matching
        let path_str = relative_path.to_string_lossy().replace('\\', "/");
        for pattern in patterns {
            if pattern.matches(&path_str) {
                return true;
            }
        }
    }
    
    false
}

/// Remove empty directories from the tree
fn remove_empty_directories(node: &mut Node) {
    if !node.is_dir {
        return;
    }
    
    // Recursively process children first
    for child in &mut node.children {
        if child.is_dir {
            remove_empty_directories(child);
        }
    }
    
    // Remove empty directory children
    node.children.retain(|child| {
        !child.is_dir || !child.children.is_empty()
    });
}

pub fn insert_path_into_tree(
    root: &mut Node,
    path: &Path,
    common_ancestor: &Option<PathBuf>,
    args: &Args,
    display_root: &Path,
    original_input: Option<String>,
) {
    let components: Vec<_> = if let Some(ref ancestor) = common_ancestor {
        path.strip_prefix(ancestor)
            .unwrap_or(path)
            .components()
            .collect()
    } else {
        path.components().collect()
    };

    let mut current_children = &mut root.children;

    for (i, component) in components.iter().enumerate() {
        let name = component.as_os_str().to_string_lossy().to_string();
        let is_last = i == components.len() - 1;

        // Check if child already exists
        let child_pos = current_children
            .iter()
            .position(|child| child.name == *name);

        if let Some(pos) = child_pos {
            if !is_last {
                // Navigate deeper
                current_children = &mut current_children[pos].children;
            }
        } else {
            // Create new node
            let node_path = if is_last {
                path.to_path_buf()
            } else {
                PathBuf::new()
            };

            let display_path = if is_last && !node_path.as_os_str().is_empty() {
                calculate_display_path(
                    &node_path,
                    &args.display_path,
                    display_root,
                    original_input.as_deref(),
                    &args.strip_prefix,
                )
            } else {
                PathBuf::from(&name)
            };

            let new_node = Node::new(name.clone(), node_path, !is_last)
                .with_display_path(display_path)
                .with_original_input(original_input.clone());

            current_children.push(new_node);

            if !is_last {
                // Navigate to the newly created node's children
                let new_pos = current_children.len() - 1;
                current_children = &mut current_children[new_pos].children;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use clap::Parser;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_pattern_filtering_keeps_directories() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        // Create nested structure
        fs::create_dir_all(root.join("src/a/b")).unwrap();
        fs::write(root.join("src/a/b/test.rs"), "fn main() {}").unwrap();
        fs::write(root.join("src/a/b/data.txt"), "data").unwrap();
        
        let args = Args::parse_from(&["tree2md", root.to_str().unwrap(), "-f", "**/*.rs"]);
        let patterns = crate::filter::compile_patterns(&args.find_patterns).unwrap();
        let display_root = root.to_path_buf();
        
        let tree = build_tree(
            root.to_str().unwrap(),
            &args,
            &patterns,
            root,
            &display_root,
        ).unwrap();
        
        // Verify src directory exists
        let src = tree.children.iter().find(|n| n.name == "src");
        assert!(src.is_some(), "src directory should exist");
        
        let src = src.unwrap();
        assert!(src.is_dir);
        
        // Verify a directory exists under src
        let a = src.children.iter().find(|n| n.name == "a");
        assert!(a.is_some(), "a directory should exist");
        
        let a = a.unwrap();
        assert!(a.is_dir);
        
        // Verify b directory exists under a
        let b = a.children.iter().find(|n| n.name == "b");
        assert!(b.is_some(), "b directory should exist");
        
        let b = b.unwrap();
        assert!(b.is_dir);
        
        // Verify only .rs file exists, not .txt
        assert_eq!(b.children.len(), 1);
        assert_eq!(b.children[0].name, "test.rs");
    }
    
    #[test]
    fn test_empty_directories_removed_with_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        // Create structure with empty directories
        fs::create_dir_all(root.join("empty/nested/deep")).unwrap();
        fs::create_dir_all(root.join("has_match")).unwrap();
        fs::write(root.join("has_match/test.rs"), "fn main() {}").unwrap();
        
        let args = Args::parse_from(&["tree2md", root.to_str().unwrap(), "-f", "**/*.rs"]);
        let patterns = crate::filter::compile_patterns(&args.find_patterns).unwrap();
        let display_root = root.to_path_buf();
        
        let tree = build_tree(
            root.to_str().unwrap(),
            &args,
            &patterns,
            root,
            &display_root,
        ).unwrap();
        
        // Verify empty directories are removed
        assert!(tree.children.iter().find(|n| n.name == "empty").is_none(), 
                "empty directory tree should be removed");
        
        // Verify directory with matching file is kept
        assert!(tree.children.iter().find(|n| n.name == "has_match").is_some(),
                "directory with matching file should be kept");
    }
}