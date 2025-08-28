use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct StdinConfig {
    pub null_delimited: bool,
    pub base_dir: PathBuf,
    pub restrict_root: Option<PathBuf>,
    pub expand_dirs: bool,
    pub keep_order: bool,
}

#[derive(Debug)]
pub enum StdinError {
    RestrictRootViolation(PathBuf, PathBuf),
    DirectoriesNotAllowed(Vec<PathBuf>),
    NoValidFiles,
    IoError(io::Error),
}

impl std::fmt::Display for StdinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StdinError::RestrictRootViolation(path, root) => {
                write!(
                    f,
                    "Path '{}' is not within restrict-root directory '{}'",
                    path.display(),
                    root.display()
                )
            }
            StdinError::DirectoriesNotAllowed(dirs) => {
                writeln!(
                    f,
                    "Error: stdin contains directories but --expand-dirs was not specified:"
                )?;
                for dir in dirs {
                    writeln!(f, "  {}", dir.display())?;
                }
                write!(
                    f,
                    "Use --expand-dirs to expand directories or provide only file paths"
                )
            }
            StdinError::NoValidFiles => write!(f, "No valid files found in stdin input"),
            StdinError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for StdinError {}

impl From<io::Error> for StdinError {
    fn from(e: io::Error) -> Self {
        StdinError::IoError(e)
    }
}

pub type Result<T> = std::result::Result<T, StdinError>;

pub fn process_stdin_input(config: &StdinConfig) -> Result<Vec<PathBuf>> {
    let raw_paths = if config.null_delimited {
        read_null_delimited()?
    } else {
        read_line_delimited()?
    };

    let mut paths = Vec::new();
    let mut warnings = Vec::new();

    for raw_path in raw_paths {
        let path = if raw_path.is_absolute() {
            raw_path
        } else {
            config.base_dir.join(&raw_path)
        };

        // Canonicalize path (resolve symlinks)
        let real_path = match fs::canonicalize(&path) {
            Ok(p) => p,
            Err(_) => {
                warnings.push(format!("Warning: File not found: {}", path.display()));
                continue;
            }
        };

        // Check restrict-root constraint
        if let Some(ref restrict_root) = config.restrict_root {
            let root_real = fs::canonicalize(restrict_root)?;
            if !is_within(&real_path, &root_real)? {
                return Err(StdinError::RestrictRootViolation(real_path, root_real));
            }
        }

        paths.push(real_path);
    }

    // Print warnings
    for warning in warnings {
        eprintln!("{}", warning);
    }

    // Handle directories
    let (files, dirs): (Vec<_>, Vec<_>) = paths
        .into_iter()
        .partition(|p| !p.is_dir());

    let mut result = files;

    if !dirs.is_empty() {
        if config.expand_dirs {
            for dir in dirs {
                expand_directory(&dir, &mut result)?;
            }
        } else {
            return Err(StdinError::DirectoriesNotAllowed(dirs));
        }
    }

    // Remove duplicates while preserving order if needed
    result = if config.keep_order {
        dedup_preserving_order(result)
    } else {
        let mut sorted = result;
        sorted.sort();
        sorted.dedup();
        sorted
    };

    if result.is_empty() {
        return Err(StdinError::NoValidFiles);
    }

    Ok(result)
}

fn read_line_delimited() -> io::Result<Vec<PathBuf>> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    let mut paths = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            paths.push(PathBuf::from(trimmed));
        }
    }

    Ok(paths)
}

fn read_null_delimited() -> io::Result<Vec<PathBuf>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let paths: Vec<PathBuf> = buffer
        .split(|&b| b == 0)
        .filter(|s| !s.is_empty())
        .map(|bytes| {
            let s = String::from_utf8_lossy(bytes);
            PathBuf::from(s.into_owned())
        })
        .collect();

    Ok(paths)
}

fn is_within(path: &Path, root: &Path) -> io::Result<bool> {
    let path = path.canonicalize()?;
    let root = root.canonicalize()?;
    Ok(path.starts_with(&root))
}

fn expand_directory(dir: &Path, result: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            result.push(path);
        } else if path.is_dir() {
            expand_directory(&path, result)?;
        }
    }
    Ok(())
}

fn dedup_preserving_order(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for path in paths {
        if seen.insert(path.clone()) {
            result.push(path);
        }
    }

    result
}

pub fn find_common_ancestor(paths: &[PathBuf]) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }

    let mut common = paths[0].clone();

    for path in &paths[1..] {
        while !path.starts_with(&common) {
            if !common.pop() {
                return None;
            }
        }
    }

    // Make sure common is a directory
    if common.is_file() {
        common.pop();
    }

    Some(common)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_dedup_preserving_order() {
        let paths = vec![
            PathBuf::from("/a/b"),
            PathBuf::from("/c/d"),
            PathBuf::from("/a/b"),
            PathBuf::from("/e/f"),
            PathBuf::from("/c/d"),
        ];

        let result = dedup_preserving_order(paths);
        assert_eq!(
            result,
            vec![
                PathBuf::from("/a/b"),
                PathBuf::from("/c/d"),
                PathBuf::from("/e/f"),
            ]
        );
    }

    #[test]
    fn test_find_common_ancestor() {
        let paths = vec![
            PathBuf::from("/home/user/project/src/main.rs"),
            PathBuf::from("/home/user/project/tests/test.rs"),
            PathBuf::from("/home/user/project/README.md"),
        ];

        let ancestor = find_common_ancestor(&paths);
        assert_eq!(ancestor, Some(PathBuf::from("/home/user/project")));
    }

    #[test]
    fn test_find_common_ancestor_no_common() {
        let paths = vec![
            PathBuf::from("/home/user/project/src/main.rs"),
            PathBuf::from("/var/log/app.log"),
        ];

        let ancestor = find_common_ancestor(&paths);
        assert_eq!(ancestor, Some(PathBuf::from("/")));
    }

    #[test]
    fn test_is_within() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();
        let sub_dir = root.join("subdir");
        fs::create_dir(&sub_dir)?;

        assert!(is_within(&sub_dir, root)?);
        assert!(!is_within(root, &sub_dir)?);

        Ok(())
    }
}
