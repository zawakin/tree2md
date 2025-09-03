use glob::Pattern;

/// Safety preset that defines conservative exclude patterns
#[derive(Debug, Clone)]
pub struct SafetyPreset {
    patterns: Vec<Pattern>,
}

impl SafetyPreset {
    /// Create a new SafetyPreset with default patterns
    pub fn new() -> Self {
        let pattern_strings = Self::default_patterns();
        let patterns = pattern_strings
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();

        SafetyPreset { patterns }
    }

    /// Get the default safety patterns
    pub fn default_patterns() -> Vec<&'static str> {
        vec![
            // Environment files
            ".env",
            ".env.*",
            // SSH and security files
            ".ssh/**",
            "**/.ssh/**",
            "*.pem",
            "*.key",
            "*.p12",
            "*.pfx",
            "*_rsa",
            "*_dsa",
            "*_ecdsa",
            "*_ed25519",
            "id_*",
            // Credentials and secrets
            "*.crt",
            "*.cer",
            "*.der",
            "*.keystore",
            "*.jks",
            "credentials",
            "credentials.*",
            "**/credentials",
            "**/credentials.*",
            "secrets",
            "secrets.*",
            "**/secrets",
            "**/secrets.*",
            // Heavy build directories
            "target/**",
            "**/target/**",
            "node_modules/**",
            "**/node_modules/**",
            "vendor/**",
            "**/vendor/**",
            "dist/**",
            "**/dist/**",
            "build/**",
            "**/build/**",
            ".build/**",
            "**/.build/**",
            "out/**",
            "**/out/**",
            // Package manager directories
            ".npm/**",
            "**/.npm/**",
            ".yarn/**",
            "**/.yarn/**",
            ".pnpm-store/**",
            "**/.pnpm-store/**",
            // IDE and editor directories
            ".idea/**",
            "**/.idea/**",
            ".vscode/**",
            "**/.vscode/**",
            "*.swp",
            "*.swo",
            "*~",
            // OS-specific files
            ".DS_Store",
            "Thumbs.db",
            "desktop.ini",
            // Cache directories
            ".cache/**",
            "**/.cache/**",
            "__pycache__/**",
            "**/__pycache__/**",
            "*.pyc",
            // Git directory (always excluded)
            ".git/**",
            "**/.git/**",
            // Other sensitive or large directories
            "logs/**",
            "**/logs/**",
            "*.log",
            "tmp/**",
            "**/tmp/**",
            "temp/**",
            "**/temp/**",
        ]
    }

    /// Check if a path matches any safety pattern
    pub fn matches(&self, path: &str) -> bool {
        // Normalize path separators to forward slashes for consistent matching
        let normalized_path = path.replace('\\', "/");

        self.patterns
            .iter()
            .any(|pattern| pattern.matches(&normalized_path))
    }
}

impl Default for SafetyPreset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_preset_matches() {
        let preset = SafetyPreset::new();

        // Should match environment files
        assert!(preset.matches(".env"));
        assert!(preset.matches(".env.local"));
        assert!(preset.matches(".env.production"));

        // Should match SSH files
        assert!(preset.matches(".ssh/id_rsa"));
        assert!(preset.matches("home/.ssh/config"));

        // Should match key files
        assert!(preset.matches("server.pem"));
        assert!(preset.matches("private.key"));
        assert!(preset.matches("id_rsa"));

        // Should match build directories
        assert!(preset.matches("target/debug/app"));
        assert!(preset.matches("node_modules/package/index.js"));
        assert!(preset.matches("project/node_modules/lib.js"));

        // Should match OS files
        assert!(preset.matches(".DS_Store"));
        assert!(preset.matches("Thumbs.db"));

        // Should NOT match normal files
        assert!(!preset.matches("src/main.rs"));
        assert!(!preset.matches("README.md"));
        assert!(!preset.matches("Cargo.toml"));
    }
}
