use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::credentials::AuthMode;

/// Per-platform install commands for a provider's CLI tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformInstallCommands {
    /// Install command for Linux (e.g., `sudo apt install gh`).
    pub linux: Option<String>,
    /// Install command for macOS (e.g., `brew install gh`).
    pub macos: Option<String>,
    /// Install command for Windows (e.g., `winget install GitHub.cli`).
    pub windows: Option<String>,
}

impl PlatformInstallCommands {
    /// Return the install command for the current OS at runtime.
    pub fn for_current_os(&self) -> Option<&str> {
        let cmd = match std::env::consts::OS {
            "linux" => self.linux.as_deref(),
            "macos" => self.macos.as_deref(),
            "windows" => self.windows.as_deref(),
            _ => None,
        };
        // Fall through: if current OS has no entry, try linux as default
        cmd.or(self.linux.as_deref())
    }
}

/// A model available for selection with a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Model identifier (e.g. "claude-sonnet-4-6").
    pub id: String,
    /// Human-readable model name (e.g. "Claude Sonnet 4.6").
    pub name: String,
    /// Short description (e.g. "Balanced performance and cost").
    pub description: String,
    /// Whether this is the default model for the provider.
    pub is_default: bool,
}

/// Whether a provider can be used in cloud (headless) deployments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum DeploymentConstraint {
    /// Can run in desktop or cloud mode (default).
    #[default]
    Any,
    /// Can only run on the user's local machine — not usable in cloud projects.
    ///
    /// Applies to providers that require a local daemon (Ollama) or interactive
    /// subscription login with no API key mode (GitHub Copilot).
    LocalOnly,
}

/// Metadata describing a provider's installation, authentication, and testing requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Default/fallback install hint (shown when no platform-specific command exists).
    pub install_hint: String,
    /// Per-platform install commands. When set, takes priority over `install_hint`.
    #[serde(default)]
    pub platform_install: PlatformInstallCommands,
    /// Website URL for manual installation instructions.
    pub install_url: Option<String>,
    /// CLI arguments for a lightweight connection test probe.
    pub test_args: Vec<String>,
    /// Supported authentication modes.
    pub auth_modes: Vec<AuthMode>,
    /// Optional shell command to check if the provider is installed.
    ///
    /// When set, this command is run instead of `which <binary>`. Useful for
    /// Python SDK providers that don't install a CLI binary (e.g., xai-sdk, google-genai).
    /// The command should exit 0 when the provider is installed.
    #[serde(default)]
    pub install_check: Option<String>,
    /// Optional bash command to test provider connectivity directly.
    ///
    /// When set, this command is preferred over both `test_args` and `install_check`
    /// in `test_provider`. Useful for providers where the CLI is configured for one
    /// auth mode (e.g., OAuth) but the API can be tested directly without the CLI
    /// (e.g., via curl against the REST API). The command exits 0 on success.
    #[serde(default)]
    pub connection_check: Option<String>,
    /// Available models for user selection in the UI. Empty list means free-text input
    /// (e.g. Ollama where any local model name is valid).
    #[serde(default)]
    pub supported_models: Vec<ModelSpec>,
    /// Path to the provider's local configuration file (tilde notation supported).
    /// e.g. "~/.claude/settings.json". None when the provider has no local config file.
    #[serde(default)]
    pub config_file_path: Option<String>,
    /// CLI arguments for a one-shot completion (prompt appended last).
    ///
    /// When set, `CliNormalizationProvider` appends the prompt text as the final
    /// argument and captures stdout as the completion. `None` means the provider
    /// does not support one-shot invocation and normalization is skipped.
    ///
    /// Example: `Some(vec!["--print".to_string()])` → `claude --print <prompt>`.
    #[serde(default)]
    pub one_shot_args: Option<Vec<String>>,
    /// Whether this provider can be used in cloud (headless) deployments.
    ///
    /// Providers marked `LocalOnly` are filtered out of cloud project wizard
    /// options and blocked at pre-flight validation.
    #[serde(default)]
    pub deployment_constraint: DeploymentConstraint,
    /// Minimum required version string (e.g. `"0.1.0"`).
    ///
    /// When set, `check_provider_status` parses the output of `{binary} --version`
    /// and compares it against this string. If the installed version is older,
    /// the status is `VersionTooOld` rather than `Ready`. `None` disables the check.
    #[serde(default)]
    pub min_version: Option<String>,
}

impl ProviderMetadata {
    /// Create new provider metadata.
    pub fn new(install_hint: impl Into<String>) -> Self {
        Self {
            install_hint: install_hint.into(),
            platform_install: PlatformInstallCommands::default(),
            install_url: None,
            test_args: Vec::new(),
            auth_modes: Vec::new(),
            install_check: None,
            connection_check: None,
            supported_models: Vec::new(),
            config_file_path: None,
            one_shot_args: None,
            deployment_constraint: DeploymentConstraint::Any,
            min_version: None,
        }
    }

    /// Set per-platform install commands.
    pub fn with_platform_install(
        mut self,
        linux: Option<&str>,
        macos: Option<&str>,
        windows: Option<&str>,
    ) -> Self {
        self.platform_install = PlatformInstallCommands {
            linux: linux.map(|s| s.to_string()),
            macos: macos.map(|s| s.to_string()),
            windows: windows.map(|s| s.to_string()),
        };
        self
    }

    /// Return the best install command for the current OS.
    ///
    /// Uses platform-specific command if available, falls back to `install_hint`.
    pub fn install_command_for_current_os(&self) -> &str {
        self.platform_install
            .for_current_os()
            .unwrap_or(&self.install_hint)
    }

    /// Set the install URL.
    pub fn with_install_url(mut self, url: impl Into<String>) -> Self {
        self.install_url = Some(url.into());
        self
    }

    /// Set the test command arguments.
    pub fn with_test_args(mut self, args: Vec<String>) -> Self {
        self.test_args = args;
        self
    }

    /// Add an authentication mode.
    pub fn with_auth_mode(mut self, mode: AuthMode) -> Self {
        self.auth_modes.push(mode);
        self
    }

    /// Set an install check command (used instead of `which <binary>` for SDK-only providers).
    pub fn with_install_check(mut self, command: impl Into<String>) -> Self {
        self.install_check = Some(command.into());
        self
    }

    /// Set a direct connection test command (highest priority in `test_provider`).
    ///
    /// When set, this overrides both `test_args` and `install_check` for the
    /// connection test. The command is run via `bash -c` with provider env vars
    /// injected. Use this to bypass CLI OAuth quirks and test the API directly.
    pub fn with_connection_check(mut self, command: impl Into<String>) -> Self {
        self.connection_check = Some(command.into());
        self
    }

    /// Set the list of available models for user selection in the UI.
    pub fn with_models(mut self, models: Vec<ModelSpec>) -> Self {
        self.supported_models = models;
        self
    }

    /// Set the path to the provider's local configuration file.
    ///
    /// Supports tilde notation (e.g. `"~/.claude/settings.json"`).
    pub fn with_config_file(mut self, path: &str) -> Self {
        self.config_file_path = Some(path.to_string());
        self
    }

    /// Set the CLI arguments used for one-shot normalization completions.
    ///
    /// The prompt text is always appended as the final argument at invocation
    /// time. Pass `None` (the default) to indicate that this provider does not
    /// support one-shot mode, causing normalization to be skipped for it.
    pub fn with_one_shot_args(mut self, args: Vec<String>) -> Self {
        self.one_shot_args = Some(args);
        self
    }

    /// Mark this provider as local-only, preventing its use in cloud projects.
    pub fn with_deployment_constraint(mut self, constraint: DeploymentConstraint) -> Self {
        self.deployment_constraint = constraint;
        self
    }

    /// Set the minimum required version string (e.g. `"0.40.0"`).
    ///
    /// When set, the status check will run `{binary} --version`, extract the
    /// first `X.Y.Z` version found in the output, and compare it against this
    /// string. If the installed version is older, status becomes `VersionTooOld`.
    pub fn with_min_version(mut self, version: impl Into<String>) -> Self {
        self.min_version = Some(version.into());
        self
    }
}

/// Parse the first semver-like `X.Y.Z` triple from arbitrary version output.
///
/// Returns `None` if no version string can be found.
pub fn parse_version_from_output(output: &str) -> Option<(u32, u32, u32)> {
    // Match patterns like "1.2.3" or "v1.2.3" anywhere in the string.
    let re = regex::Regex::new(r"v?(\d+)\.(\d+)\.(\d+)").ok()?;
    let caps = re.captures(output)?;
    let major: u32 = caps[1].parse().ok()?;
    let minor: u32 = caps[2].parse().ok()?;
    let patch: u32 = caps[3].parse().ok()?;
    Some((major, minor, patch))
}

/// Return true if `found` is at least as new as `required`.
///
/// Both are `(major, minor, patch)` tuples compared lexicographically.
pub fn version_meets_minimum(found: (u32, u32, u32), required: (u32, u32, u32)) -> bool {
    found >= required
}

/// Search for a binary on the system PATH.
///
/// Returns the full path if the binary is found, `None` otherwise.
/// Uses `which` on Unix and `where.exe` on Windows.
pub fn find_binary(name: &str) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        let output = std::process::Command::new("which")
            .arg(name)
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }
    #[cfg(windows)]
    {
        let output = std::process::Command::new("where.exe")
            .arg(name)
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()?
                .trim()
                .to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }
}

/// Check if a CLI login check command succeeds (exit code 0).
pub fn check_cli_auth(check_command: &str) -> bool {
    let parts: Vec<&str> = check_command.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }
    let output = std::process::Command::new(parts[0])
        .args(&parts[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Async version of [`find_binary`] — uses `tokio::process::Command` to avoid blocking.
pub async fn find_binary_async(name: &str) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        let output = tokio::process::Command::new("which")
            .arg(name)
            .output()
            .await
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }
    #[cfg(windows)]
    {
        let output = tokio::process::Command::new("where.exe")
            .arg(name)
            .output()
            .await
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()?
                .trim()
                .to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }
}

/// Run an install check command asynchronously.
///
/// Returns a synthetic path with the module version if the check succeeds,
/// `None` otherwise. Used for SDK-only providers (e.g., Python SDKs) where
/// `which <binary>` doesn't apply.
pub async fn run_install_check_async(check_command: &str) -> Option<PathBuf> {
    let output = tokio::process::Command::new("bash")
        .arg("-c")
        .arg(check_command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .await
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Return "Python SDK (version)" for a meaningful UI display, or just
        // "(Python SDK)" if the check command didn't print a version.
        if !stdout.is_empty() {
            Some(PathBuf::from(format!("Python SDK v{stdout}")))
        } else {
            Some(PathBuf::from("Python SDK"))
        }
    } else {
        None
    }
}

/// Async version of [`check_cli_auth`] — uses `tokio::process::Command` to avoid blocking.
///
/// Times out after 5 seconds so a hanging auth check (e.g. `claude auth status` waiting
/// for network) never blocks the provider status scan.
pub async fn check_cli_auth_async(check_command: &str) -> bool {
    let parts: Vec<&str> = check_command.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }
    let fut = tokio::process::Command::new(parts[0])
        .args(&parts[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
    match tokio::time::timeout(std::time::Duration::from_secs(5), fut).await {
        Ok(Ok(o)) => o.status.success(),
        Ok(Err(_)) | Err(_) => false,
    }
}
