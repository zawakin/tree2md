use super::node::Node;
use crate::cli::Args;
use crate::domain::filter::{MatchSpec, MatcherEngine, RelPath, Selection};
use crate::util::path::calculate_display_path;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;

/// Build tree using WalkBuilder for unified gitignore support with MatcherEngine
pub fn build_tree(
    path: &str,
    args: &Args,
    root_path: &Path,
    display_root: &Path,
) -> io::Result<Node> {
    // Create MatchSpec from CLI arguments
    let spec = MatchSpec::from_args(args, Path::new(path))?;
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
        let matcher = Arc::new(MatcherEngine::compile(spec, root_path)?);

        // Root that RelPaths for matching are taken against.
        //
        // When following links we strip the *logical* walk root (`path_buf`)
        // rather than the canonical `root_path`. `from_root_rel` falls back
        // to canonicalizing both sides, which resolves a symlink to its
        // target — and a target outside the scanned root would then fail the
        // prefix check and be dropped. Stripping the logical path keeps the
        // entry under its link path, so out-of-root targets are surfaced.
        let match_root = if args.follow_links {
            path_buf.to_path_buf()
        } else {
            root_path.to_path_buf()
        };

        let nested_repo_pruned = Arc::new(AtomicBool::new(false));
        let walker = new_walker(
            path_buf,
            args,
            Arc::clone(&matcher),
            match_root.clone(),
            Arc::clone(&nested_repo_pruned),
        );

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

            // Skip symlinks unless --follow-links is set. With follow_links(true)
            // the walker resolves links and reports the target's type, so this
            // guard only fires for the default (non-following) behavior.
            if !args.follow_links && entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
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

            // Create RelPath for matching.
            let rel_path = match RelPath::from_root_rel(entry_path, &match_root) {
                Some(rp) => rp,
                None => continue,
            };

            // Directories reaching this point were not pruned by the walker's
            // filter (see `new_walker`), so only files still need a decision.
            if entry_metadata.is_dir() {
                // The walk is sequential and yields a directory before reading
                // its contents, so loading its `.gitignore` here makes the
                // layer visible to the filter for every child.
                matcher.load_nested_gitignore(&rel_path, entry_path)?;
            } else if matcher.select_file(&rel_path) != Selection::Include {
                continue;
            }

            let entry_name = entry_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("."))
                .to_string_lossy()
                .to_string();

            // The node's stored path is canonicalized so content/LOC reads
            // resolve the real (symlink target) file.
            let resolved_entry_path = entry_path
                .canonicalize()
                .unwrap_or_else(|_| entry_path.to_path_buf());

            // Display path: when following links, show the logical link path
            // (e.g. `link_dir/inside.txt`) instead of the canonicalized target,
            // which would otherwise collapse the link onto its real location.
            let entry_display_path = if args.follow_links {
                entry_path
                    .strip_prefix(path_buf)
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|_| calculate_display_path(&resolved_entry_path, display_root))
            } else {
                calculate_display_path(&resolved_entry_path, display_root)
            };

            let node = Node::new(entry_name, resolved_entry_path, entry_metadata.is_dir())
                .with_display_path(entry_display_path);

            nodes_map.insert(entry_path.to_path_buf(), node);
        }

        // Build the tree structure from the flat map
        build_tree_from_map(&mut root_node, nodes_map, path_buf);

        // Remove directories left empty after pruning (include filtering,
        // nested-repo detection, etc.). Not run unconditionally because
        // empty dirs at --level boundary should remain visible.
        if spec.has_includes() || nested_repo_pruned.load(AtomicOrdering::Relaxed) {
            remove_empty_directories(&mut root_node);
        }
    }

    Ok(root_node)
}

/// Configure the directory walker.
///
/// Directory pruning runs inside the walker via `filter_entry`: a rejected
/// directory is neither yielded nor read, so excluded subtrees (`target/`,
/// `node_modules/`, `.git/`, anything outside `-I`, ...) cost nothing. Files
/// always pass the filter and are decided by the caller with `select_file`.
fn new_walker(
    path: &Path,
    args: &Args,
    matcher: Arc<MatcherEngine>,
    match_root: PathBuf,
    nested_repo_pruned: Arc<AtomicBool>,
) -> WalkBuilder {
    let mut walker = WalkBuilder::new(path);
    walker
        .hidden(false) // Don't exclude hidden files by default (will be handled by patterns)
        .git_ignore(false) // We handle gitignore in MatcherEngine
        .git_global(false)
        .git_exclude(false)
        .parents(false)
        .ignore(false)
        .follow_links(args.follow_links) // Opt-in via --follow-links; cycles are detected by `ignore`
        .max_depth(args.level) // Use level directly
        .filter_entry(move |entry| {
            // The walk root (depth 0) never reaches this filter.
            let is_dir = entry.file_type().is_some_and(|ft| ft.is_dir());
            // Non-UTF8 paths pass through so the caller can warn and skip them.
            if !is_dir || entry.path().to_str().is_none() {
                return true;
            }

            // Prune nested git repositories / worktrees / submodules.
            // If a subdirectory contains a `.git` entry (file or directory),
            // it represents a separate repository boundary and should not be
            // traversed. This prevents worktrees, submodules, and nested repos
            // from leaking into the output.
            if entry.path().join(".git").exists() {
                nested_repo_pruned.store(true, AtomicOrdering::Relaxed);
                return false;
            }

            match RelPath::from_root_rel(entry.path(), &match_root) {
                Some(rel_path) => matcher.select_dir(&rel_path) != Selection::PruneDir,
                None => true,
            }
        });
    walker
}

type ChildrenIndex = HashMap<PathBuf, Vec<(PathBuf, Node)>>;

fn build_tree_from_map(parent: &mut Node, nodes_map: HashMap<PathBuf, Node>, base_path: &Path) {
    // Group nodes by parent path once, so each directory's children are a
    // single lookup instead of a scan over every collected node (which made
    // assembly O(dirs × nodes)).
    let mut children_index: ChildrenIndex = HashMap::new();
    for (path, node) in nodes_map {
        if let Some(parent_path) = path.parent() {
            children_index
                .entry(parent_path.to_path_buf())
                .or_default()
                .push((path, node));
        }
    }

    attach_children(parent, &mut children_index, base_path);
}

fn attach_children(parent: &mut Node, children_index: &mut ChildrenIndex, base_path: &Path) {
    let Some(mut direct_children) = children_index.remove(base_path) else {
        return;
    };

    // Sort children: directories first, then files, alphabetically within each group
    direct_children.sort_by(|(_, a), (_, b)| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    // Add children to parent and recursively build their subtrees
    for (child_path, mut child) in direct_children {
        if child.is_dir {
            attach_children(&mut child, children_index, &child_path);
        }
        parent.children.push(child);
    }
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
    use crate::domain::filter::MatchSpec;
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
        let spec = MatchSpec::new().with_include_glob(vec!["*.rs".to_string()]);

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
    fn test_walker_does_not_descend_into_pruned_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        // Pruned by -I (no include relates to it)
        fs::create_dir_all(root.join("vendor/lib")).unwrap();
        fs::write(root.join("vendor/lib/a.txt"), "a").unwrap();
        // Pruned as a nested repository
        fs::create_dir_all(root.join("nested/.git")).unwrap();
        fs::write(root.join("nested/b.txt"), "b").unwrap();
        // .git is never traversed
        fs::create_dir_all(root.join(".git/objects")).unwrap();
        fs::write(root.join(".git/objects/obj"), "").unwrap();

        let args = Args::parse_from(&["tree2md", root.to_str().unwrap()]);
        let spec = MatchSpec::new().with_include_glob(vec!["src/**".to_string()]);
        let matcher = Arc::new(MatcherEngine::compile(&spec, root).unwrap());
        let nested_repo_pruned = Arc::new(AtomicBool::new(false));

        let walked: Vec<PathBuf> = new_walker(
            root,
            &args,
            matcher,
            root.to_path_buf(),
            Arc::clone(&nested_repo_pruned),
        )
        .build()
        .filter_map(Result::ok)
        .map(|e| e.path().strip_prefix(root).unwrap().to_path_buf())
        .collect();

        assert!(walked.contains(&PathBuf::from("src/main.rs")));
        for pruned in ["vendor", "nested", ".git"] {
            assert!(
                !walked.iter().any(|p| p.starts_with(pruned)),
                "walker entered {}: {:?}",
                pruned,
                walked
            );
        }
        assert!(nested_repo_pruned.load(AtomicOrdering::Relaxed));
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
