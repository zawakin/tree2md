use std::env;

/// Detects terminal environment and capabilities
pub struct TerminalDetector {
    is_tty: bool,
    is_ci: bool,
    force_mode: Option<TerminalMode>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalMode {
    Tty,
    Plain,
}

impl TerminalDetector {
    /// Create a new terminal detector with automatic detection
    pub fn new() -> Self {
        let is_tty = atty::is(atty::Stream::Stdout);
        let is_ci = Self::detect_ci();

        Self {
            is_tty,
            is_ci,
            force_mode: None,
        }
    }

    /// Force a specific terminal mode (for testing or user override)
    pub fn with_mode(mut self, mode: TerminalMode) -> Self {
        self.force_mode = Some(mode);
        self
    }

    /// Check if we're in a TTY environment suitable for rich output
    pub fn is_tty(&self) -> bool {
        if let Some(mode) = self.force_mode {
            return mode == TerminalMode::Tty;
        }

        // No rich output in CI environments even if TTY
        self.is_tty && !self.is_ci
    }

    /// Check if we're in a CI environment
    pub fn is_ci_environment(&self) -> bool {
        self.is_ci
    }

    /// Detect if we're running in a CI environment
    fn detect_ci() -> bool {
        // Common CI environment variables
        env::var("CI").is_ok()
            || env::var("CONTINUOUS_INTEGRATION").is_ok()
            || env::var("GITHUB_ACTIONS").is_ok()
            || env::var("GITLAB_CI").is_ok()
            || env::var("TRAVIS").is_ok()
            || env::var("CIRCLECI").is_ok()
            || env::var("JENKINS_URL").is_ok()
            || env::var("TEAMCITY_VERSION").is_ok()
    }

    /// Get the recommended output mode based on detection
    pub fn output_mode(&self) -> TerminalMode {
        if self.is_tty() {
            TerminalMode::Tty
        } else {
            TerminalMode::Plain
        }
    }

    /// Check if colors/animations should be enabled
    pub fn should_use_colors(&self) -> bool {
        if !self.is_tty() {
            return false;
        }

        // Check for explicit color environment variables
        if let Ok(val) = env::var("NO_COLOR") {
            if !val.is_empty() {
                return false;
            }
        }

        if let Ok(val) = env::var("FORCE_COLOR") {
            if !val.is_empty() && val != "0" {
                return true;
            }
        }

        // Check TERM environment variable
        if let Ok(term) = env::var("TERM") {
            if term == "dumb" {
                return false;
            }
        }

        true
    }

    /// Check if unicode should be used
    pub fn supports_unicode(&self) -> bool {
        if !self.is_tty() {
            return false;
        }

        // Check for UTF-8 support in locale
        if let Ok(lang) = env::var("LANG") {
            if lang.to_lowercase().contains("utf-8") || lang.to_lowercase().contains("utf8") {
                return true;
            }
        }

        if let Ok(lc_all) = env::var("LC_ALL") {
            if lc_all.to_lowercase().contains("utf-8") || lc_all.to_lowercase().contains("utf8") {
                return true;
            }
        }

        // Default to true on most modern systems
        cfg!(not(windows)) || env::var("WT_SESSION").is_ok() // Windows Terminal supports Unicode
    }
}

impl Default for TerminalDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_mode() {
        let detector = TerminalDetector::new().with_mode(TerminalMode::Plain);
        assert!(!detector.is_tty());

        let detector = TerminalDetector::new().with_mode(TerminalMode::Tty);
        assert!(detector.is_tty());
    }

    #[test]
    fn test_output_mode() {
        let detector = TerminalDetector::new().with_mode(TerminalMode::Tty);
        assert_eq!(detector.output_mode(), TerminalMode::Tty);

        let detector = TerminalDetector::new().with_mode(TerminalMode::Plain);
        assert_eq!(detector.output_mode(), TerminalMode::Plain);
    }

    #[test]
    fn test_terminal_mode_precedence() {
        // Test that Force > CI > TTY
        let detector = TerminalDetector::new().with_mode(TerminalMode::Tty);
        assert!(detector.is_tty());

        let detector = TerminalDetector::new().with_mode(TerminalMode::Plain);
        assert!(!detector.is_tty());
    }

    #[test]
    fn test_ci_detection() {
        // Save original env
        let original_ci = env::var("CI").ok();
        let original_github = env::var("GITHUB_ACTIONS").ok();

        // Test CI=true detection
        env::set_var("CI", "true");
        let detector = TerminalDetector::new();
        assert!(detector.is_ci_environment());
        env::remove_var("CI");

        // Test CI=1 detection
        env::set_var("CI", "1");
        let detector = TerminalDetector::new();
        assert!(detector.is_ci_environment());
        env::remove_var("CI");

        // Test GITHUB_ACTIONS detection
        env::set_var("GITHUB_ACTIONS", "true");
        let detector = TerminalDetector::new();
        assert!(detector.is_ci_environment());
        env::remove_var("GITHUB_ACTIONS");

        // Restore original env
        if let Some(val) = original_ci {
            env::set_var("CI", val);
        }
        if let Some(val) = original_github {
            env::set_var("GITHUB_ACTIONS", val);
        }
    }

    #[test]
    fn test_terminal_tty_detection() {
        // Test TTY detection
        let detector = TerminalDetector::new().with_mode(TerminalMode::Plain);
        assert!(!detector.is_tty());

        // Test TTY mode
        let detector = TerminalDetector::new().with_mode(TerminalMode::Tty);
        assert!(detector.is_tty());
    }
}
