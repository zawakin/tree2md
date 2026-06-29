use clap::error::ErrorKind;
use clap::{Parser, ValueEnum};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse CLI arguments, exiting with actionable guidance on usage errors.
///
/// On `--help`/`--version` we defer to clap (stdout, exit 0). On a usage error
/// we keep clap's message + "similar argument" tip, then append a short block of
/// runnable next commands so a failed invocation points straight at the fix.
pub fn parse() -> Args {
    let argv: Vec<String> = std::env::args().collect();
    match Args::try_parse_from(&argv) {
        Ok(mut args) => {
            args.filter_rules = extract_filter_rules(argv.iter().map(|s| s.as_str()));
            args
        }
        Err(e) => {
            e.print().ok();
            let is_help_or_version = matches!(
                e.kind(),
                ErrorKind::DisplayHelp
                    | ErrorKind::DisplayVersion
                    | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            );
            if !is_help_or_version {
                eprintln!();
                eprintln!("Try:");
                eprintln!("  tree2md                # pretty tree of the current directory");
                eprintln!("  tree2md -c | pbcopy    # tree + file contents for AI context");
            }
            std::process::exit(e.exit_code());
        }
    }
}

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
#[command(
    about = "Like the `tree` command, but optimized for AI agents.\n\nQUICK START:\n  tree2md                # pretty tree of the current directory\n  tree2md -c | pbcopy    # tree + file contents, copied for AI context"
)]
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

    /// Follow symbolic links (default: symlinks are skipped)
    #[arg(
        short = 'l',
        long = "follow-links",
        help_heading = "Filtering",
        hide_short_help = true
    )]
    pub follow_links: bool,

    // ==================== Fun & Emojis ====================
    /// Custom emoji mappings (e.g., --emoji ".rs=🚀" --emoji "test=🧪")
    #[arg(
        long = "emoji",
        value_name = "MAPPING",
        help_heading = "Fun & Style",
        hide_short_help = true
    )]
    pub emoji: Vec<String>,

    /// Load emoji mappings from TOML file
    #[arg(
        long = "emoji-map",
        value_name = "FILE",
        help_heading = "Fun & Style",
        hide_short_help = true
    )]
    pub emoji_map: Option<String>,

    /// Fun mode with emojis and animations
    #[arg(
        long = "fun",
        value_enum,
        default_value = "auto",
        help_heading = "Fun & Style",
        hide_short_help = true
    )]
    pub fun: FunMode,

    /// Disable animations
    #[arg(
        long = "no-anim",
        conflicts_with = "fun",
        help_heading = "Fun & Style",
        hide_short_help = true
    )]
    pub no_anim: bool,

    // ==================== Statistics ====================
    /// Statistics display: off|min|full (default: full)
    #[arg(
        long = "stats",
        value_enum,
        default_value = "full",
        help_heading = "Statistics",
        hide_short_help = true
    )]
    pub stats: StatsMode,

    /// Line counting mode: off|fast|accurate
    #[arg(
        long = "loc",
        value_enum,
        default_value = "fast",
        help_heading = "Statistics",
        hide_short_help = true
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
        help_heading = "Contents",
        hide_short_help = true
    )]
    pub contents_mode: ContentsMode,

    // ==================== Safety & Security ====================
    /// Apply safety filters (enabled by default)
    #[arg(long = "safe", help_heading = "Safety", hide_short_help = true)]
    pub safe: bool,

    /// Disable all safety filters (not recommended)
    #[arg(long = "unsafe", conflicts_with = "safe", help_heading = "Safety")]
    pub unsafe_mode: bool,

    /// Ordered -I / -X rules captured from argv. Populated AFTER clap parses
    /// because clap derive cannot preserve the relative order of `-I` and
    /// `-X` across two separate Vec fields. Last-match-wins semantics relies
    /// on this order.
    #[arg(skip)]
    pub filter_rules: Vec<FilterRule>,
}

/// Whether a filter rule includes or excludes paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuleKind {
    #[default]
    Include,
    Exclude,
}

/// A single -I/-X rule with its original pattern, preserving CLI order.
#[derive(Debug, Clone, Default)]
pub struct FilterRule {
    pub kind: RuleKind,
    pub pattern: String,
}

/// Walk argv and extract `-I` / `-X` (and `--include` / `--exclude`) in the
/// order they appeared on the command line. Used to drive last-match-wins
/// semantics in the filter engine.
pub fn extract_filter_rules<I, S>(argv: I) -> Vec<FilterRule>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut rules = Vec::new();
    let mut iter = argv.into_iter().peekable();
    // Skip program name
    iter.next();
    while let Some(arg) = iter.next() {
        let a = arg.as_ref();
        if a == "--" {
            break;
        }
        let push = |kind: RuleKind, pat: String, rules: &mut Vec<FilterRule>| {
            rules.push(FilterRule { kind, pattern: pat });
        };
        // Long forms with `=`
        if let Some(rest) = a.strip_prefix("--include=") {
            push(RuleKind::Include, rest.to_string(), &mut rules);
            continue;
        }
        if let Some(rest) = a.strip_prefix("--exclude=") {
            push(RuleKind::Exclude, rest.to_string(), &mut rules);
            continue;
        }
        // Long forms with space
        if a == "--include" {
            if let Some(v) = iter.next() {
                push(RuleKind::Include, v.as_ref().to_string(), &mut rules);
            }
            continue;
        }
        if a == "--exclude" {
            if let Some(v) = iter.next() {
                push(RuleKind::Exclude, v.as_ref().to_string(), &mut rules);
            }
            continue;
        }
        // Short forms `-I`, `-X` with space
        if a == "-I" {
            if let Some(v) = iter.next() {
                push(RuleKind::Include, v.as_ref().to_string(), &mut rules);
            }
            continue;
        }
        if a == "-X" {
            if let Some(v) = iter.next() {
                push(RuleKind::Exclude, v.as_ref().to_string(), &mut rules);
            }
            continue;
        }
        // Short forms `-Ifoo`, `-Xfoo` (attached value)
        if let Some(rest) = a.strip_prefix("-I") {
            if !rest.is_empty() {
                push(RuleKind::Include, rest.to_string(), &mut rules);
                continue;
            }
        }
        if let Some(rest) = a.strip_prefix("-X") {
            if !rest.is_empty() {
                push(RuleKind::Exclude, rest.to_string(), &mut rules);
                continue;
            }
        }
    }
    rules
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
