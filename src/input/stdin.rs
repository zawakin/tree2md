use std::collections::{HashMap, HashSet};
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
    pub respect_gitignore: bool,
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

#[derive(Debug)]
pub struct StdinResult {
    pub canonical_paths: Vec<PathBuf>,
    pub original_map: HashMap<PathBuf, String>,
}

pub fn process_stdin_input(config: &StdinConfig) -> Result<StdinResult> {
    let raw_inputs = if config.null_delimited {
        read_null_delimited_strings()?
    } else {
        read_line_delimited_strings()?
    };

    process_stdin_input_from_raw(&raw_inputs, config)
}

pub fn process_stdin_input_from_raw(
    raw_inputs: &[String],
    config: &StdinConfig,
) -> Result<StdinResult> {
    let mut original_map = HashMap::new();

    let mut paths = Vec::new();
    let mut warnings = Vec::new();

    for raw_input in raw_inputs {
        let raw_path = PathBuf::from(raw_input.trim());
        if raw_path.as_os_str().is_empty() {
            continue;
        }
        let path = if raw_path.is_absolute() {
            raw_path
        } else {
            config.base_dir.join(&raw_path)
        };

        // Canonicalize path (resolve symlinks)
        let real_path = match fs::canonicalize(&path) {
            Ok(p) => {
                // Store original input for display-path input mode
                original_map.insert(p.clone(), raw_input.clone());
                p
            }
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
    let (files, dirs): (Vec<_>, Vec<_>) = paths.into_iter().partition(|p| !p.is_dir());

    let mut result = files;

    if !dirs.is_empty() {
        if config.expand_dirs {
            for dir in dirs {
                // Include the directory itself in the results first (stdin authoritative)
                // The directory was explicitly provided, so it should be in the output
                if config.respect_gitignore {
                    expand_directory_with_gitignore(&dir, &mut result)?;
                } else {
                    expand_directory(&dir, &mut result)?;
                }
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

    Ok(StdinResult {
        canonical_paths: result,
        original_map,
    })
}

fn read_line_delimited_strings() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    let mut inputs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            inputs.push(trimmed.to_string());
        }
    }

    Ok(inputs)
}

fn read_null_delimited_strings() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let inputs: Vec<String> = buffer
        .split(|&b| b == 0)
        .filter(|s| !s.is_empty())
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect();

    Ok(inputs)
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

fn expand_directory_with_gitignore(dir: &Path, result: &mut Vec<PathBuf>) -> io::Result<()> {
    use ignore::{gitignore::GitignoreBuilder, WalkBuilder};

    // Build .gitignore (return immediately if directory itself is ignored)
    let mut builder = if let Some(parent) = dir.parent() {
        GitignoreBuilder::new(parent)
    } else {
        GitignoreBuilder::new(dir)
    };
    // Collect .gitignore files from parent chain
    let mut cur = dir.to_path_buf();
    loop {
        let gi = cur.join(".gitignore");
        if gi.exists() {
            builder.add(gi);
        }
        if let Some(p) = cur.parent() {
            cur = p.to_path_buf();
        } else {
            break;
        }
    }
    // Global gitignore
    if let Some(home) = dirs::home_dir() {
        let global = home.join(".gitignore");
        if global.exists() {
            builder.add(global);
        }
    }
    let gi = builder.build().ok();
    if let Some(ref gi) = gi {
        if gi.matched(dir, true).is_ignore() {
            return Ok(()); // Directory itself is ignored
        }
    }

    // Walk: use filter_entry to prune ignored entries before descending
    let mut walker = WalkBuilder::new(dir);
    walker
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry({
            let gi = gi.clone();
            move |entry| {
                if let Some(ref gi) = gi {
                    let p = entry.path();
                    let is_dir = entry
                        .file_type()
                        .map(|t| t.is_dir())
                        .unwrap_or_else(|| p.is_dir());
                    if gi.matched(p, is_dir).is_ignore() {
                        return false; // Prune here
                    }
                }
                true
            }
        });
    let walker = walker.build();

    for entry in walker {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let is_dir = entry
                    .file_type()
                    .map(|t| t.is_dir())
                    .unwrap_or_else(|| path.is_dir());
                // Final check for safety
                if let Some(ref gi) = gi {
                    if gi.matched(path, is_dir).is_ignore() {
                        continue;
                    }
                }
                if !is_dir && path.is_file() {
                    result.push(path.to_path_buf());
                }
            }
            Err(_) => continue,
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
