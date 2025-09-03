use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub path: PathBuf,
    pub display_path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(name: String, path: PathBuf, is_dir: bool) -> Self {
        let display_path = path.clone();
        Self {
            name,
            path,
            display_path,
            is_dir,
            children: Vec::new(),
        }
    }

    pub fn with_display_path(mut self, display_path: PathBuf) -> Self {
        self.display_path = display_path;
        self
    }
}
