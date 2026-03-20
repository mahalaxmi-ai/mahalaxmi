//! Domain area discovery for adversarial manager deliberation.
//!
//! [`AreaDiscoveryEngine`] makes a single Anthropic API call (using the
//! cheap `discovery_model`, typically Haiku) to partition the submitted
//! requirements into 2–5 non-overlapping domain areas.  Results feed
//! directly into [`AdversarialManagerTeam`](super::team::AdversarialManagerTeam)
//! which runs a separate deliberation team for each area.

use std::sync::Arc;

use mahalaxmi_core::config::AdversarialDeliberationConfig;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::MahalaxmiResult;

use crate::deliberation::api_client::{AnthropicApiClient, ChatMessage};
use crate::deliberation::team::DeliberationClient;

/// A discrete domain area identified for adversarial deliberation.
///
/// Each `DomainArea` represents a coherent slice of the codebase or product
/// (e.g. "Authentication", "Database Layer", "Frontend UI") that should be
/// deliberated over independently by the adversarial manager team.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomainArea {
    /// Short human-readable name (e.g. `"Authentication"`).
    pub name: String,
    /// Brief description of the domain's scope and boundaries.
    pub description: String,
    /// Key files most relevant to this domain (relative project paths).
    ///
    /// Used by the Proposer to ground proposals in actual code locations.
    #[serde(default)]
    pub key_files: Vec<String>,
}

/// System prompt for the area discovery API call.
const DISCOVERY_SYSTEM: &str = "You are analyzing a software project to divide work across \
    specialist teams. Identify 2-5 non-overlapping domains that the submitted work touches. \
    Each domain should be cohesive (files and logic that naturally belong together), \
    non-overlapping (no file belongs to two domains), and complete (every part of the work \
    is covered by exactly one domain). Respond with ONLY valid JSON — no prose, no code \
    fences, no markdown:\n\
    [\n  {\n    \"name\": \"short_domain_name\",\n    \
    \"description\": \"what this domain covers in 1-2 sentences\",\n    \
    \"key_files\": [\"path/to/file1.rs\", \"path/to/file2.ts\"]\n  }\n]";

/// Discovers domain areas from submitted requirements via a single Anthropic API call.
///
/// Uses the `discovery_model` (typically `claude-haiku-4-5-20251001`) for a
/// fast, cheap classification pass before the more expensive deliberation
/// turns.
pub struct AreaDiscoveryEngine<C: DeliberationClient = AnthropicApiClient> {
    /// The API client used for the discovery call.
    pub client: Arc<C>,
}

impl<C: DeliberationClient> AreaDiscoveryEngine<C> {
    /// Create a new discovery engine with the given API client.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Discover 2–5 non-overlapping domain areas for the submitted requirements.
    ///
    /// Makes a single API call using `config.discovery_model`.  Results are
    /// clamped to `config.max_domains` if the model returns more areas than
    /// configured.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the API call fails or the response cannot be parsed.
    /// The caller should fall back to the standard PTY manager path on `Err`.
    pub async fn discover(
        &self,
        requirements: &str,
        codebase_summary: &str,
        config: &AdversarialDeliberationConfig,
    ) -> MahalaxmiResult<Vec<DomainArea>> {
        let user_message = format!(
            "Requirements submitted:\n{requirements}\n\n\
             Codebase summary (top files and modules):\n{codebase_summary}"
        );

        let response = self
            .client
            .chat(
                DISCOVERY_SYSTEM,
                vec![ChatMessage {
                    role: "user".to_owned(),
                    content: user_message,
                }],
            )
            .await?;

        let areas = parse_domain_areas(&response).ok_or_else(|| MahalaxmiError::Provider {
            message: format!(
                "AreaDiscoveryEngine: failed to parse domain areas JSON from response: {response}"
            ),
            i18n_key: "error.deliberation.discovery_parse_failed".to_owned(),
        })?;

        // Clamp to max_domains.
        let clamped: Vec<DomainArea> = areas.into_iter().take(config.max_domains).collect();

        tracing::info!(
            count = clamped.len(),
            max_domains = config.max_domains,
            "Area discovery complete"
        );

        Ok(clamped)
    }
}

/// Parse domain areas from free-form AI output.
///
/// Locates the outermost `[` and `]` in `text` and attempts JSON
/// deserialization as `Vec<DomainArea>`.  Returns `None` if no valid
/// array is found or deserialization fails.
fn parse_domain_areas(text: &str) -> Option<Vec<DomainArea>> {
    let start = text.find('[')?;
    let end = text.rfind(']')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&text[start..=end]).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deliberation::api_client::ChatMessage;
    use mahalaxmi_core::error::MahalaxmiError;
    use mahalaxmi_core::MahalaxmiResult;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// Test double that returns pre-configured responses in FIFO order.
    struct MockClient {
        responses: Mutex<VecDeque<MahalaxmiResult<String>>>,
    }

    impl MockClient {
        fn new(responses: Vec<MahalaxmiResult<String>>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
            }
        }
    }

    impl DeliberationClient for MockClient {
        fn chat(
            &self,
            _system: &str,
            _messages: Vec<ChatMessage>,
        ) -> impl std::future::Future<Output = MahalaxmiResult<String>> + Send {
            let result = {
                let mut queue = self.responses.lock().unwrap();
                queue.pop_front().unwrap_or_else(|| {
                    Err(MahalaxmiError::Orchestration {
                        message: "MockClient exhausted".to_owned(),
                        i18n_key: "error.deliberation.mock_exhausted".to_owned(),
                    })
                })
            };
            std::future::ready(result)
        }
    }

    fn test_config() -> AdversarialDeliberationConfig {
        AdversarialDeliberationConfig {
            enabled: true,
            max_domains: 5,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_discovery_parses_valid_json() {
        let response = r#"[
            {"name": "auth", "description": "User authentication", "key_files": ["src/auth.rs"]},
            {"name": "api", "description": "REST API layer", "key_files": ["src/routes.rs"]},
            {"name": "db", "description": "Database access", "key_files": ["src/db.rs"]}
        ]"#;
        let client = Arc::new(MockClient::new(vec![Ok(response.to_owned())]));
        let engine = AreaDiscoveryEngine::new(client);

        let areas = engine
            .discover("Add user auth and REST API", "Rust workspace", &test_config())
            .await
            .expect("discovery should succeed");

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0].name, "auth");
        assert_eq!(areas[0].key_files, vec!["src/auth.rs"]);
        assert_eq!(areas[1].name, "api");
        assert_eq!(areas[2].name, "db");
    }

    #[tokio::test]
    async fn test_discovery_respects_max_domains() {
        // API returns 8 areas — should be clamped to max_domains = 3.
        let response = r#"[
            {"name": "a", "description": "A", "key_files": []},
            {"name": "b", "description": "B", "key_files": []},
            {"name": "c", "description": "C", "key_files": []},
            {"name": "d", "description": "D", "key_files": []},
            {"name": "e", "description": "E", "key_files": []},
            {"name": "f", "description": "F", "key_files": []},
            {"name": "g", "description": "G", "key_files": []},
            {"name": "h", "description": "H", "key_files": []}
        ]"#;
        let client = Arc::new(MockClient::new(vec![Ok(response.to_owned())]));
        let engine = AreaDiscoveryEngine::new(client);
        let config = AdversarialDeliberationConfig {
            max_domains: 3,
            ..test_config()
        };

        let areas = engine
            .discover("requirements", "summary", &config)
            .await
            .expect("discovery should succeed");

        assert_eq!(areas.len(), 3, "should be clamped to max_domains");
        assert_eq!(areas[0].name, "a");
        assert_eq!(areas[2].name, "c");
    }

    #[tokio::test]
    async fn test_discovery_api_failure_returns_err() {
        let client = Arc::new(MockClient::new(vec![Err(MahalaxmiError::Provider {
            message: "500 Internal Server Error".to_owned(),
            i18n_key: "error.deliberation.api_error_response".to_owned(),
        })]));
        let engine = AreaDiscoveryEngine::new(client);

        let result = engine
            .discover("requirements", "summary", &test_config())
            .await;

        assert!(result.is_err(), "API 500 must propagate as Err");
    }

    #[tokio::test]
    async fn test_discovery_handles_missing_key_files() {
        // key_files defaults to [] when absent from JSON response.
        let response = r#"[
            {"name": "auth", "description": "Authentication domain"}
        ]"#;
        let client = Arc::new(MockClient::new(vec![Ok(response.to_owned())]));
        let engine = AreaDiscoveryEngine::new(client);

        let areas = engine
            .discover("requirements", "summary", &test_config())
            .await
            .expect("discovery with no key_files should succeed");

        assert_eq!(areas.len(), 1);
        assert!(areas[0].key_files.is_empty());
    }
}
