use clap::{Parser, ValueEnum};

pub const VERSION: &str = "0.7.0";

#[derive(Debug, Clone, ValueEnum)]
pub enum UseGitignoreMode {
    /// Use .gitignore if in a git repository
    Auto,
    /// Never use .gitignore
    Never,
    /// Always use .gitignore
    Always,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FoldMode {
    /// Use <details> for directories with many items
    Auto,
    /// Always use <details> for directories
    On,
    /// Never use <details>
    Off,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum LinksMode {
    /// Generate clickable links
    On,
    /// Plain text without links
    Off,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum StampMode {
    /// Include version
    Version,
    /// Include date
    Date,
    /// Include commit hash
    Commit,
    /// No stamp
    None,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum OutputMode {
    /// Auto-detect based on terminal
    Auto,
    /// Terminal output with tree characters
    Tty,
    /// Pure Markdown output
    Md,
    /// HTML output with collapsible sections
    Html,
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
pub enum PresetMode {
    /// Optimized for README files
    Readme,
    /// Light mode with minimal features
    Light,
    /// Fun mode with all features
    Fun,
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

#[derive(Parser, Clone)]
#[command(name = "tree2md")]
#[command(version = VERSION)]
#[command(about = "📁 Generate beautiful tree structures in Markdown, HTML, or Terminal format")]
#[command(
    long_about = r#"tree2md — Generate a GitHub-ready Markdown tree with clickable links.

🚀 QUICK START:
  tree2md                          # Generate tree of current directory
  tree2md src/                     # Generate tree of src/ directory
  tree2md > STRUCTURE.md           # Save output to file
  tree2md --output html            # Generate HTML with collapsible sections

📝 COMMON USAGE:
  # For README: Add GitHub links and inject into README.md
  tree2md . --github https://github.com/you/repo/tree/main --inject README.md

  # Simple markdown tree with stats
  tree2md . --stats full > STRUCTURE.md

  # HTML with collapsible folders (great for large projects)
  tree2md . --output html --fold auto > tree.html

  # Limit depth and filter by extension
  tree2md . -L 3 -I "*.rs" -I "*.toml"

🎨 OUTPUT FORMATS:
  --output md        Markdown with bullet lists (default when piping)
  --output html      HTML with <ul>/<li> and collapsible <details>
  --output tty       Terminal tree with box characters (default in terminal)
  --output auto      Auto-detect based on terminal (default)

⚡ PRESETS:
  --preset readme    Optimized for README files (markdown, full stats, no fun)
  --preset light     Minimal output with basic stats
  --preset fun       Colorful output with emojis and animations

🔒 SAFETY:
  • Safe by default: excludes .env, private keys, node_modules, etc.
  • Use --unsafe to disable safety filters (not recommended)
  • Use -I patterns to selectively include filtered items
  • No file contents are ever emitted

📌 KEY FEATURES:
  • Respects .gitignore automatically in git repos
  • Deterministic output (dirs first, alphabetical)
  • Symlinks are always skipped
  • GitHub URL rewriting for clickable links
  • Idempotent README injection
  • File type statistics and line counting"#
)]
pub struct Args {
    /// Target directory to scan
    #[arg(default_value = ".", value_name = "TARGET")]
    pub target: String,

    // ==================== 📁 Filtering Options ====================
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

    // ==================== 🔗 Links & GitHub ====================
    /// Generate clickable links in output (default: on)
    #[arg(
        long = "links",
        value_enum,
        default_value = "on",
        help_heading = "Links & GitHub"
    )]
    pub links: LinksMode,

    /// Rewrite links to GitHub URL (e.g., https://github.com/user/repo/tree/main)
    #[arg(long = "github", value_name = "URL", help_heading = "Links & GitHub")]
    pub github: Option<String>,

    // ==================== 😊 Fun & Emojis ====================
    /// Custom emoji mappings (e.g., --emoji ".rs=🚀" --emoji "test=🧪")
    #[arg(long = "emoji", value_name = "MAPPING", help_heading = "Fun & Style")]
    pub emoji: Vec<String>,

    /// Load emoji mappings from TOML file
    #[arg(long = "emoji-map", value_name = "FILE", help_heading = "Fun & Style")]
    pub emoji_map: Option<String>,

    // ==================== 📄 Output Format ====================
    /// Output format: auto|tty|md|html (default: auto-detect)
    #[arg(
        long = "output",
        value_enum,
        default_value = "auto",
        help_heading = "Output Format"
    )]
    pub output: OutputMode,

    /// Quick presets: readme|light|fun
    #[arg(long = "preset", value_enum, help_heading = "Output Format")]
    pub preset: Option<PresetMode>,

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

    /// Collapsible folders in HTML output (auto|on|off)
    #[arg(
        long = "fold",
        value_enum,
        default_value = "auto",
        help_heading = "Output Format"
    )]
    pub fold: FoldMode,

    /// Statistics display: off|min|full (default: full)
    #[arg(
        long = "stats",
        value_enum,
        default_value = "full",
        help_heading = "Statistics"
    )]
    pub stats: StatsMode,

    /// Suppress stats footer (deprecated, use --stats off)
    #[arg(long = "no-stats", hide = true)]
    pub no_stats: bool,

    /// Line counting mode: off|fast|accurate
    #[arg(
        long = "loc",
        value_enum,
        default_value = "fast",
        help_heading = "Statistics"
    )]
    pub loc: LocMode,

    /// (deprecated) Dump file contents as Markdown code blocks
    #[arg(
        short = 'c',
        long = "contents",
        help = "(deprecated) Dump file contents as Markdown code blocks."
    )]
    pub contents: bool,

    // ==================== 🔒 Safety & Security ====================
    /// Apply safety filters (enabled by default)
    #[arg(long = "safe", help_heading = "Safety")]
    pub safe: bool,

    /// Disable all safety filters (⚠️ not recommended)
    #[arg(long = "unsafe", conflicts_with = "safe", help_heading = "Safety")]
    pub unsafe_mode: bool,

    /// Restrict traversal to this root directory
    #[arg(long = "restrict-root", value_name = "DIR", help_heading = "Safety")]
    pub restrict_root: Option<String>,

    // ==================== 🏷️ Metadata & Stamps ====================
    /// Add metadata stamp: version|date|commit|none
    #[arg(
        long = "stamp",
        value_enum,
        value_name = "MODE",
        help_heading = "Metadata"
    )]
    pub stamp: Vec<StampMode>,

    /// Date format for stamp (strftime syntax)
    #[arg(
        long = "stamp-date-format",
        default_value = "%Y-%m-%d",
        value_name = "FMT",
        help_heading = "Metadata"
    )]
    pub stamp_date_format: String,

    // ==================== 📝 README Injection ====================
    /// Replace tagged block in file (e.g., README.md)
    #[arg(
        long = "inject",
        value_name = "FILE",
        help_heading = "README Injection"
    )]
    pub inject: Option<String>,

    /// Start tag for injection
    #[arg(
        long = "tag-start",
        default_value = "<!-- tree2md:start -->",
        value_name = "STR",
        help_heading = "README Injection"
    )]
    pub tag_start: String,

    /// End tag for injection
    #[arg(
        long = "tag-end",
        default_value = "<!-- tree2md:end -->",
        value_name = "STR",
        help_heading = "README Injection"
    )]
    pub tag_end: String,

    /// Preview changes without writing files
    #[arg(long = "dry-run", help_heading = "README Injection")]
    pub dry_run: bool,
}

impl Args {
    /// Apply preset configurations
    pub fn apply_preset(&mut self) {
        if let Some(preset) = &self.preset {
            match preset {
                PresetMode::Readme => {
                    // Optimized for README files
                    self.output = OutputMode::Md;
                    self.stats = StatsMode::Full;
                    self.fun = FunMode::Off;
                    self.loc = LocMode::Fast;
                    // Don't override links, they should stay on for README
                }
                PresetMode::Light => {
                    // Light mode with minimal features
                    self.output = OutputMode::Auto;
                    self.stats = StatsMode::Min;
                    self.fun = FunMode::Auto;
                }
                PresetMode::Fun => {
                    // Fun mode with all features
                    self.output = OutputMode::Auto;
                    self.stats = StatsMode::Full;
                    self.fun = FunMode::On;
                }
            }
        }

        // Handle deprecated no_stats flag
        if self.no_stats {
            self.stats = StatsMode::Off;
        }

        // Handle no_anim flag
        if self.no_anim {
            self.fun = FunMode::Off;
        }
    }

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

    /// Determine output format
    pub fn output_format(&self, is_tty: bool) -> OutputMode {
        match &self.output {
            OutputMode::Auto => {
                if is_tty {
                    OutputMode::Tty
                } else {
                    OutputMode::Md
                }
            }
            mode => mode.clone(),
        }
    }

    /// Validate arguments for logical consistency
    pub fn validate(&self) -> Result<(), String> {
        // Validate GitHub URL format if provided
        if let Some(github_url) = &self.github {
            if !github_url.starts_with("http://") && !github_url.starts_with("https://") {
                return Err("GitHub URL must start with http:// or https://".to_string());
            }
        }

        // Validate restrict-root exists if provided
        if let Some(restrict_root) = &self.restrict_root {
            if !std::path::Path::new(restrict_root).exists() {
                return Err(format!(
                    "Restrict root directory does not exist: {}",
                    restrict_root
                ));
            }
        }

        // Validate inject file exists if provided
        if let Some(inject_file) = &self.inject {
            if !self.dry_run && !std::path::Path::new(inject_file).exists() {
                return Err(format!("Inject file does not exist: {}", inject_file));
            }
        }

        Ok(())
    }

    /// Check if deprecated features are used and warn
    pub fn check_deprecated(&self) {
        if self.contents {
            eprintln!("Warning: --contents is deprecated and will be ignored in this version.");
            eprintln!("         For legacy AI-context use only.");
        }
    }
}
