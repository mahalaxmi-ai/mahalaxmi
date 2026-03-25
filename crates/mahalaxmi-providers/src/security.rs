//! Automatic sensitive code routing — zero-configuration security classification.
//!
//! Tasks are automatically classified as `Normal`, `Sensitive`, or `Restricted`
//! based on their title, description, and affected files. Routing behavior adjusts
//! automatically — users never need to configure routing constraints manually.
//!
//! - **Restricted**: Must route to a local provider (e.g., Ollama). Hard block on cloud.
//! - **Sensitive**: Prefers local provider. Falls back to cloud with a notification.
//! - **Normal**: Routes normally per TaskRouter scoring logic.

use serde::{Deserialize, Serialize};

use crate::types::CostTier;
use crate::RoutingConstraints;
use mahalaxmi_core::types::ProviderId;

/// Security classification for a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityClassification {
    /// No security concerns — route anywhere.
    Normal,
    /// Contains authentication/security logic — prefer local, warn if cloud.
    Sensitive,
    /// Contains secrets, credentials, or private keys — local only, hard block on cloud.
    Restricted,
}

/// User-facing setting for security routing behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityRoutingMode {
    /// Auto-classify tasks and route accordingly (default).
    #[default]
    Automatic,
    /// Route everything to local providers (high-security environments).
    AllLocal,
    /// Disable security routing — user takes responsibility.
    Disabled,
}

/// Result of applying security routing to constraints.
#[derive(Debug, Clone)]
pub struct SecurityRoutingResult {
    /// The classification that was applied.
    pub classification: SecurityClassification,
    /// Modified routing constraints (may add exclusions or preferences).
    pub constraints: RoutingConstraints,
    /// User-facing notification message (if any).
    pub notification: Option<String>,
    /// Whether routing should be blocked entirely (no suitable provider).
    pub blocked: bool,
    /// Block reason (if blocked).
    pub block_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Keywords and file patterns for auto-classification
// ---------------------------------------------------------------------------

/// Keywords in task title/description that trigger Restricted classification.
const RESTRICTED_KEYWORDS: &[&str] = &[
    ".env",
    "secrets",
    "credentials",
    "api_key",
    "api-key",
    "apikey",
    "password",
    "token",
    "private_key",
    "private-key",
    "ssh_key",
    "ssh-key",
    "secret_key",
    "secret-key",
    "access_key",
    "access-key",
];

/// File patterns that trigger Restricted classification.
const RESTRICTED_FILE_PATTERNS: &[&str] = &[
    ".env",
    ".env.",
    "secret",
    ".pem",
    ".key",
    ".p12",
    ".pfx",
    "credential",
    "id_rsa",
    "id_ed25519",
];

/// Keywords in task title/description that trigger Sensitive classification.
const SENSITIVE_KEYWORDS: &[&str] = &[
    "auth",
    "authentication",
    "authorization",
    "login",
    "session",
    "jwt",
    "oauth",
    "encryption",
    "decrypt",
    "cipher",
    "hash",
    "bcrypt",
    "argon2",
    "rbac",
    "permission",
    "access control",
];

/// File patterns that trigger Sensitive classification.
const SENSITIVE_FILE_PATTERNS: &[&str] = &[
    "auth",
    "login",
    "session",
    "security",
    "permission",
    "middleware/auth",
    "guard",
];

// ---------------------------------------------------------------------------
// Classification logic
// ---------------------------------------------------------------------------

/// Classify a task based on its title, description, and affected files.
///
/// This is called automatically during task decomposition — users never
/// need to set classifications manually.
pub fn classify_task(
    title: &str,
    description: &str,
    affected_files: &[String],
) -> SecurityClassification {
    let text = format!("{} {}", title, description).to_lowercase();

    // Check restricted keywords first (higher priority)
    for keyword in RESTRICTED_KEYWORDS {
        if text.contains(keyword) {
            return SecurityClassification::Restricted;
        }
    }

    // Check restricted file patterns
    for file in affected_files {
        let lower = file.to_lowercase();
        for pattern in RESTRICTED_FILE_PATTERNS {
            if lower.contains(pattern) {
                return SecurityClassification::Restricted;
            }
        }
    }

    // Check sensitive keywords
    for keyword in SENSITIVE_KEYWORDS {
        if text.contains(keyword) {
            return SecurityClassification::Sensitive;
        }
    }

    // Check sensitive file patterns
    for file in affected_files {
        let lower = file.to_lowercase();
        for pattern in SENSITIVE_FILE_PATTERNS {
            if lower.contains(pattern) {
                return SecurityClassification::Sensitive;
            }
        }
    }

    SecurityClassification::Normal
}

/// Check if a provider is local based on its declared capabilities.
///
/// Uses the `supports_local_only` capability field rather than ID-based heuristics.
/// Prefer this over `is_local_provider()` when capabilities are available.
pub fn is_local_by_capabilities(capabilities: &crate::types::ProviderCapabilities) -> bool {
    capabilities.supports_local_only
}

/// Check if a provider is local (runs on the user's machine, no cloud API calls).
pub fn is_local_provider(provider_id: &ProviderId) -> bool {
    let id = provider_id.as_str();
    matches!(
        id,
        "ollama" | "deepseek" | "qwen" | "opencode" | "custom-cli"
    ) || id.starts_with("local-")
        || id.starts_with("ollama-")
}

/// Apply security routing to constraints based on classification and mode.
///
/// This modifies routing constraints to enforce security policies without
/// the user needing to configure anything manually.
pub fn apply_security_routing(
    classification: SecurityClassification,
    mode: SecurityRoutingMode,
    base_constraints: &RoutingConstraints,
    local_provider_ids: &[ProviderId],
) -> SecurityRoutingResult {
    match mode {
        SecurityRoutingMode::Disabled => SecurityRoutingResult {
            classification,
            constraints: base_constraints.clone(),
            notification: None,
            blocked: false,
            block_reason: None,
        },

        SecurityRoutingMode::AllLocal => {
            if local_provider_ids.is_empty() {
                return SecurityRoutingResult {
                    classification,
                    constraints: base_constraints.clone(),
                    notification: None,
                    blocked: true,
                    block_reason: Some(
                        "Security routing is set to 'All Local' but no local providers \
                         are configured. Install Ollama or another local provider."
                            .to_string(),
                    ),
                };
            }
            let mut constraints = base_constraints.clone();
            constraints.max_cost_tier = Some(CostTier::Free);
            if let Some(first_local) = local_provider_ids.first() {
                constraints.preferred_provider = Some(first_local.clone());
            }
            SecurityRoutingResult {
                classification,
                constraints,
                notification: None,
                blocked: false,
                block_reason: None,
            }
        }

        SecurityRoutingMode::Automatic => match classification {
            SecurityClassification::Normal => SecurityRoutingResult {
                classification,
                constraints: base_constraints.clone(),
                notification: None,
                blocked: false,
                block_reason: None,
            },

            SecurityClassification::Restricted => {
                if local_provider_ids.is_empty() {
                    SecurityRoutingResult {
                        classification,
                        constraints: base_constraints.clone(),
                        notification: None,
                        blocked: true,
                        block_reason: Some(
                            "This task involves restricted content (secrets/credentials). \
                             It requires a local AI provider like Ollama. Would you like \
                             to skip this task or configure a local provider?"
                                .to_string(),
                        ),
                    }
                } else {
                    let mut constraints = base_constraints.clone();
                    constraints.max_cost_tier = Some(CostTier::Free);
                    if let Some(first_local) = local_provider_ids.first() {
                        constraints.preferred_provider = Some(first_local.clone());
                    }
                    SecurityRoutingResult {
                        classification,
                        constraints,
                        notification: None,
                        blocked: false,
                        block_reason: None,
                    }
                }
            }

            SecurityClassification::Sensitive => {
                let mut constraints = base_constraints.clone();
                let notification = if local_provider_ids.is_empty() {
                    Some(
                        "This task involves authentication logic. It's being processed \
                         by a cloud provider. For maximum security, configure a local \
                         provider like Ollama."
                            .to_string(),
                    )
                } else {
                    if let Some(first_local) = local_provider_ids.first() {
                        constraints.preferred_provider = Some(first_local.clone());
                    }
                    None
                };
                SecurityRoutingResult {
                    classification,
                    constraints,
                    notification,
                    blocked: false,
                    block_reason: None,
                }
            }
        },
    }
}
