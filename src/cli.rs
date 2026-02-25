use clap::{Parser, ValueEnum};

pub const VERSION: &str = "0.9.2";

#[derive(Debug, Clone, ValueEnum)]
pub enum UseGitignoreMode {
    /// Use .gitignore if in a git repository
    Auto,
    /// Never use .gitignore
    Never,
    /// Always use .gitignore
    Always,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum FunMode {
    /// Auto-detect based on terminal
    Auto,
    /// Enable fun features (animations, emojis)
    On,
    /// Disable fun features
    Off,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum StatsMode {
    /// No statistics
    Off,
    /// Minimal statistics
    Min,
    /// Full statistics with progress bars
    Full,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum LocMode {
    /// Don't count lines of code
    Off,
    /// Fast line counting
    Fast,
    /// Accurate line counting
    Accurate,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum ContentsMode {
    /// Keep the first N characters per file (line-boundary cut)
    Head,
    /// Keep low-indentation lines, collapse deeply-indented blocks
    Nest,
}

#[derive(Parser, Clone)]
#[command(name = "tree2md")]
#[command(version = VERSION)]
#[command(about = "Like the tree command, but optimized for AI agents")]
#[command(
    long_about = r#"tree2md — Visualize your codebase structure for humans and AI agents.

QUICK START:
  tree2md                          # Pretty tree in terminal (TTY)
  tree2md | pbcopy                 # Pipe-friendly tree for clipboard
  tree2md -c -I "*.rs" -L 2       # Tree + file contents for AI context
  tree2md -c --max-chars 30000    # Fit contents within token budget

OUTPUT MODES (auto-detected):
  TTY    Pretty output with emoji, LOC bars, stats, tree characters
  Pipe   Simple tree + line counts — ideal for pbcopy or LLM pipes
  -c     Tree + file contents (code-fenced) — full context for agents
  -c --max-chars N   Truncate contents to fit within N characters

FILTERING:
  -L N                 Limit depth to N levels
  -I "*.rs"            Include only matching files
  -X "*.log"           Exclude matching files
  --use-gitignore      Respect .gitignore (auto|never|always)

SAFETY:
  Safe by default: excludes .env, private keys, node_modules, etc.
  Use --unsafe to disable safety filters (not recommended)
  Use -I patterns to selectively include filtered items"#
)]
pub struct Args {
    /// Target directory to scan
    #[arg(default_value = ".", value_name = "TARGET")]
    pub target: String,

    // ==================== Filtering Options ====================
    /// Limit traversal depth (e.g., -L 3 for max 3 levels deep)
    #[arg(
        short = 'L',
        long = "level",
        value_name = "N",
        help_heading = "Filtering"
    )]
    pub level: Option<usize>,

    /// Include patterns (e.g., -I "*.rs" -I "src/**")
    #[arg(
        short = 'I',
        long = "include",
        value_name = "GLOB",
        help_heading = "Filtering"
    )]
    pub include: Vec<String>,

    /// Exclude patterns (e.g., -X "*.log" -X "temp/**")
    #[arg(
        short = 'X',
        long = "exclude",
        value_name = "GLOB",
        help_heading = "Filtering"
    )]
    pub exclude: Vec<String>,

    /// Respect .gitignore (default: auto)
    #[arg(
        long = "use-gitignore",
        value_enum,
        default_value = "auto",
        value_name = "MODE",
        help_heading = "Filtering"
    )]
    pub use_gitignore: UseGitignoreMode,

    // ==================== Fun & Emojis ====================
    /// Custom emoji mappings (e.g., --emoji ".rs=🚀" --emoji "test=🧪")
    #[arg(long = "emoji", value_name = "MAPPING", help_heading = "Fun & Style")]
    pub emoji: Vec<String>,

    /// Load emoji mappings from TOML file
    #[arg(long = "emoji-map", value_name = "FILE", help_heading = "Fun & Style")]
    pub emoji_map: Option<String>,

    /// Fun mode with emojis and animations
    #[arg(
        long = "fun",
        value_enum,
        default_value = "auto",
        help_heading = "Fun & Style"
    )]
    pub fun: FunMode,

    /// Disable animations
    #[arg(long = "no-anim", conflicts_with = "fun", help_heading = "Fun & Style")]
    pub no_anim: bool,

    // ==================== Statistics ====================
    /// Statistics display: off|min|full (default: full)
    #[arg(
        long = "stats",
        value_enum,
        default_value = "full",
        help_heading = "Statistics"
    )]
    pub stats: StatsMode,

    /// Line counting mode: off|fast|accurate
    #[arg(
        long = "loc",
        value_enum,
        default_value = "fast",
        help_heading = "Statistics"
    )]
    pub loc: LocMode,

    // ==================== Contents ====================
    /// Include file contents as code blocks (for AI context)
    #[arg(short = 'c', long = "contents")]
    pub contents: bool,

    /// Limit total content to N characters — controls AI context budget (only with -c)
    #[arg(
        long = "max-chars",
        value_name = "N",
        requires = "contents",
        help_heading = "Contents"
    )]
    pub max_chars: Option<usize>,

    /// Truncation strategy: head = first N lines, nest = collapse deep indentation (only with --max-chars)
    #[arg(
        long = "contents-mode",
        value_enum,
        default_value = "head",
        help_heading = "Contents"
    )]
    pub contents_mode: ContentsMode,

    // ==================== Safety & Security ====================
    /// Apply safety filters (enabled by default)
    #[arg(long = "safe", help_heading = "Safety")]
    pub safe: bool,

    /// Disable all safety filters (not recommended)
    #[arg(long = "unsafe", conflicts_with = "safe", help_heading = "Safety")]
    pub unsafe_mode: bool,
}

impl Args {
    /// Determine if safe mode is enabled (default: true)
    pub fn is_safe_mode(&self) -> bool {
        !self.unsafe_mode
    }

    /// Check if stats should be shown
    pub fn should_show_stats(&self) -> bool {
        self.stats != StatsMode::Off
    }

    /// Check if fun mode is enabled
    pub fn is_fun_enabled(&self, is_tty: bool) -> bool {
        match self.fun {
            FunMode::On => true,
            FunMode::Off => false,
            FunMode::Auto => is_tty,
        }
    }
}
