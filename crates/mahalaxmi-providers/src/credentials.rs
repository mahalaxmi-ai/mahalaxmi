use serde::{Deserialize, Serialize};

/// Method used to authenticate with an AI provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthMethod {
    /// API key passed via environment variable.
    ApiKey,
    /// AWS IAM role-based authentication.
    AwsIam,
    /// OAuth 2.0 token flow.
    OAuth,
    /// Read credential from an environment variable.
    EnvironmentVariable,
    /// Read credential from the OS system keyring.
    SystemKeyring,
    /// Google Service Account key file path via environment variable.
    ServiceAccount,
}

/// Specification of a credential required by a provider.
///
/// The `description_key` is an i18n key — resolve it via `I18nService` before
/// displaying to the user. The `Debug` implementation intentionally omits it.
#[derive(Clone, Serialize, Deserialize)]
pub struct CredentialSpec {
    /// The authentication method.
    pub method: AuthMethod,
    /// Name of the environment variable to check (e.g., "ANTHROPIC_API_KEY").
    pub env_var_name: Option<String>,
    /// i18n key for the human-readable description (e.g., "credential-anthropic-api-key").
    /// Resolve via `I18nService::translate()` before displaying to the user.
    pub description_key: String,
    /// Whether this credential is required (vs. optional fallback).
    pub required: bool,
}

impl std::fmt::Debug for CredentialSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialSpec")
            .field("method", &self.method)
            .field("env_var_name", &self.env_var_name)
            .field("required", &self.required)
            .finish()
    }
}

/// Multi-level status for a provider's readiness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    /// CLI binary not found on PATH.
    NotInstalled,
    /// Binary found but credentials missing or invalid.
    NotConfigured,
    /// Binary + credentials present, not yet tested.
    Ready,
    /// Successfully tested with a live probe.
    Verified,
    /// Test failed (bad key, network issue, etc.).
    Error(String),
    /// Installed binary is older than the required minimum version.
    ///
    /// Payload: human-readable message with found and required versions.
    VersionTooOld(String),
}

impl ProviderStatus {
    /// Returns the status as a string slug for serialization to the frontend.
    pub fn as_str(&self) -> &str {
        match self {
            Self::NotInstalled => "not_installed",
            Self::NotConfigured => "not_configured",
            Self::Ready => "ready",
            Self::Verified => "verified",
            Self::Error(_) => "error",
            Self::VersionTooOld(_) => "version_too_old",
        }
    }
}

impl std::fmt::Display for ProviderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "not_installed"),
            Self::NotConfigured => write!(f, "not_configured"),
            Self::Ready => write!(f, "ready"),
            Self::Verified => write!(f, "verified"),
            Self::Error(msg) => write!(f, "error: {msg}"),
            Self::VersionTooOld(msg) => write!(f, "version_too_old: {msg}"),
        }
    }
}

/// Authentication mode supported by a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMode {
    /// Interactive CLI login (e.g., `claude auth login`, `gh auth login`).
    CliLogin {
        login_command: String,
        check_command: String,
    },
    /// API key via environment variable.
    ApiKey { env_var: String },
    /// No authentication needed (local models).
    None,
    /// Google Service Account key file path via environment variable.
    ServiceAccount {
        env_var: String,
        description: String, // Added description for the UI
    },
}
