mod language;
mod stdin;
mod utils;

use clap::{Parser, ValueEnum};
use glob::Pattern;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use language::detect_lang;
use stdin::{process_stdin_input, StdinConfig};
use utils::{
    compile_patterns, generate_truncation_message, parse_ext_list, TruncateType, TruncationInfo,
};

const VERSION: &str = "0.4.0";

#[derive(Debug, Clone, ValueEnum)]
enum StdinMode {
    /// Use only files from stdin
    Authoritative,
    /// Merge stdin files with directory scan
    Merge,
}

#[derive(Debug)]
struct Node {
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<Node>,
}

#[derive(Parser)]
#[command(name = "tree2md")]
#[command(version = VERSION)]
#[command(about = "Scans directories and outputs their structure in Markdown format")]
struct Args {
    /// Include file contents (code blocks)
    #[arg(short = 'c', long = "contents")]
    contents: bool,

    /// Truncate file content to the first N bytes
    #[arg(short = 't', long = "truncate")]
    truncate: Option<usize>,

    /// Limit file content to the first N lines
    #[arg(long = "max-lines")]
    max_lines: Option<usize>,

    /// Comma-separated list of extensions to include (e.g., .go,.py)
    #[arg(short = 'e', long = "include-ext")]
    include_ext: Option<String>,

    /// Include hidden files and directories
    #[arg(short = 'a', long = "all")]
    all: bool,

    /// Respect .gitignore files
    #[arg(long = "respect-gitignore")]
    respect_gitignore: bool,

    /// Find files matching wildcard patterns (e.g., "*.rs", "src/**/*.go")
    /// Multiple patterns can be specified by using this option multiple times
    #[arg(short = 'f', long = "find")]
    find_patterns: Vec<String>,

    /// Read file paths from stdin (newline-delimited)
    #[arg(long = "stdin", conflicts_with = "stdin0")]
    stdin: bool,

    /// Read file paths from stdin (null-delimited)
    #[arg(long = "stdin0", conflicts_with = "stdin")]
    stdin0: bool,

    /// Stdin mode: 'authoritative' (default) or 'merge'
    #[arg(long = "stdin-mode", value_enum, default_value = "authoritative")]
    stdin_mode: StdinMode,

    /// Keep the input order from stdin (default: sort alphabetically)
    #[arg(long = "keep-order")]
    keep_order: bool,

    /// Base directory for resolving relative paths from stdin
    #[arg(long = "base", default_value = ".")]
    base: String,

    /// Restrict all paths to be within this directory
    #[arg(long = "restrict-root")]
    restrict_root: Option<String>,

    /// Expand directories found in stdin input
    #[arg(long = "expand-dirs")]
    expand_dirs: bool,

    /// Use flat output format instead of tree structure
    #[arg(long = "flat")]
    flat: bool,

    /// Directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    directory: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Handle stdin mode
    if args.stdin || args.stdin0 {
        return handle_stdin_mode(&args);
    }

    // Load gitignore if requested
    let gitignore = if args.respect_gitignore {
        load_gitignore(&args.directory)?
    } else {
        None
    };

    // Compile wildcard patterns
    let patterns = compile_patterns(&args.find_patterns)?;

    // Get the root path for pattern matching
    let root_path = Path::new(&args.directory)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(&args.directory).to_path_buf());

    // Build tree
    let root_node = build_tree(
        &args.directory,
        &args,
        gitignore.as_ref(),
        &patterns,
        &root_path,
    )?;

    // Filter by extensions if specified
    let mut root_node = root_node;
    if let Some(ref ext_str) = args.include_ext {
        let exts = parse_ext_list(ext_str);
        filter_by_extension(&mut root_node, &exts);
    }

    // Print tree structure
    println!("## File Structure");
    print_tree(&root_node, "");

    // Print code blocks if requested
    if args.contents {
        print_code_blocks(&root_node, &args);
    }

    Ok(())
}

fn handle_stdin_mode(args: &Args) -> io::Result<()> {
    let stdin_config = StdinConfig {
        null_delimited: args.stdin0,
        base_dir: PathBuf::from(&args.base),
        restrict_root: args.restrict_root.as_ref().map(PathBuf::from),
        expand_dirs: args.expand_dirs,
        keep_order: args.keep_order,
    };

    // Process stdin input
    let file_paths = match process_stdin_input(&stdin_config) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error: {}", e);
            match e {
                stdin::StdinError::RestrictRootViolation(_, _) => std::process::exit(2),
                stdin::StdinError::DirectoriesNotAllowed(_) => std::process::exit(3),
                stdin::StdinError::NoValidFiles => std::process::exit(4),
                _ => std::process::exit(1),
            }
        }
    };

    let mut all_paths = file_paths;

    // Handle merge mode
    if matches!(args.stdin_mode, StdinMode::Merge) {
        // Get files from directory scan
        let gitignore = if args.respect_gitignore {
            load_gitignore(&args.directory)?
        } else {
            None
        };

        let patterns = compile_patterns(&args.find_patterns)?;
        let root_path = Path::new(&args.directory)
            .canonicalize()
            .unwrap_or_else(|_| Path::new(&args.directory).to_path_buf());

        let dir_node = build_tree(
            &args.directory,
            args,
            gitignore.as_ref(),
            &patterns,
            &root_path,
        )?;

        // Collect files from directory tree
        let mut dir_files = Vec::new();
        collect_files_from_tree(&dir_node, &mut dir_files);

        // Merge with stdin files
        for file in dir_files {
            if !all_paths.contains(&file) {
                all_paths.push(file);
            }
        }

        // Sort if not keeping order
        if !args.keep_order {
            all_paths.sort();
        }
    }

    // Filter by extensions if specified
    if let Some(ref ext_str) = args.include_ext {
        let exts = parse_ext_list(ext_str);
        all_paths.retain(|path| {
            if let Some(ext) = path.extension() {
                let ext_str = format!(".{}", ext.to_string_lossy().to_lowercase());
                exts.contains(&ext_str)
            } else {
                false
            }
        });
    }

    // Generate output
    if args.flat {
        print_flat_structure(&all_paths, args);
    } else {
        print_tree_from_paths(&all_paths, args);
    }

    Ok(())
}

fn collect_files_from_tree(node: &Node, files: &mut Vec<PathBuf>) {
    if !node.is_dir {
        files.push(node.path.clone());
    }
    for child in &node.children {
        collect_files_from_tree(child, files);
    }
}

fn print_flat_structure(paths: &[PathBuf], args: &Args) {
    println!("## File Structure");
    println!();

    for path in paths {
        println!("- {}", path.display());
    }

    if args.contents {
        for path in paths {
            print_file_content(path, args);
        }
    }
}

fn print_tree_from_paths(paths: &[PathBuf], args: &Args) {
    // Find common ancestor
    let common_ancestor = stdin::find_common_ancestor(paths);

    // Build tree structure from paths
    let mut root = if let Some(ref ancestor) = common_ancestor {
        Node {
            name: ancestor
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("."))
                .to_string_lossy()
                .to_string(),
            path: ancestor.clone(),
            is_dir: true,
            children: Vec::new(),
        }
    } else {
        Node {
            name: ".".to_string(),
            path: PathBuf::from("."),
            is_dir: true,
            children: Vec::new(),
        }
    };

    // Build tree from paths
    for path in paths {
        insert_path_into_tree(&mut root, path, &common_ancestor);
    }

    // Print the tree
    println!("## File Structure");
    print_tree(&root, "");

    // Print code blocks if requested
    if args.contents {
        print_code_blocks(&root, args);
    }
}

fn insert_path_into_tree(root: &mut Node, path: &Path, common_ancestor: &Option<PathBuf>) {
    let relative = if let Some(ref ancestor) = common_ancestor {
        path.strip_prefix(ancestor).unwrap_or(path)
    } else {
        path
    };

    let components: Vec<_> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    if components.is_empty() {
        return;
    }

    // Navigate to the correct position in the tree
    let mut current_children = &mut root.children;

    for (i, name) in components.iter().enumerate() {
        let is_last = i == components.len() - 1;

        // Check if child already exists
        let child_pos = current_children
            .iter()
            .position(|child| &child.name == name);

        if let Some(pos) = child_pos {
            if !is_last {
                // Navigate deeper
                current_children = &mut current_children[pos].children;
            }
        } else {
            // Create new node
            let new_node = Node {
                name: name.clone(),
                path: if is_last {
                    path.to_path_buf()
                } else {
                    PathBuf::new()
                },
                is_dir: !is_last,
                children: Vec::new(),
            };

            current_children.push(new_node);

            if !is_last {
                // Navigate to the newly created node's children
                let new_pos = current_children.len() - 1;
                current_children = &mut current_children[new_pos].children;
            }
        }
    }
}

fn print_file_content(path: &Path, args: &Args) {
    let (content, truncation_info) =
        load_file_content_with_limits(path, args.truncate, args.max_lines);

    // Detect language
    let file_name = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_string_lossy();
    let lang = detect_lang(&file_name);
    let lang_name = lang.map(|l| l.name).unwrap_or("");

    // Print markdown code block
    println!("\n### {}", path.display());
    println!("```{}", lang_name);

    if let Some(l) = lang {
        println!("{}", l.to_comment(&path.display().to_string()));
    }

    print!("{}", content);

    // Ensure newline at end
    if !content.ends_with('\n') {
        println!();
    }

    if truncation_info.truncated {
        let message = generate_truncation_message(&truncation_info);
        if let Some(l) = lang {
            println!("{}", l.to_comment(&message));
        } else {
            println!("// {}", message);
        }
    }

    println!("```");
}

fn load_gitignore(dir: &str) -> io::Result<Option<Gitignore>> {
    let gitignore_path = Path::new(dir).join(".gitignore");
    if !gitignore_path.exists() {
        return Ok(None);
    }

    let mut builder = GitignoreBuilder::new(dir);
    builder.add(&gitignore_path);

    match builder.build() {
        Ok(gitignore) => Ok(Some(gitignore)),
        Err(_) => Ok(None),
    }
}

fn build_tree(
    path: &str,
    args: &Args,
    gitignore: Option<&Gitignore>,
    patterns: &[Pattern],
    root_path: &Path,
) -> io::Result<Node> {
    let path = Path::new(path);
    let metadata = fs::metadata(path)?;
    let name = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("."))
        .to_string_lossy()
        .to_string();

    let mut node = Node {
        name,
        path: path.to_path_buf(),
        is_dir: metadata.is_dir(),
        children: Vec::new(),
    };

    if metadata.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)?.filter_map(|e| e.ok()).collect();

        // Sort entries: directories first, then files, alphabetically within each group
        entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            let entry_path = entry.path();
            let entry_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files unless -a flag is set
            if !args.all && entry_name.starts_with('.') {
                continue;
            }

            // Check gitignore
            if let Some(gi) = gitignore {
                if gi.matched(&entry_path, entry_path.is_dir()).is_ignore() {
                    continue;
                }
            }

            // Skip if path cannot be converted to string (non-UTF8 paths)
            let entry_path_str = match entry_path.to_str() {
                Some(path) => path,
                None => {
                    eprintln!("Warning: Skipping non-UTF8 path: {:?}", entry_path);
                    continue;
                }
            };

            if let Ok(child_node) = build_tree(entry_path_str, args, gitignore, patterns, root_path)
            {
                // Skip if patterns are specified and node doesn't match
                if !patterns.is_empty() && !node_matches_patterns(&child_node, patterns, root_path)
                {
                    continue;
                }
                node.children.push(child_node);
            }
        }
    }

    Ok(node)
}

fn node_matches_patterns(node: &Node, patterns: &[Pattern], root_path: &Path) -> bool {
    if patterns.is_empty() {
        return true;
    }

    // Check if any pattern matches this node or its descendants
    if !node.is_dir {
        // For files, check if the relative path matches any pattern
        // Use absolute path if available, otherwise use the path as-is
        let check_path = node
            .path
            .canonicalize()
            .unwrap_or_else(|_| node.path.clone());

        if let Ok(relative_path) = check_path.strip_prefix(root_path) {
            // Convert path to use forward slashes for consistent pattern matching
            let path_str = relative_path.to_string_lossy().replace('\\', "/");
            for pattern in patterns {
                if pattern.matches(&path_str) {
                    return true;
                }
            }
        } else if let Ok(relative_path) = node.path.strip_prefix(root_path) {
            // Fallback: try stripping prefix from non-canonical path
            let path_str = relative_path.to_string_lossy().replace('\\', "/");
            for pattern in patterns {
                if pattern.matches(&path_str) {
                    return true;
                }
            }
        }
        return false;
    }

    // For directories, check if any child matches
    for child in &node.children {
        if node_matches_patterns(child, patterns, root_path) {
            return true;
        }
    }

    false
}

fn filter_by_extension(node: &mut Node, exts: &[String]) {
    if !node.is_dir {
        // Check if file has matching extension
        let path = Path::new(&node.name);
        if let Some(ext) = path.extension() {
            let ext_str = format!(".{}", ext.to_string_lossy().to_lowercase());
            if !exts.contains(&ext_str) {
                node.name.clear(); // Mark for removal
            }
        } else if !exts.is_empty() {
            node.name.clear(); // No extension, mark for removal
        }
        return;
    }

    // For directories, recursively filter children
    node.children.iter_mut().for_each(|child| {
        filter_by_extension(child, exts);
    });

    // Remove marked children
    node.children.retain(|child| !child.name.is_empty());
}

fn print_tree(node: &Node, indent: &str) {
    if indent.is_empty() {
        // Root directory
        println!("- {}/", node.name);
    }

    for (i, child) in node.children.iter().enumerate() {
        let _is_last = i == node.children.len() - 1;
        let bullet = "  - ";

        let name = if child.is_dir {
            format!("{}/", child.name)
        } else {
            child.name.clone()
        };

        println!("{}{}{}", indent, bullet, name);

        if child.is_dir {
            print_tree(child, &format!("{}    ", indent));
        }
    }
}

fn print_code_blocks(node: &Node, args: &Args) {
    if !node.is_dir {
        // Load file content with limits
        let (content, truncation_info) =
            load_file_content_with_limits(&node.path, args.truncate, args.max_lines);

        // Detect language
        let lang = detect_lang(&node.name);

        let lang_name = lang.map(|l| l.name).unwrap_or("");

        // Print markdown code block
        println!("\n### {}", node.path.display());
        println!("```{}", lang_name);

        if let Some(l) = lang {
            println!("{}", l.to_comment(&node.path.display().to_string()));
        }

        print!("{}", content);

        // Ensure newline at end
        if !content.ends_with('\n') {
            println!();
        }

        if truncation_info.truncated {
            let message = generate_truncation_message(&truncation_info);
            if let Some(l) = lang {
                println!("{}", l.to_comment(&message));
            } else {
                println!("// {}", message);
            }
        }

        println!("```");
    }

    for child in &node.children {
        print_code_blocks(child, args);
    }
}

fn load_file_content_with_limits(
    path: &Path,
    max_bytes: Option<usize>,
    max_lines: Option<usize>,
) -> (String, TruncationInfo) {
    let file_result = fs::File::open(path);

    if let Err(e) = file_result {
        return (
            format!("// Error reading file: {}\n", e),
            TruncationInfo {
                truncated: false,
                total_lines: 0,
                total_bytes: 0,
                shown_lines: 0,
                shown_bytes: 0,
                truncate_type: TruncateType::None,
            },
        );
    }

    let mut file = file_result.unwrap();
    let mut content = String::new();

    if let Err(e) = file.read_to_string(&mut content) {
        return (
            format!("// Error reading file: {}\n", e),
            TruncationInfo {
                truncated: false,
                total_lines: 0,
                total_bytes: 0,
                shown_lines: 0,
                shown_bytes: 0,
                truncate_type: TruncateType::None,
            },
        );
    }

    let total_bytes = content.len();
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let mut info = TruncationInfo {
        truncated: false,
        total_lines,
        total_bytes,
        shown_lines: total_lines,
        shown_bytes: total_bytes,
        truncate_type: TruncateType::None,
    };

    // No limits
    if max_bytes.is_none() && max_lines.is_none() {
        return (content, info);
    }

    let mut truncated_content = content.clone();
    let mut truncated_by_lines = false;
    let mut truncated_by_bytes = false;

    // Apply line limit
    if let Some(max_l) = max_lines {
        if total_lines > max_l {
            let limited_lines: Vec<&str> = lines.into_iter().take(max_l).collect();
            truncated_content = limited_lines.join("\n");
            truncated_by_lines = true;
        }
    }

    // Apply byte limit
    if let Some(max_b) = max_bytes {
        if truncated_content.len() > max_b {
            truncated_content.truncate(max_b);
            truncated_by_bytes = true;
        }
    }

    info.truncated = truncated_by_bytes || truncated_by_lines;
    info.shown_bytes = truncated_content.len();
    info.shown_lines = truncated_content.lines().count();

    info.truncate_type = match (truncated_by_bytes, truncated_by_lines) {
        (true, true) => TruncateType::Both,
        (true, false) => TruncateType::Bytes,
        (false, true) => TruncateType::Lines,
        _ => TruncateType::None,
    };

    (truncated_content, info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_lang() {
        assert_eq!(detect_lang("main.go").unwrap().name, "go");
        assert_eq!(detect_lang("script.py").unwrap().name, "python");
        assert_eq!(detect_lang("lib.rs").unwrap().name, "rust");
        assert_eq!(detect_lang("index.html").unwrap().name, "html");
        assert!(detect_lang("unknown.xyz").is_none());
    }

    #[test]
    fn test_build_tree() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create test structure
        fs::create_dir(temp_path.join("src"))?;
        fs::write(temp_path.join("src/main.rs"), "fn main() {}")?;
        fs::write(temp_path.join("Cargo.toml"), "[package]")?;
        fs::write(temp_path.join(".gitignore"), "target/")?;

        let args = Args {
            contents: false,
            truncate: None,
            max_lines: None,
            include_ext: None,
            all: false,
            respect_gitignore: false,
            find_patterns: vec![],
            stdin: false,
            stdin0: false,
            stdin_mode: StdinMode::Authoritative,
            keep_order: false,
            base: ".".to_string(),
            restrict_root: None,
            expand_dirs: false,
            flat: false,
            directory: temp_path.to_str().unwrap().to_string(),
        };

        let patterns = compile_patterns(&args.find_patterns)?;
        let root_path = temp_path.canonicalize()?;
        let node = build_tree(
            temp_path.to_str().unwrap(),
            &args,
            None,
            &patterns,
            &root_path,
        )?;

        assert!(node.is_dir);
        assert_eq!(node.children.len(), 2); // src and Cargo.toml (not .gitignore)

        Ok(())
    }

    #[test]
    fn test_filter_by_extension() {
        let mut root = Node {
            name: "root".to_string(),
            path: PathBuf::from("root"),
            is_dir: true,
            children: vec![
                Node {
                    name: "main.rs".to_string(),
                    path: PathBuf::from("main.rs"),
                    is_dir: false,
                    children: vec![],
                },
                Node {
                    name: "test.py".to_string(),
                    path: PathBuf::from("test.py"),
                    is_dir: false,
                    children: vec![],
                },
                Node {
                    name: "config.toml".to_string(),
                    path: PathBuf::from("config.toml"),
                    is_dir: false,
                    children: vec![],
                },
            ],
        };

        let exts = vec![".rs".to_string(), ".toml".to_string()];
        filter_by_extension(&mut root, &exts);

        assert_eq!(root.children.len(), 2);
        assert_eq!(root.children[0].name, "main.rs");
        assert_eq!(root.children[1].name, "config.toml");
    }
}
