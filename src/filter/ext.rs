use crate::fs_tree::Node;

/// Parse a comma-separated list of file extensions
pub fn parse_ext_list(ext_string: &str) -> Vec<String> {
    ext_string
        .split(',')
        .map(|s| {
            let ext = s.trim().to_lowercase();
            if ext.starts_with('.') {
                ext
            } else {
                format!(".{}", ext)
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn filter_by_extension(node: &mut Node, extensions: &[String]) {
    if !node.is_dir {
        return; // Files are filtered at a higher level
    }

    node.children.retain(|child| {
        if child.is_dir {
            true // Keep directories to check their contents
        } else {
            child.path.extension().is_some_and(|ext| {
                let ext_str = format!(".{}", ext.to_string_lossy().to_lowercase());
                extensions.contains(&ext_str)
            })
        }
    });

    // Recursively filter children
    for child in &mut node.children {
        filter_by_extension(child, extensions);
    }

    // Remove empty directories
    node.children
        .retain(|child| !child.is_dir || !child.children.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_ext_list() {
        let exts = parse_ext_list("go,py,.rs");
        assert_eq!(exts, vec![".go", ".py", ".rs"]);

        let exts = parse_ext_list(".md, .txt, rs");
        assert_eq!(exts, vec![".md", ".txt", ".rs"]);
    }

    #[test]
    fn test_filter_by_extension() {
        let mut node = Node {
            name: "root".to_string(),
            path: PathBuf::from("/root"),
            display_path: PathBuf::from("root"),
            is_dir: true,
            children: vec![
                Node {
                    name: "file1.rs".to_string(),
                    path: PathBuf::from("/root/file1.rs"),
                    display_path: PathBuf::from("file1.rs"),
                    is_dir: false,
                    children: vec![],
                    original_input: None,
                },
                Node {
                    name: "file2.go".to_string(),
                    path: PathBuf::from("/root/file2.go"),
                    display_path: PathBuf::from("file2.go"),
                    is_dir: false,
                    children: vec![],
                    original_input: None,
                },
                Node {
                    name: "file3.py".to_string(),
                    path: PathBuf::from("/root/file3.py"),
                    display_path: PathBuf::from("file3.py"),
                    is_dir: false,
                    children: vec![],
                    original_input: None,
                },
            ],
            original_input: None,
        };

        filter_by_extension(&mut node, &vec![".rs".to_string(), ".go".to_string()]);

        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].name, "file1.rs");
        assert_eq!(node.children[1].name, "file2.go");
    }
}
