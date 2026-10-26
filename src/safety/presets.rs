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
            // Environment files (at any depth: `**/` also matches the root)
            "**/.env",
            "**/.env.*",
            "**/.envrc",
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
            // Tokens and passwords kept by package managers, VCS, container,
            // cloud and database clients (targets of credential stealers)
            "**/.npmrc",
            "**/.pypirc",
            "**/.netrc",
            "**/_netrc",
            "**/.git-credentials",
            "**/.docker/config.json",
            "**/.kube/config",
            "**/kubeconfig",
            "*.kubeconfig",
            "*.tfstate",
            "*.tfstate.*",
            "**/.pgpass",
            "**/.my.cnf",
            "*.ppk",
            "**/.gnupg/**",
            // PHP / web hosting: Composer auth tokens, WordPress DB credentials
            // and salts, HTTP basic-auth password hashes
            "**/auth.json",
            "**/wp-config.php",
            "**/.htpasswd",
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
            "**/.DS_Store",
            "**/Thumbs.db",
            "**/desktop.ini",
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

        // Should match OS files, at any depth
        assert!(preset.matches(".DS_Store"));
        assert!(preset.matches("Thumbs.db"));
        assert!(preset.matches("assets/img/.DS_Store"));
        assert!(preset.matches("media/Thumbs.db"));

        // Should NOT match normal files
        assert!(!preset.matches("src/main.rs"));
        assert!(!preset.matches("README.md"));
        assert!(!preset.matches("Cargo.toml"));
    }

    #[test]
    fn test_env_files_match_at_any_depth() {
        let preset = SafetyPreset::new();

        for path in [
            ".env",
            "apps/web/.env",
            "backend/.env.local",
            "services/api/.env.production",
            ".envrc",
            "tools/.envrc",
        ] {
            assert!(preset.matches(path), "should match {}", path);
        }
        for path in ["src/env.rs", "environment.md", "docs/dotenv.md"] {
            assert!(!preset.matches(path), "should not match {}", path);
        }
    }

    #[test]
    fn test_credential_files_match_at_any_depth() {
        let preset = SafetyPreset::new();

        for path in [
            ".npmrc",
            "packages/app/.npmrc",
            ".pypirc",
            ".netrc",
            "_netrc",
            ".git-credentials",
            ".docker/config.json",
            "home/.docker/config.json",
            ".kube/config",
            "deploy/kubeconfig",
            "deploy/prod.kubeconfig",
            "terraform.tfstate",
            "infra/prod/terraform.tfstate.backup",
            ".pgpass",
            ".my.cnf",
            "keys/server.ppk",
            ".gnupg/secring.gpg",
            "home/.gnupg/pubring.kbx",
            // PHP / web hosting
            "auth.json",
            "app/auth.json",
            "wp-config.php",
            "site/public/wp-config.php",
            "site/.htpasswd",
        ] {
            assert!(preset.matches(path), "should match {}", path);
        }
        // `.gnupg` directories are pruned (dirs are also checked with a trailing `/`)
        assert!(preset.matches("home/.gnupg/"));

        for path in [
            "src/oauth.json",
            "src/auth.rs",
            "docs/auth.json.md",
            "wp-config-sample.php",
            "config/docker.json",
            "src/kube/config.rs",
            "src/id_generator.rs",
            "terraform/main.tf",
        ] {
            assert!(!preset.matches(path), "should not match {}", path);
        }
    }
}
