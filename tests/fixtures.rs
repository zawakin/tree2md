use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// Run tree2md with given arguments and return (stdout, stderr, success)
pub fn run_tree2md<I, S>(args: I) -> (String, String, bool)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::cargo_bin("tree2md").expect("tree2md binary not found");
    cmd.args(args);

    let Output {
        status,
        stdout,
        stderr,
    } = cmd.output().expect("Failed to execute tree2md");
    let stdout = String::from_utf8_lossy(&stdout).to_string();
    let stderr = String::from_utf8_lossy(&stderr).to_string();

    (stdout, stderr, status.success())
}

/// Helper to convert path to string
pub fn p<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().to_string()
}

/// A flexible fixture builder for creating directory structures
pub struct FixtureBuilder {
    temp_dir: TempDir,
    root_path: PathBuf,
}

impl FixtureBuilder {
    /// Create a new fixture builder
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        Self {
            temp_dir,
            root_path,
        }
    }

    /// Add a file with content
    pub fn file<P: AsRef<Path>, S: AsRef<str>>(self, path: P, content: S) -> Self {
        let full_path = self.root_path.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(full_path, content.as_ref()).expect("write file");
        self
    }

    /// Add an empty file
    pub fn touch<P: AsRef<Path>>(self, path: P) -> Self {
        self.file(path, "")
    }

    /// Create a directory
    pub fn dir<P: AsRef<Path>>(self, path: P) -> Self {
        let full_path = self.root_path.join(path);
        fs::create_dir_all(full_path).expect("create dir");
        self
    }

    /// Add files with generated content
    pub fn files_with<P, I, F>(self, paths: I, content_fn: F) -> Self
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
        F: Fn(&Path) -> String,
    {
        for path in paths {
            let path_ref = path.as_ref();
            let content = content_fn(path_ref);
            let full_path = self.root_path.join(path_ref);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).expect("create parent dirs");
            }
            fs::write(full_path, content).expect("write file");
        }
        self
    }

    /// Build the fixture and return (TempDir, root_path)
    pub fn build(self) -> (TempDir, PathBuf) {
        (self.temp_dir, self.root_path)
    }
}

/// DSL for defining directory structures declaratively
#[derive(Debug)]
pub enum FsEntry {
    File { name: String, content: String },
    Dir { name: String, entries: Vec<FsEntry> },
}

impl FsEntry {
    /// Create a file entry
    pub fn file<S1: Into<String>, S2: Into<String>>(name: S1, content: S2) -> Self {
        FsEntry::File {
            name: name.into(),
            content: content.into(),
        }
    }

    /// Create a directory entry
    pub fn dir<S: Into<String>>(name: S, entries: Vec<FsEntry>) -> Self {
        FsEntry::Dir {
            name: name.into(),
            entries,
        }
    }

    /// Create an empty file
    pub fn touch<S: Into<String>>(name: S) -> Self {
        Self::file(name, "")
    }

    /// Build this entry structure in the given directory
    pub fn build_in(&self, parent: &Path) -> std::io::Result<()> {
        match self {
            FsEntry::File { name, content } => {
                let path = parent.join(name);
                fs::write(path, content)?;
            }
            FsEntry::Dir { name, entries } => {
                let dir_path = parent.join(name);
                fs::create_dir_all(&dir_path)?;
                for entry in entries {
                    entry.build_in(&dir_path)?;
                }
            }
        }
        Ok(())
    }
}

/// Create a fixture from a declarative structure
pub fn create_fixture(entries: Vec<FsEntry>) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let root = temp_dir.path().to_path_buf();

    for entry in entries {
        entry.build_in(&root).expect("build entry");
    }

    (temp_dir, root)
}

/// Helper to create a project with many files of specific type
pub fn create_many_files(extension: &str, count: usize) -> (TempDir, PathBuf) {
    let builder = FixtureBuilder::new();
    let paths: Vec<String> = (1..=count)
        .map(|i| format!("file{}.{}", i, extension))
        .collect();

    builder
        .files_with(paths, |p| format!("// Content of {}\n", p.display()))
        .build()
}

/// Helper to create deeply nested structure
pub fn create_nested(depth: usize, files_per_level: usize) -> (TempDir, PathBuf) {
    let builder = FixtureBuilder::new();

    fn create_level(
        builder: FixtureBuilder,
        current_depth: usize,
        max_depth: usize,
        files_per_level: usize,
        path_prefix: String,
    ) -> FixtureBuilder {
        if current_depth > max_depth {
            return builder;
        }

        let mut b = builder;

        // Add files at this level
        for i in 1..=files_per_level {
            let file_path = if path_prefix.is_empty() {
                format!("file_{}.txt", i)
            } else {
                format!("{}/file_{}.txt", path_prefix, i)
            };
            b = b.file(&file_path, format!("Level {} File {}\n", current_depth, i));
        }

        // Add subdirectory and recurse
        if current_depth < max_depth {
            let new_prefix = if path_prefix.is_empty() {
                format!("level_{}", current_depth + 1)
            } else {
                format!("{}/level_{}", path_prefix, current_depth + 1)
            };
            b = b.dir(&new_prefix);
            b = create_level(b, current_depth + 1, max_depth, files_per_level, new_prefix);
        }

        b
    }

    create_level(builder, 0, depth, files_per_level, String::new()).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_builder() {
        let (_dir, root) = FixtureBuilder::new()
            .file("README.md", "# Test")
            .file("src/main.rs", "fn main() {}")
            .dir("empty_dir")
            .touch("empty.txt")
            .build();

        assert!(root.join("README.md").exists());
        assert!(root.join("src/main.rs").exists());
        assert!(root.join("empty_dir").is_dir());
        assert!(root.join("empty.txt").exists());
    }

    #[test]
    fn test_declarative_fixture() {
        let (_dir, root) = create_fixture(vec![
            FsEntry::file("README.md", "# Test"),
            FsEntry::dir(
                "src",
                vec![
                    FsEntry::file("main.rs", "fn main() {}"),
                    FsEntry::file("lib.rs", "pub fn lib() {}"),
                ],
            ),
            FsEntry::dir(
                "tests",
                vec![FsEntry::touch("test1.rs"), FsEntry::touch("test2.rs")],
            ),
        ]);

        assert!(root.join("README.md").exists());
        assert!(root.join("src/main.rs").exists());
        assert!(root.join("src/lib.rs").exists());
        assert!(root.join("tests/test1.rs").exists());
    }

    #[test]
    fn test_many_files() {
        let (_dir, root) = create_many_files("rs", 5);

        for i in 1..=5 {
            assert!(root.join(format!("file{}.rs", i)).exists());
        }
    }

    #[test]
    fn test_nested_structure() {
        let (_dir, root) = create_nested(3, 2);

        assert!(root.join("file_1.txt").exists());
        assert!(root.join("level_1/file_1.txt").exists());
        assert!(root.join("level_1/level_2/file_1.txt").exists());
        assert!(root.join("level_1/level_2/level_3/file_1.txt").exists());
    }
}
