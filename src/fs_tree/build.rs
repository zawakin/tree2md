use super::node::Node;
use crate::cli::Args;
use crate::matcher::{MatchSpec, MatcherEngine, RelPath, Selection};
use crate::util::path::calculate_display_path;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Build tree using WalkBuilder for unified gitignore support with MatcherEngine
pub fn build_tree(
    path: &str,
    args: &Args,
    root_path: &Path,
    display_root: &Path,
) -> io::Result<Node> {
    // Create MatchSpec from CLI arguments
    let spec = MatchSpec::from_args(args, Path::new(path));
    build_tree_with_spec(path, args, &spec, root_path, display_root)
}

/// Build tree using the MatcherEngine architecture
pub fn build_tree_with_spec(
    path: &str,
    args: &Args,
    spec: &MatchSpec,
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

    let display_path = calculate_display_path(&resolved_path, display_root);

    let mut root_node =
        Node::new(name, resolved_path.clone(), metadata.is_dir()).with_display_path(display_path);

    if metadata.is_dir() {
        // Compile the matcher engine
        let matcher = MatcherEngine::compile(spec, root_path)?;

        // Use WalkBuilder for recursive directory traversal
        let mut walker = WalkBuilder::new(path);
        walker
            .hidden(false) // Don't exclude hidden files by default (will be handled by patterns)
            .git_ignore(false) // We handle gitignore in MatcherEngine
            .git_global(false)
            .git_exclude(false)
            .parents(false)
            .ignore(false)
            .follow_links(false) // Skip symlinks as per spec
            .max_depth(args.level); // Use level directly

        // Build a map of paths to nodes for efficient tree construction
        let mut nodes_map: HashMap<PathBuf, Node> = HashMap::new();
        let mut pruned_dirs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

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

            // Skip symlinks entirely (per spec: "Symlinks are always skipped")
            if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                continue;
            }

            // Skip if path cannot be converted to string (non-UTF8 paths)
            if entry_path.to_str().is_none() {
                eprintln!("Warning: Skipping non-UTF8 path: {:?}", entry_path);
                continue;
            }

            // Check if this path is under a pruned directory
            let is_under_pruned = pruned_dirs
                .iter()
                .any(|pruned| entry_path.starts_with(pruned));

            if is_under_pruned {
                continue;
            }

            let entry_metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Create RelPath for matching
            let rel_path = match RelPath::from_root_rel(entry_path, root_path) {
                Some(rp) => rp,
                None => continue,
            };

            // Apply matcher engine selection
            let selection = if entry_metadata.is_dir() {
                matcher.select_dir(&rel_path)
            } else {
                matcher.select_file(&rel_path)
            };

            match selection {
                Selection::PruneDir => {
                    // Mark this directory as pruned so we skip its children
                    pruned_dirs.insert(entry_path.to_path_buf());
                    continue;
                }
                Selection::Exclude => {
                    continue;
                }
                Selection::Include => {
                    // Include this file/dir in the tree
                }
            }

            let entry_name = entry_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("."))
                .to_string_lossy()
                .to_string();

            let resolved_entry_path = entry_path
                .canonicalize()
                .unwrap_or_else(|_| entry_path.to_path_buf());

            let entry_display_path = calculate_display_path(&resolved_entry_path, display_root);

            let node = Node::new(entry_name, resolved_entry_path, entry_metadata.is_dir())
                .with_display_path(entry_display_path);

            nodes_map.insert(entry_path.to_path_buf(), node);
        }

        // Build the tree structure from the flat map
        build_tree_from_map(&mut root_node, &nodes_map, path_buf)?;

        // PruneDir prevents descending into directories but doesn't remove them from output.
        // When include rules filter out all files in a directory, we need to clean up empty dirs.
        if spec.has_includes() {
            remove_empty_directories(&mut root_node);
        }
    }

    Ok(root_node)
}

fn build_tree_from_map(
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

/// Remove directories that have no children after filtering.
/// This is needed because PruneDir only prevents descending, it doesn't remove the directory node.
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
    node.children
        .retain(|child| !child.is_dir || !child.children.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use crate::matcher::MatchSpec;
    use clap::Parser;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_matcher_engine_integration() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test structure
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn lib() {}").unwrap();
        fs::write(root.join("src/test.txt"), "test").unwrap();

        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::write(root.join("target/debug/app"), "binary").unwrap();

        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".gitignore"), "target/").unwrap();

        let args = Args::parse_from(&["tree2md", root.to_str().unwrap()]);

        // Test with extension filter
        let spec = MatchSpec::new().with_include_ext(vec![".rs".to_string()]);

        let display_root = root.to_path_buf();
        let tree = build_tree_with_spec(root.to_str().unwrap(), &args, &spec, root, &display_root)
            .unwrap();

        // Should have src directory
        let src = tree.children.iter().find(|n| n.name == "src");
        assert!(src.is_some(), "src directory should exist");

        let src = src.unwrap();
        // Should have two .rs files but not .txt
        assert_eq!(src.children.len(), 2, "Should have two .rs files");
        assert!(src.children.iter().any(|n| n.name == "main.rs"));
        assert!(src.children.iter().any(|n| n.name == "lib.rs"));
        assert!(!src.children.iter().any(|n| n.name == "test.txt"));

        // target directory should be pruned (no matching files)
        assert!(tree.children.iter().find(|n| n.name == "target").is_none());
    }

    #[test]
    fn test_gitignore_pruning() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test structure with gitignore
        fs::write(root.join(".gitignore"), "target/\n*.tmp").unwrap();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::write(root.join("target/debug/app"), "binary").unwrap();

        fs::write(root.join("temp.tmp"), "temporary").unwrap();
        fs::write(root.join("data.txt"), "data").unwrap();

        let args = Args::parse_from(&["tree2md", root.to_str().unwrap()]);

        // Test with gitignore enabled
        let spec = MatchSpec::new().with_gitignore(true);

        let display_root = root.to_path_buf();
        let tree = build_tree_with_spec(root.to_str().unwrap(), &args, &spec, root, &display_root)
            .unwrap();

        // Should have src and data.txt, but not target or temp.tmp
        assert!(tree.children.iter().any(|n| n.name == "src"));
        assert!(tree.children.iter().any(|n| n.name == "data.txt"));
        assert!(!tree.children.iter().any(|n| n.name == "target"));
        assert!(!tree.children.iter().any(|n| n.name == "temp.tmp"));
    }

    #[test]
    fn test_glob_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create nested structure
        fs::create_dir_all(root.join("src/module")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("src/module/lib.rs"), "pub fn lib() {}").unwrap();
        fs::write(root.join("test.rs"), "test").unwrap();
        fs::write(root.join("README.md"), "readme").unwrap();

        let args = Args::parse_from(&["tree2md", root.to_str().unwrap()]);

        // Test with glob pattern
        let spec =
            MatchSpec::new().with_include_glob(vec!["src/**/*.rs".to_string(), "*.md".to_string()]);

        let display_root = root.to_path_buf();
        let tree = build_tree_with_spec(root.to_str().unwrap(), &args, &spec, root, &display_root)
            .unwrap();

        // Should have README.md at root
        assert!(tree.children.iter().any(|n| n.name == "README.md"));

        // Should NOT have test.rs at root (doesn't match pattern)
        assert!(!tree.children.iter().any(|n| n.name == "test.rs"));

        // Should have src with nested structure
        let src = tree.children.iter().find(|n| n.name == "src").unwrap();
        assert!(src.children.iter().any(|n| n.name == "main.rs"));

        let module = src.children.iter().find(|n| n.name == "module").unwrap();
        assert!(module.children.iter().any(|n| n.name == "lib.rs"));
    }
}
