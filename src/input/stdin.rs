use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct StdinConfig {
    pub base_dir: PathBuf,
    pub restrict_root: Option<PathBuf>,
    pub expand_dirs: bool,
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
    let raw_inputs = read_line_delimited_strings()?;

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

    // Remove duplicates while preserving input order
    result = dedup_preserving_order(result);

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

    // 1) Build gitignore with dir's parent as base to check if dir itself is ignored
    let base_for_check = dir.parent().unwrap_or(dir);
    let mut builder = GitignoreBuilder::new(base_for_check);

    // Collect .gitignore files from parent chain
    let mut cur = dir.to_path_buf();
    let mut git_root = None;
    loop {
        let gi = cur.join(".gitignore");
        if gi.exists() {
            builder.add(gi);
        }
        // Check if this is a git repository root
        if cur.join(".git").exists() && git_root.is_none() {
            git_root = Some(cur.clone());
        }
        if let Some(p) = cur.parent() {
            cur = p.to_path_buf();
        } else {
            break;
        }
    }
    // Add .git/info/exclude if in a git repository
    if let Some(root) = git_root {
        let exclude = root.join(".git/info/exclude");
        if exclude.exists() {
            builder.add(exclude);
        }
    }
    // Add global gitignore if exists
    if let Some(home) = dirs::home_dir() {
        let global = home.join(".gitignore");
        if global.exists() {
            builder.add(global);
        }
    }
    
    // Check if the directory itself is ignored
    if let Ok(gi) = builder.build() {
        if let Ok(rel) = dir.strip_prefix(base_for_check) {
            if gi.matched(rel, true).is_ignore() {
                // Directory itself is ignored, don't expand
                return Ok(());
            }
        }
    }
    
    // 2) Build gitignore again with dir as base for filtering contents
    let base = dir;
    let mut builder = GitignoreBuilder::new(base);

    // Collect .gitignore files from parent chain again
    let mut cur = base.to_path_buf();
    let mut git_root = None;
    loop {
        let gi = cur.join(".gitignore");
        if gi.exists() {
            builder.add(gi);
        }
        // Check if this is a git repository root
        if cur.join(".git").exists() && git_root.is_none() {
            git_root = Some(cur.clone());
        }
        if let Some(p) = cur.parent() {
            cur = p.to_path_buf();
        } else {
            break;
        }
    }
    // Add .git/info/exclude if in a git repository
    if let Some(root) = git_root {
        let exclude = root.join(".git/info/exclude");
        if exclude.exists() {
            builder.add(exclude);
        }
    }
    // Add global gitignore if exists
    if let Some(home) = dirs::home_dir() {
        let global = home.join(".gitignore");
        if global.exists() {
            builder.add(global);
        }
    }
    let gi = builder.build().ok();

    // 2) Use WalkBuilder with filter_entry for pre-pruning
    let mut walker = WalkBuilder::new(base);
    walker
        .hidden(false)
        .git_ignore(false)  // Disable WalkBuilder's gitignore, use our own
        .git_global(false)  // We handle global gitignore ourselves
        .git_exclude(false) // We handle git exclude ourselves
        .parents(false)     // We collect parent gitignores ourselves
        .filter_entry({
            let gi = gi.clone();
            let base = base.to_path_buf(); // Clone base for the closure
            move |entry| {
                if let Some(ref gi) = gi {
                    // Use relative path from base for matching
                    if let Ok(rel) = entry.path().strip_prefix(&base) {
                        let is_dir = entry.file_type()
                            .map(|t| t.is_dir())
                            .unwrap_or_else(|| entry.path().is_dir());
                        if gi.matched(rel, is_dir).is_ignore() {
                            return false; // Prune here before descending
                        }
                    }
                }
                true
            }
        });

    for entry in walker.build() {
        if let Ok(entry) = entry {
            let p = entry.path();
            let is_file = entry.file_type()
                .map(|t| t.is_file())
                .unwrap_or_else(|| p.is_file());
            if is_file {
                result.push(p.to_path_buf());
            }
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
