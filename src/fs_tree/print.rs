use super::node::Node;
use crate::cli::Args;
use crate::content::read::print_file_content_with_display;

pub fn print_code_blocks(node: &Node, args: &Args) {
    if !node.is_dir && !node.path.as_os_str().is_empty() {
        print_file_content_with_display(&node.path, &node.display_path, args);
    }

    for child in &node.children {
        print_code_blocks(child, args);
    }
}
