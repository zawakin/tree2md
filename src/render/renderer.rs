use crate::fs_tree::Node;
use crate::output::stats::Stats;
use crate::profile::{EmojiMapper, FileType};

/// Output format for the renderer
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum OutputFormat {
    /// Pipe output with plain tree characters
    Pipe,
    /// Terminal with Unicode tree branches
    Terminal,
}

/// Configuration for rendering
#[allow(dead_code)]
pub struct RenderConfig {
    pub format: OutputFormat,
    pub use_emoji: bool,
    pub use_colors: bool,
    pub show_stats: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Pipe,
            use_emoji: false,
            use_colors: false,
            show_stats: true,
        }
    }
}

/// Trait for rendering directory trees in different formats
pub trait Renderer {
    /// Render the tree structure
    fn render_tree(&mut self, root: &Node) -> String;

    /// Render statistics footer
    fn render_stats(&self, stats: &Stats) -> String;

    /// Check if this renderer supports animations
    #[allow(dead_code)]
    fn supports_animation(&self) -> bool {
        false
    }

    /// Check if this renderer supports colors
    #[allow(dead_code)]
    fn supports_colors(&self) -> bool {
        false
    }

    /// Get the output format
    #[allow(dead_code)]
    fn output_format(&self) -> OutputFormat;
}

/// Helper struct for managing node metadata during rendering
#[allow(dead_code)]
pub struct NodeMetadata {
    pub file_type: FileType,
    pub emoji: String,
    pub line_count: Option<usize>,
    pub size_bytes: u64,
}

impl NodeMetadata {
    #[allow(dead_code)]
    pub fn from_node(node: &Node, emoji_mapper: &EmojiMapper) -> Self {
        let file_type = if node.is_dir {
            FileType::Directory
        } else {
            FileType::classify_path(&node.path)
        };

        let emoji = emoji_mapper.get_emoji(&node.path, file_type);

        Self {
            file_type,
            emoji,
            line_count: None,
            size_bytes: 0,
        }
    }
}
