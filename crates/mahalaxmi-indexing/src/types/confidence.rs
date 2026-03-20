//! Confidence scoring for indexed symbols and relevance ranking.
//!
//! Provides a discrete confidence classification used by the ranking
//! and relevance scoring subsystems to categorize how important or
//! reliable an indexed symbol is.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Discrete confidence level bands for indexed symbol relevance.
///
/// Used by the ranking system to classify how strongly a symbol
/// contributes to the codebase understanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    /// Low confidence (0.0 to 0.3).
    Low,
    /// Medium confidence (0.3 to 0.7).
    Medium,
    /// High confidence (0.7 to 1.0).
    High,
}

impl ConfidenceLevel {
    /// Derive a confidence level from a numeric score.
    ///
    /// The score is clamped to the 0.0..=1.0 range before classification:
    /// - 0.0..=0.3 -> Low
    /// - 0.3..=0.7 -> Medium
    /// - 0.7..=1.0 -> High
    pub fn from_score(score: f64) -> Self {
        let clamped = score.clamp(0.0, 1.0);
        if clamped <= 0.3 {
            Self::Low
        } else if clamped <= 0.7 {
            Self::Medium
        } else {
            Self::High
        }
    }

    /// Returns the numeric range for this confidence level as (min, max).
    pub fn as_range(&self) -> (f64, f64) {
        match self {
            Self::Low => (0.0, 0.3),
            Self::Medium => (0.3, 0.7),
            Self::High => (0.7, 1.0),
        }
    }

    /// Returns the level as a static string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_score_bands() {
        assert_eq!(ConfidenceLevel::from_score(0.0), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.3), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.5), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.7), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.8), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(1.0), ConfidenceLevel::High);
    }

    #[test]
    fn from_score_clamps_out_of_range() {
        assert_eq!(ConfidenceLevel::from_score(-0.5), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(1.5), ConfidenceLevel::High);
    }

    #[test]
    fn as_range_covers_full_spectrum() {
        assert_eq!(ConfidenceLevel::Low.as_range(), (0.0, 0.3));
        assert_eq!(ConfidenceLevel::Medium.as_range(), (0.3, 0.7));
        assert_eq!(ConfidenceLevel::High.as_range(), (0.7, 1.0));
    }

    #[test]
    fn display_matches_as_str() {
        for level in [
            ConfidenceLevel::Low,
            ConfidenceLevel::Medium,
            ConfidenceLevel::High,
        ] {
            assert_eq!(format!("{level}"), level.as_str());
        }
    }

    #[test]
    fn serde_roundtrip() {
        for level in [
            ConfidenceLevel::Low,
            ConfidenceLevel::Medium,
            ConfidenceLevel::High,
        ] {
            let json = serde_json::to_string(&level).expect("serialize");
            let deserialized: ConfidenceLevel = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(level, deserialized);
        }
    }
}
