use clap::Parser;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const VERSION: &str = "0.2.0";

#[derive(Debug)]
struct Node {
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<Node>,
}

#[derive(Debug)]
struct TruncationInfo {
    truncated: bool,
    total_lines: usize,
    total_bytes: usize,
    shown_lines: usize,
    shown_bytes: usize,
    truncate_type: TruncateType,
}

#[derive(Debug)]
enum TruncateType {
    None,
    Bytes,
    Lines,
    Both,
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

    /// Directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    directory: String,
}

struct Lang {
    ext: &'static str,
    name: &'static str,
    comment_fn: fn(&str) -> String,
}

impl Lang {
    fn to_comment(&self, s: &str) -> String {
        (self.comment_fn)(s)
    }
}

const LANGS: &[Lang] = &[
    Lang {
        ext: ".go",
        name: "go",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".py",
        name: "python",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".sh",
        name: "shell",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".js",
        name: "javascript",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".ts",
        name: "typescript",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".tsx",
        name: "tsx",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".html",
        name: "html",
        comment_fn: |s| format!("<!-- {} -->", s),
    },
    Lang {
        ext: ".css",
        name: "css",
        comment_fn: |s| format!("/* {} */", s),
    },
    Lang {
        ext: ".scss",
        name: "scss",
        comment_fn: |s| format!("/* {} */", s),
    },
    Lang {
        ext: ".sass",
        name: "sass",
        comment_fn: |s| format!("/* {} */", s),
    },
    Lang {
        ext: ".sql",
        name: "sql",
        comment_fn: |s| format!("-- {}", s),
    },
    Lang {
        ext: ".rs",
        name: "rust",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".toml",
        name: "toml",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".yaml",
        name: "yaml",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".yml",
        name: "yaml",
        comment_fn: |s| format!("# {}", s),
    },
    Lang {
        ext: ".json",
        name: "json",
        comment_fn: |s| format!("// {}", s),
    },
    Lang {
        ext: ".md",
        name: "markdown",
        comment_fn: |s| format!("<!-- {} -->", s),
    },
];

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Load gitignore if requested
    let gitignore = if args.respect_gitignore {
        load_gitignore(&args.directory)?
    } else {
        None
    };

    // Build tree
    let root_node = build_tree(&args.directory, &args, gitignore.as_ref())?;

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

fn build_tree(path: &str, args: &Args, gitignore: Option<&Gitignore>) -> io::Result<Node> {
    let path = Path::new(path);
    let metadata = fs::metadata(path)?;
    let name = path.file_name()
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
        let mut entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();
        
        // Sort entries for consistent output
        entries.sort_by_key(|e| e.file_name());

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

            if let Ok(child_node) = build_tree(
                entry_path.to_str().unwrap_or(""),
                args,
                gitignore
            ) {
                node.children.push(child_node);
            }
        }
    }

    Ok(node)
}

fn parse_ext_list(ext_string: &str) -> Vec<String> {
    ext_string
        .split(',')
        .map(|s| {
            let ext = s.trim().to_lowercase();
            if ext.starts_with('.') {
                ext
            } else {
                format!(".{}", ext)
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
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
        let (content, truncation_info) = load_file_content_with_limits(
            &node.path,
            args.truncate,
            args.max_lines,
        );

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

fn generate_truncation_message(info: &TruncationInfo) -> String {
    match info.truncate_type {
        TruncateType::Lines => {
            format!(
                "[Content truncated: showing first {} of {} lines]",
                info.shown_lines, info.total_lines
            )
        }
        TruncateType::Bytes => {
            format!(
                "[Content truncated: showing first {} of {} bytes]",
                info.shown_bytes, info.total_bytes
            )
        }
        TruncateType::Both => {
            format!(
                "[Content truncated: showing first {} of {} lines, {} of {} bytes]",
                info.shown_lines, info.total_lines, info.shown_bytes, info.total_bytes
            )
        }
        TruncateType::None => "[Content truncated]".to_string(),
    }
}

fn detect_lang(filename: &str) -> Option<&'static Lang> {
    let path = Path::new(filename);
    let ext = path.extension()?.to_str()?;
    let ext_with_dot = format!(".{}", ext.to_lowercase());
    
    LANGS.iter().find(|lang| lang.ext == ext_with_dot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_ext_list() {
        let exts = parse_ext_list("go,py,.rs");
        assert_eq!(exts, vec![".go", ".py", ".rs"]);
        
        let exts = parse_ext_list(".md, .txt, rs");
        assert_eq!(exts, vec![".md", ".txt", ".rs"]);
    }

    #[test]
    fn test_detect_lang() {
        assert_eq!(detect_lang("main.go").unwrap().name, "go");
        assert_eq!(detect_lang("script.py").unwrap().name, "python");
        assert_eq!(detect_lang("lib.rs").unwrap().name, "rust");
        assert_eq!(detect_lang("index.html").unwrap().name, "html");
        assert!(detect_lang("unknown.xyz").is_none());
    }

    #[test]
    fn test_generate_truncation_message() {
        let info = TruncationInfo {
            truncated: true,
            total_lines: 100,
            total_bytes: 5000,
            shown_lines: 50,
            shown_bytes: 2500,
            truncate_type: TruncateType::Lines,
        };
        assert_eq!(
            generate_truncation_message(&info),
            "[Content truncated: showing first 50 of 100 lines]"
        );

        let info = TruncationInfo {
            truncated: true,
            total_lines: 100,
            total_bytes: 5000,
            shown_lines: 50,
            shown_bytes: 2500,
            truncate_type: TruncateType::Both,
        };
        assert_eq!(
            generate_truncation_message(&info),
            "[Content truncated: showing first 50 of 100 lines, 2500 of 5000 bytes]"
        );
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
            directory: temp_path.to_str().unwrap().to_string(),
        };

        let node = build_tree(temp_path.to_str().unwrap(), &args, None)?;
        
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