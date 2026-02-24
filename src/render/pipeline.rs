use crate::fs_tree::{LocCounter, Node};
use crate::output::stats::Stats;
use crate::profile::{EmojiMapper, FileType};
use std::path::PathBuf;

/// Intermediate representation for a file
#[derive(Debug, Clone)]
pub struct IrFile {
    pub name: String,
    /// Actual filesystem path (for reading file contents)
    pub path: PathBuf,
    pub display_path: PathBuf,
    #[allow(dead_code)]
    pub file_type: FileType,
    pub emoji: String,
    pub loc: Option<usize>,
    #[allow(dead_code)]
    pub size_bytes: u64,
}

/// Intermediate representation for a directory
#[derive(Debug, Clone)]
pub struct IrDir {
    pub name: String,
    pub display_path: PathBuf,
    pub files: Vec<IrFile>,
    pub dirs: Vec<IrDir>,
}

/// Context for aggregation during IR building
pub struct AggregationContext<'a> {
    pub emoji_mapper: &'a EmojiMapper,
    pub stats: &'a mut Stats,
    pub loc_counter: &'a LocCounter,
}

/// Build the intermediate representation from the filesystem tree
pub fn build_ir(root: &Node, ctx: &mut AggregationContext) -> IrDir {
    build_ir_node(root, ctx)
}

fn build_ir_node(node: &Node, ctx: &mut AggregationContext) -> IrDir {
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    // Process children
    for child in &node.children {
        if child.is_dir {
            // Add directory to stats
            ctx.stats.add_directory();

            // Recursively build IR for subdirectory
            let ir_dir = build_ir_node(child, ctx);
            dirs.push(ir_dir);
        } else {
            // Classify file type
            let file_type = FileType::classify_path(&child.path);

            // Get emoji for file
            let emoji = ctx
                .emoji_mapper
                .get_emoji(&child.path, file_type)
                .to_string();

            // Add file to stats
            ctx.stats.add_file(file_type, emoji.clone(), &child.path);

            // Count lines of code if enabled
            let loc = if let Some(line_count) = ctx.loc_counter.count_lines(&child.path) {
                ctx.stats.add_loc(file_type, line_count);
                Some(line_count)
            } else {
                None
            };

            // Get file size
            let size_bytes = std::fs::metadata(&child.path)
                .ok()
                .map(|m| m.len())
                .unwrap_or(0);

            // Create IR file
            let ir_file = IrFile {
                name: child.name.clone(),
                path: child.path.clone(),
                display_path: child.display_path.clone(),
                file_type,
                emoji,
                loc,
                size_bytes,
            };

            files.push(ir_file);
        }
    }

    // Create IR directory
    // Note: We don't increment directory stats here for the root itself,
    // as it will be handled by the parent or caller
    IrDir {
        name: node.name.clone(),
        display_path: node.display_path.clone(),
        files,
        dirs,
    }
}

/// Extension methods for IR nodes to simplify rendering
impl IrDir {
    /// Get total count of immediate children (files and directories)
    #[allow(dead_code)]
    pub fn immediate_child_count(&self) -> (usize, usize) {
        (self.files.len(), self.dirs.len())
    }

    /// Check if directory is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.dirs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::LocMode;
    use std::path::PathBuf;

    fn create_test_node() -> Node {
        Node {
            name: "root".to_string(),
            path: PathBuf::from("root"),
            is_dir: true,
            display_path: PathBuf::from("."),
            children: vec![
                Node {
                    name: "src".to_string(),
                    path: PathBuf::from("root/src"),
                    is_dir: true,
                    display_path: PathBuf::from("src"),
                    children: vec![Node {
                        name: "main.rs".to_string(),
                        path: PathBuf::from("root/src/main.rs"),
                        is_dir: false,
                        display_path: PathBuf::from("src/main.rs"),
                        children: vec![],
                    }],
                },
                Node {
                    name: "README.md".to_string(),
                    path: PathBuf::from("root/README.md"),
                    is_dir: false,
                    display_path: PathBuf::from("README.md"),
                    children: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_build_ir_basic() {
        let root = create_test_node();
        let emoji_mapper = EmojiMapper::new(false);
        let mut stats = Stats::new();
        let loc_counter = LocCounter::new(LocMode::Off);

        let mut ctx = AggregationContext {
            emoji_mapper: &emoji_mapper,
            stats: &mut stats,
            loc_counter: &loc_counter,
        };

        let ir = build_ir(&root, &mut ctx);

        assert_eq!(ir.name, "root");
        assert_eq!(ir.dirs.len(), 1);
        assert_eq!(ir.files.len(), 1);

        let src_dir = &ir.dirs[0];
        assert_eq!(src_dir.name, "src");
        assert_eq!(src_dir.files.len(), 1);
        assert_eq!(src_dir.files[0].name, "main.rs");

        assert_eq!(ir.files[0].name, "README.md");
    }

    #[test]
    fn test_ir_dir_methods() {
        let ir_dir = IrDir {
            name: "test".to_string(),
            display_path: PathBuf::from("test"),
            files: vec![
                IrFile {
                    name: "file1.txt".to_string(),
                    path: PathBuf::from("test/file1.txt"),
                    display_path: PathBuf::from("test/file1.txt"),
                    file_type: FileType::Text,
                    emoji: String::new(),
                    loc: None,
                    size_bytes: 0,
                },
                IrFile {
                    name: "file2.txt".to_string(),
                    path: PathBuf::from("test/file2.txt"),
                    display_path: PathBuf::from("test/file2.txt"),
                    file_type: FileType::Text,
                    emoji: String::new(),
                    loc: None,
                    size_bytes: 0,
                },
            ],
            dirs: vec![IrDir {
                name: "subdir".to_string(),
                display_path: PathBuf::from("test/subdir"),
                files: vec![],
                dirs: vec![],
            }],
        };

        assert_eq!(ir_dir.immediate_child_count(), (2, 1));
        assert!(!ir_dir.is_empty());

        let empty_dir = IrDir {
            name: "empty".to_string(),
            display_path: PathBuf::from("empty"),
            files: vec![],
            dirs: vec![],
        };

        assert!(empty_dir.is_empty());
    }
}
