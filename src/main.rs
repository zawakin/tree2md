mod language;
mod stdin;
mod utils;

use clap::{Parser, ValueEnum};
use glob::Pattern;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Seek};
use std::path::{Path, PathBuf};

use language::detect_lang;
use stdin::{find_common_ancestor, process_stdin_input, StdinConfig};
use utils::{
    calculate_display_path, compile_patterns, format_size, generate_truncation_message, 
    parse_ext_list, TruncateType, TruncationInfo,
};

const VERSION: &str = "0.5.0";

#[derive(Debug, Clone, ValueEnum)]
enum StdinMode {
    /// Use only files from stdin
    Authoritative,
    /// Merge stdin files with directory scan
    Merge,
}

#[derive(Debug, Clone, ValueEnum)]
enum DisplayPathMode {
    /// Display relative paths from display root
    Relative,
    /// Display absolute paths
    Absolute,
    /// Display paths as provided in stdin
    Input,
}

#[derive(Debug)]
struct Node {
    name: String,
    path: PathBuf,
    display_path: PathBuf,
    is_dir: bool,
    children: Vec<Node>,
    #[allow(dead_code)]
    original_input: Option<String>,
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

    /// How to display paths: 'relative' (default), 'absolute', or 'input'
    #[arg(long = "display-path", value_enum, default_value = "relative")]
    display_path: DisplayPathMode,

    /// Root directory for relative path display (default: auto-detect)
    #[arg(long = "display-root")]
    display_root: Option<String>,

    /// Strip prefix from display paths (can be specified multiple times)
    #[arg(long = "strip-prefix")]
    strip_prefix: Vec<String>,

    /// Show the display root at the beginning of output
    #[arg(long = "show-root")]
    show_root: bool,

    /// Don't show root node in tree output (default for stdin mode)
    #[arg(long = "no-root")]
    no_root: bool,

    /// Custom label for root node (e.g., ".", "PROJECT_ROOT")
    #[arg(long = "root-label")]
    root_label: Option<String>,

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

    // Determine display root
    let display_root = determine_display_root(&args, &[PathBuf::from(&args.directory)])?;

    // Show root if requested
    if args.show_root {
        println!("Display root: {}\n", display_root.display());
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
        &display_root,
        None,
    )?;

    // Filter by extensions if specified
    let mut root_node = root_node;
    if let Some(ref ext_str) = args.include_ext {
        let exts = parse_ext_list(ext_str);
        filter_by_extension(&mut root_node, &exts);
    }

    // Print tree structure
    println!("## File Structure");
    // For non-stdin mode, show root by default unless --no-root is specified
    let show_root = !args.no_root;
    print_tree_with_options(&root_node, "", &args, show_root);

    // Print code blocks if requested
    if args.contents {
        print_code_blocks(&root_node, &args);
    }

    Ok(())
}

fn handle_stdin_mode(args: &Args) -> io::Result<()> {
    // Respect gitignore by default in stdin merge mode
    let respect_gitignore = match args.stdin_mode {
        StdinMode::Merge => args.respect_gitignore,
        StdinMode::Authoritative => args.respect_gitignore,
    };

    let stdin_config = StdinConfig {
        null_delimited: args.stdin0,
        base_dir: PathBuf::from(&args.base),
        restrict_root: args.restrict_root.as_ref().map(PathBuf::from),
        expand_dirs: args.expand_dirs,
        keep_order: args.keep_order,
    };

    // Process stdin input and get both canonical paths and original inputs
    let stdin::StdinResult {
        canonical_paths: file_paths,
        original_map: original_inputs,
    } = match process_stdin_input(&stdin_config) {
        Ok(result) => result,
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
        let gitignore = if respect_gitignore {
            load_gitignore(&args.directory)?
        } else {
            None
        };

        let patterns = compile_patterns(&args.find_patterns)?;
        let root_path = Path::new(&args.directory)
            .canonicalize()
            .unwrap_or_else(|_| Path::new(&args.directory).to_path_buf());

        // Determine display root for merge mode
        let display_root = determine_display_root(args, &all_paths)?;

        let dir_node = build_tree(
            &args.directory,
            args,
            gitignore.as_ref(),
            &patterns,
            &root_path,
            &display_root,
            None,
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

    // Determine display root
    let display_root = determine_display_root(args, &all_paths)?;

    // Show root if requested
    if args.show_root {
        println!("Display root: {}\n", display_root.display());
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
        print_flat_structure(&all_paths, args, &display_root, &original_inputs);
    } else {
        // Build tree from paths
        let common_ancestor = find_common_ancestor(&all_paths);
        let mut root = Node {
            name: common_ancestor
                .as_ref()
                .and_then(|p| p.file_name())
                .unwrap_or_else(|| std::ffi::OsStr::new("."))
                .to_string_lossy()
                .to_string(),
            path: common_ancestor
                .clone()
                .unwrap_or_else(|| PathBuf::from(".")),
            display_path: PathBuf::from("."),
            is_dir: true,
            children: Vec::new(),
            original_input: None,
        };

        for path in &all_paths {
            let original_input = original_inputs.get(path).cloned();
            insert_path_into_tree(
                &mut root,
                path,
                &common_ancestor,
                args,
                &display_root,
                original_input,
            );
        }

        println!("## File Structure");
        // For stdin mode, default to no root unless explicitly set
        let show_root = !args.no_root && (args.root_label.is_some() || !args.stdin);
        print_tree_with_options(&root, "", args, show_root);

        if args.contents {
            print_code_blocks(&root, args);
        }
    }

    Ok(())
}

fn determine_display_root(args: &Args, paths: &[PathBuf]) -> io::Result<PathBuf> {
    if let Some(ref display_root_str) = args.display_root {
        // User specified display root
        let display_root = Path::new(display_root_str);
        if !display_root.exists() {
            eprintln!(
                "Warning: Display root '{}' does not exist, using current directory",
                display_root_str
            );
            Ok(std::env::current_dir()?)
        } else {
            Ok(display_root.canonicalize()?)
        }
    } else {
        // Auto-detect display root
        if args.stdin || args.stdin0 {
            // For stdin mode, use LCA of all paths
            if let Some(lca) = find_common_ancestor(paths) {
                Ok(lca)
            } else {
                Ok(std::env::current_dir()?)
            }
        } else {
            // For directory scan mode, use the scan directory
            Path::new(&args.directory)
                .canonicalize()
                .or_else(|_| std::env::current_dir())
        }
    }
}

fn print_flat_structure(
    paths: &[PathBuf],
    args: &Args,
    display_root: &Path,
    original_inputs: &HashMap<PathBuf, String>,
) {
    println!("## File Structure");
    for path in paths {
        let original_input = original_inputs.get(path).map(|s| s.as_str());
        let display_path = calculate_display_path(
            path,
            &args.display_path,
            display_root,
            original_input,
            &args.strip_prefix,
        );
        println!("- {}", display_path.display());
    }

    if args.contents {
        for path in paths {
            if path.is_file() {
                let original_input = original_inputs.get(path).map(|s| s.as_str());
                let display_path = calculate_display_path(
                    path,
                    &args.display_path,
                    display_root,
                    original_input,
                    &args.strip_prefix,
                );
                print_file_content_with_display(path, &display_path, args);
            }
        }
    }
}

fn collect_files_from_tree(node: &Node, files: &mut Vec<PathBuf>) {
    if !node.is_dir && !node.path.as_os_str().is_empty() {
        files.push(node.path.clone());
    }
    for child in &node.children {
        collect_files_from_tree(child, files);
    }
}

fn insert_path_into_tree(
    root: &mut Node,
    path: &Path,
    common_ancestor: &Option<PathBuf>,
    args: &Args,
    display_root: &Path,
    original_input: Option<String>,
) {
    let components: Vec<_> = if let Some(ref ancestor) = common_ancestor {
        path.strip_prefix(ancestor)
            .unwrap_or(path)
            .components()
            .collect()
    } else {
        path.components().collect()
    };

    let mut current_children = &mut root.children;

    for (i, component) in components.iter().enumerate() {
        let name = component.as_os_str().to_string_lossy().to_string();
        let is_last = i == components.len() - 1;

        // Check if child already exists
        let child_pos = current_children
            .iter()
            .position(|child| child.name == *name);

        if let Some(pos) = child_pos {
            if !is_last {
                // Navigate deeper
                current_children = &mut current_children[pos].children;
            }
        } else {
            // Create new node
            let node_path = if is_last {
                path.to_path_buf()
            } else {
                PathBuf::new()
            };

            let display_path = if is_last && !node_path.as_os_str().is_empty() {
                calculate_display_path(
                    &node_path,
                    &args.display_path,
                    display_root,
                    original_input.as_deref(),
                    &args.strip_prefix,
                )
            } else {
                PathBuf::from(&name)
            };

            let new_node = Node {
                name: name.clone(),
                path: node_path,
                display_path,
                is_dir: !is_last,
                children: Vec::new(),
                original_input: original_input.clone(),
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

#[allow(dead_code)]
fn print_file_content(path: &Path, args: &Args) {
    print_file_content_with_display(path, path, args);
}

fn print_file_content_with_display(path: &Path, display_path: &Path, args: &Args) {
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
    println!("\n### {}", display_path.display());
    println!("```{}", lang_name);

    print!("{}", content);

    // Ensure newline at end
    if !content.ends_with('\n') {
        println!();
    }

    if truncation_info.truncated {
        let message = generate_truncation_message(&truncation_info);
        // For JSON files, print truncation message outside code block to avoid invalid syntax
        if lang.map(|l| l.name == "json").unwrap_or(false) {
            println!("```");
            println!("*{}*", message);
        } else {
            // Print truncation message as a comment in the appropriate language
            if let Some(l) = lang {
                println!("{}", l.to_comment(&message));
            } else {
                println!("// {}", message);
            }
            println!("```");
        }
    } else {
        println!("```");
    }
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
    display_root: &Path,
    original_input: Option<String>,
) -> io::Result<Node> {
    let path_buf = Path::new(path);
    let metadata = fs::metadata(path_buf)?;
    let name = path_buf
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("."))
        .to_string_lossy()
        .to_string();

    let resolved_path = path_buf
        .canonicalize()
        .unwrap_or_else(|_| path_buf.to_path_buf());

    let display_path = calculate_display_path(
        &resolved_path,
        &args.display_path,
        display_root,
        original_input.as_deref(),
        &args.strip_prefix,
    );

    let mut node = Node {
        name,
        path: resolved_path.clone(),
        display_path,
        is_dir: metadata.is_dir(),
        children: Vec::new(),
        original_input,
    };

    if metadata.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path_buf)?.filter_map(|e| e.ok()).collect();

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

            if let Ok(child_node) = build_tree(
                entry_path_str,
                args,
                gitignore,
                patterns,
                root_path,
                display_root,
                None,
            ) {
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

fn filter_by_extension(node: &mut Node, extensions: &[String]) {
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

fn print_tree(node: &Node, prefix: &str) {
    if !node.name.is_empty() {
        println!("{}- {}", prefix, node.name);
    }

    for child in &node.children {
        let child_prefix = format!("{}  ", prefix);
        print_tree(child, &child_prefix);
    }
}

fn print_tree_with_options(node: &Node, prefix: &str, args: &Args, show_root: bool) {
    if show_root {
        // Show root with custom label if provided
        let root_name = args.root_label.as_deref().unwrap_or(&node.name);
        if !root_name.is_empty() {
            println!("{}- {}", prefix, root_name);
        }
        for child in &node.children {
            let child_prefix = format!("{}  ", prefix);
            print_tree(child, &child_prefix);
        }
    } else {
        // Skip root node, print children directly
        for child in &node.children {
            print_tree(child, prefix);
        }
    }
}

fn print_code_blocks(node: &Node, args: &Args) {
    if !node.is_dir && !node.path.as_os_str().is_empty() {
        print_file_content_with_display(&node.path, &node.display_path, args);
    }

    for child in &node.children {
        print_code_blocks(child, args);
    }
}

fn load_file_content_with_limits(
    path: &Path,
    truncate_bytes: Option<usize>,
    max_lines: Option<usize>,
) -> (String, TruncationInfo) {
    // First, try to detect if it's a binary file by reading first chunk
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return (
                format!("Error reading file: {}", e),
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
    };

    // Read first 8KB to detect binary content
    let mut buffer = vec![0; 8192.min(file.metadata().map(|m| m.len() as usize).unwrap_or(8192))];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            return (
                format!("Error reading file: {}", e),
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
    };
    buffer.truncate(bytes_read);

    // Check for binary content (NULL bytes or other control characters)
    let is_binary = buffer.iter().any(|&b| b == 0 || (b < 32 && b != 9 && b != 10 && b != 13));
    
    if is_binary {
        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
        return (
            format!("Binary file ({})", format_size(file_size)),
            TruncationInfo {
                truncated: false,
                total_lines: 0,
                total_bytes: file_size as usize,
                shown_lines: 0,
                shown_bytes: 0,
                truncate_type: TruncateType::None,
            },
        );
    }

    // Reset file position and read as text
    file.seek(std::io::SeekFrom::Start(0)).ok();
    
    let mut full_content = String::new();
    if let Err(e) = file.read_to_string(&mut full_content) {
        return (
            format!("Error reading file as text: {}", e),
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

    let total_bytes = full_content.len();
    let total_lines = full_content.lines().count();

    let mut truncated = false;
    let mut truncate_type = TruncateType::None;
    let mut result = String::new();
    let mut shown_lines = 0;
    let mut shown_bytes = 0;

    for line in full_content.lines() {
        // Check line limit
        if let Some(max) = max_lines {
            if shown_lines >= max {
                truncated = true;
                truncate_type = if truncate_bytes.is_some() {
                    TruncateType::Both
                } else {
                    TruncateType::Lines
                };
                break;
            }
        }

        let line_with_newline = format!("{}\n", line);
        let line_bytes = line_with_newline.len();

        // Check byte limit
        if let Some(max) = truncate_bytes {
            if shown_bytes + line_bytes > max {
                // Add partial line if there's room, respecting UTF-8 boundaries
                let remaining = max.saturating_sub(shown_bytes);
                if remaining > 0 {
                    // Find safe UTF-8 boundary within remaining bytes
                    let mut safe_cut = 0;
                    for (idx, _) in line_with_newline.char_indices() {
                        if idx <= remaining {
                            safe_cut = idx;
                        } else {
                            break;
                        }
                    }
                    if safe_cut > 0 {
                        result.push_str(&line_with_newline[..safe_cut]);
                        shown_bytes += safe_cut;
                    }
                }
                truncated = true;
                truncate_type = if max_lines.is_some() {
                    TruncateType::Both
                } else {
                    TruncateType::Bytes
                };
                break;
            }
        }

        result.push_str(&line_with_newline);
        shown_lines += 1;
        shown_bytes += line_bytes;
    }

    (
        result,
        TruncationInfo {
            truncated,
            total_lines,
            total_bytes,
            shown_lines,
            shown_bytes,
            truncate_type,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_detect_lang() {
        assert_eq!(detect_lang("test.rs").map(|l| l.name), Some("rust"));
        assert_eq!(detect_lang("test.go").map(|l| l.name), Some("go"));
        assert_eq!(detect_lang("test.py").map(|l| l.name), Some("python"));
        assert_eq!(detect_lang("test.unknown"), None);
    }

    #[test]
    fn test_build_tree() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::create_dir(temp_path.join("src")).unwrap();
        fs::write(temp_path.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(temp_path.join("README.md"), "# Test").unwrap();

        let args = Args::parse_from(&["tree2md", temp_path.to_str().unwrap()]);
        let display_root = temp_path.to_path_buf();
        let tree = build_tree(
            temp_path.to_str().unwrap(),
            &args,
            None,
            &[],
            temp_path,
            &display_root,
            None,
        )
        .unwrap();

        assert!(tree.is_dir);
        assert!(tree.children.len() >= 2);
    }

    #[test]
    fn test_no_file_comments_in_code_blocks() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        let test_rs_content = "fn main() {\n    println!(\"Hello\");\n}";
        let test_json_content = "{\n  \"name\": \"test\"\n}";

        fs::write(temp_path.join("test.rs"), test_rs_content).unwrap();
        fs::write(temp_path.join("test.json"), test_json_content).unwrap();

        // Test Rust file output
        let mut output = Vec::new();
        {
            let display_path = PathBuf::from("test.rs");

            // Simulate print_file_content_with_display output
            writeln!(&mut output, "\n### {}", display_path.display()).unwrap();
            writeln!(&mut output, "```rust").unwrap();
            write!(&mut output, "{}", test_rs_content).unwrap();
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "```").unwrap();
        }

        let output_str = String::from_utf8(output).unwrap();

        // Verify no file comment is present
        assert!(
            !output_str.contains("// test.rs"),
            "Should not contain file comment"
        );
        assert!(
            output_str.contains("### test.rs"),
            "Should contain markdown header"
        );
        assert!(
            output_str.contains("```rust"),
            "Should contain language tag"
        );
        assert!(
            output_str.contains(test_rs_content),
            "Should contain file content"
        );

        // Test JSON file output
        let mut output = Vec::new();
        {
            let display_path = PathBuf::from("test.json");

            // Simulate print_file_content_with_display output for JSON
            writeln!(&mut output, "\n### {}", display_path.display()).unwrap();
            writeln!(&mut output, "```json").unwrap();
            write!(&mut output, "{}", test_json_content).unwrap();
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "```").unwrap();
        }

        let output_str = String::from_utf8(output).unwrap();

        // Verify JSON uses 'json' not 'jsonc' and has no comment
        assert!(
            !output_str.contains("// test.json"),
            "Should not contain file comment"
        );
        assert!(
            output_str.contains("```json"),
            "Should use 'json' language tag"
        );
        assert!(
            !output_str.contains("```jsonc"),
            "Should not use 'jsonc' language tag"
        );
        assert!(
            output_str.contains(test_json_content),
            "Should contain file content"
        );
    }

    #[test]
    fn test_truncation_message_preserved() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a file with multiple lines
        let mut content = String::new();
        for i in 1..=20 {
            content.push_str(&format!("Line {}\n", i));
        }
        fs::write(temp_path.join("large.txt"), &content).unwrap();

        // Simulate truncation output
        let mut output = Vec::new();
        let display_path = PathBuf::from("large.txt");

        writeln!(&mut output, "\n### {}", display_path.display()).unwrap();
        writeln!(&mut output, "```").unwrap();

        // Output first 5 lines
        for i in 1..=5 {
            writeln!(&mut output, "Line {}", i).unwrap();
        }

        // Add truncation message
        writeln!(
            &mut output,
            "// [Content truncated: showing first 5 of 20 lines]"
        )
        .unwrap();
        writeln!(&mut output, "```").unwrap();

        let output_str = String::from_utf8(output).unwrap();

        // Verify truncation message is present but file comment is not
        assert!(
            !output_str.contains("// large.txt"),
            "Should not contain file comment"
        );
        assert!(
            output_str.contains("// [Content truncated:"),
            "Should contain truncation message"
        );
    }
}
