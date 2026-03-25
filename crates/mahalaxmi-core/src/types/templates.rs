use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a template category.
///
/// Categories group related templates together (e.g., "software", "legal", "music").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(String);

impl CategoryId {
    /// Create a new category ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the category ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CategoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a template within a category.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateId(String);

impl TemplateId {
    /// Create a new template ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the template ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TemplateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Difficulty level of a template.
///
/// Used to help users select appropriate templates for their experience level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateDifficulty {
    /// Suitable for beginners.
    Beginner,
    /// Requires some experience.
    Intermediate,
    /// Requires significant experience.
    Advanced,
    /// Expert-level complexity.
    Expert,
    /// Difficulty varies based on user input (e.g., custom templates).
    Variable,
}

impl TemplateDifficulty {
    /// Parse from a string representation.
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "beginner" => Some(Self::Beginner),
            "intermediate" => Some(Self::Intermediate),
            "advanced" => Some(Self::Advanced),
            "expert" => Some(Self::Expert),
            "variable" => Some(Self::Variable),
            _ => None,
        }
    }

    /// Convert to a string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Beginner => "beginner",
            Self::Intermediate => "intermediate",
            Self::Advanced => "advanced",
            Self::Expert => "expert",
            Self::Variable => "variable",
        }
    }
}

impl fmt::Display for TemplateDifficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// License tier determining available features and limits.
///
/// Higher tiers include all capabilities of lower tiers.
/// Ordering: Trial < Basic < Pro < AllAccess < Enterprise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LicenseTier {
    /// Free trial with limited features.
    Trial,
    /// Basic paid tier.
    Basic,
    /// Professional tier with expanded features.
    Pro,
    /// Full access to all categories.
    AllAccess,
    /// Enterprise tier with maximum capacity.
    Enterprise,
}

impl LicenseTier {
    /// Returns true if this tier includes (is at least as high as) the given tier.
    pub fn includes_tier(&self, other: LicenseTier) -> bool {
        *self >= other
    }

    /// Returns the maximum number of concurrent workers for this tier.
    pub fn max_workers(&self) -> u32 {
        match self {
            Self::Trial => 5,
            Self::Basic => 20,
            Self::Pro => 50,
            Self::AllAccess => 100,
            Self::Enterprise => 500,
        }
    }
}

impl fmt::Display for LicenseTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Trial => "Trial",
            Self::Basic => "Basic",
            Self::Pro => "Pro",
            Self::AllAccess => "AllAccess",
            Self::Enterprise => "Enterprise",
        };
        write!(f, "{}", label)
    }
}

/// Severity level for template validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Informational — does not block activation.
    Info,
    /// Warning — activation proceeds but user should review.
    Warning,
    /// Error — blocks template activation.
    Error,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
        };
        write!(f, "{}", label)
    }
}
