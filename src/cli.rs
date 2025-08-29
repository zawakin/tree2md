use clap::{Parser, ValueEnum};

pub const VERSION: &str = "0.6.0";

#[derive(Debug, Clone, ValueEnum)]
pub enum DisplayPathMode {
    /// Display relative paths from display root
    Relative,
    /// Display absolute paths
    Absolute,
    /// Display paths as provided in stdin
    Input,
}

#[derive(Parser, Clone)]
#[command(name = "tree2md")]
#[command(version = VERSION)]
#[command(about = "Scans directories and outputs their structure in Markdown format")]
#[command(
    long_about = "Scans directories and outputs their structure in Markdown format.\n\nBy default, .gitignore files are respected for directory scans.\nUse --no-gitignore to include ignored files."
)]
pub struct Args {
    /// Include file contents (code blocks)
    #[arg(short = 'c', long = "contents")]
    pub contents: bool,

    /// Truncate file content to the first N bytes
    #[arg(short = 't', long = "truncate")]
    pub truncate: Option<usize>,

    /// Limit file content to the first N lines
    #[arg(long = "max-lines")]
    pub max_lines: Option<usize>,

    /// Comma-separated list of extensions to include (e.g., .go,.py)
    #[arg(short = 'e', long = "include-ext")]
    pub include_ext: Option<String>,

    /// Include hidden files and directories
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Force respect .gitignore files (default for directory scans)
    #[arg(
        long = "respect-gitignore",
        help = "Force respect .gitignore files in all modes"
    )]
    pub respect_gitignore: bool,

    /// Do not respect .gitignore files (overrides default behavior)
    #[arg(
        long = "no-gitignore",
        conflicts_with = "respect_gitignore",
        help = "Ignore .gitignore files and include all files"
    )]
    pub no_gitignore: bool,

    /// Find files matching wildcard patterns (e.g., "*.rs", "src/**/*.go")
    /// Multiple patterns can be specified by using this option multiple times
    #[arg(short = 'f', long = "find")]
    pub find_patterns: Vec<String>,

    /// Read file paths from stdin (newline-delimited)
    #[arg(long = "stdin")]
    pub stdin: bool,

    /// Base directory for resolving relative paths from stdin
    #[arg(long = "base", default_value = ".")]
    pub base: String,

    /// Restrict all paths to be within this directory
    #[arg(long = "restrict-root")]
    pub restrict_root: Option<String>,

    /// Expand directories found in stdin input
    #[arg(long = "expand-dirs")]
    pub expand_dirs: bool,

    /// Use flat output format instead of tree structure
    #[arg(long = "flat")]
    pub flat: bool,

    /// How to display paths: 'relative' (default), 'absolute', or 'input'
    #[arg(long = "display-path", value_enum, default_value = "relative")]
    pub display_path: DisplayPathMode,

    /// Root directory for relative path display (default: auto-detect)
    #[arg(long = "display-root")]
    pub display_root: Option<String>,

    /// Strip prefix from display paths (can be specified multiple times)
    #[arg(long = "strip-prefix")]
    pub strip_prefix: Vec<String>,

    /// Show the display root at the beginning of output
    #[arg(long = "show-root")]
    pub show_root: bool,

    /// Don't show root node in tree output (default for stdin mode)
    #[arg(long = "no-root")]
    pub no_root: bool,

    /// Custom label for root node (e.g., ".", "PROJECT_ROOT")
    #[arg(long = "root-label")]
    pub root_label: Option<String>,

    /// Directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    pub directory: String,
}

impl Args {
    /// Validate arguments for logical consistency
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        // Add validation logic here if needed
        Ok(())
    }

    /// Determine the effective gitignore setting based on flags and mode
    pub fn effective_gitignore(&self, is_stdin: bool) -> bool {
        if self.no_gitignore {
            // Explicit opt-out always wins
            false
        } else if self.respect_gitignore {
            // Explicit opt-in
            true
        } else if is_stdin {
            // For stdin mode, default is false (don't filter stdin-provided paths)
            false
        } else {
            // For directory scan mode, default is true (respect .gitignore)
            true
        }
    }
}
