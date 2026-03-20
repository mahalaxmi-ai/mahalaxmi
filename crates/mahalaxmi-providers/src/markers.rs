use regex::Regex;
use serde::{Deserialize, Serialize};

/// Patterns used to detect significant events in a provider's terminal output.
///
/// The PTY engine and state detection layer use these markers to determine
/// when a provider has finished processing, encountered an error, or is waiting for input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMarkers {
    /// Regex pattern that indicates the AI has completed its task successfully.
    #[serde(with = "regex_serde")]
    pub completion_marker: Regex,
    /// Regex pattern that indicates the AI has encountered an error.
    #[serde(with = "regex_serde")]
    pub error_marker: Regex,
    /// Regex pattern that indicates the AI is waiting for user input (prompt).
    #[serde(with = "regex_serde")]
    pub prompt_marker: Regex,
}

/// Serde helper for Regex serialization.
mod regex_serde {
    use regex::Regex;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(regex: &Regex, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(regex.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Regex::new(&s).map_err(serde::de::Error::custom)
    }
}

impl OutputMarkers {
    /// Create output markers from string patterns.
    ///
    /// Returns an error if any pattern is not a valid regex.
    pub fn new(completion: &str, error: &str, prompt: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            completion_marker: Regex::new(completion)?,
            error_marker: Regex::new(error)?,
            prompt_marker: Regex::new(prompt)?,
        })
    }
}
