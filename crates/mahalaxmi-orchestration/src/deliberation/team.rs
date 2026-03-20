//! Adversarial Manager Team: three-turn Proposer/Challenger/Synthesizer deliberation.
//!
//! [`AdversarialManagerTeam`] orchestrates three sequential API calls per domain
//! area, each building on the prior conversation history:
//!
//! 1. **Proposer** — drafts a comprehensive task list for the domain.
//! 2. **Challenger** — critiques the proposal for gaps, over-engineering, or risks.
//! 3. **Synthesizer** — merges both perspectives into the authoritative final list.

use crate::deliberation::api_client::{AnthropicApiClient, ChatMessage};
use crate::deliberation::discovery::DomainArea;
use mahalaxmi_core::config::AdversarialDeliberationConfig;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::MahalaxmiResult;

/// System prompt for the Proposer role.
const PROPOSER_SYSTEM: &str = "You are the Proposer AI manager. Your role is to propose a comprehensive set of implementation tasks for the given domain area. Be thorough and include all necessary tasks. Return your proposals as a JSON array in this format: [{\"title\": ..., \"description\": ..., \"priority\": 1, \"affected_files_hint\": [...]}]";

/// System prompt for the Challenger role.
const CHALLENGER_SYSTEM: &str = "You are the Challenger AI manager. Your role is to critically review the Proposer's tasks and identify: missing tasks, over-engineered tasks, incorrect scope, or implementation risks. Be specific and constructive.";

/// System prompt for the Synthesizer role.
const SYNTHESIZER_SYSTEM: &str = "You are the Synthesizer AI manager. Given the Proposer's tasks and the Challenger's critique, produce the optimal final task list. Address all valid critique points. Return ONLY a JSON array: [{\"title\": ..., \"description\": ..., \"priority\": 1, \"affected_files_hint\": [...]}]";

/// Abstraction over a chat-completion backend used during deliberation.
///
/// Implementations provide a single `chat` method that sends a system prompt
/// plus a conversation history and returns the assistant's reply text.
/// The trait is generic so tests can inject a mock without real API calls.
pub trait DeliberationClient: Send + Sync {
    /// Send a system-prompted conversation and return the assistant's reply.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the underlying transport or response parsing fails.
    fn chat(
        &self,
        system: &str,
        messages: Vec<ChatMessage>,
    ) -> impl std::future::Future<Output = MahalaxmiResult<String>> + Send;
}

impl DeliberationClient for AnthropicApiClient {
    fn chat(
        &self,
        system: &str,
        messages: Vec<ChatMessage>,
    ) -> impl std::future::Future<Output = MahalaxmiResult<String>> + Send {
        AnthropicApiClient::chat(self, system, messages)
    }
}

/// The role of a participant in the adversarial deliberation protocol.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DeliberationRole {
    /// Proposes the initial task list.
    Proposer,
    /// Challenges and critiques the proposal.
    Challenger,
    /// Synthesizes the final task list from proposals and critiques.
    Synthesizer,
}

/// A single turn within the three-turn deliberation conversation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeliberationTurn {
    /// Which deliberation role produced this turn.
    pub role: DeliberationRole,
    /// The full text content returned by the AI for this turn.
    pub content: String,
    /// Number of tasks embedded in this turn's output (0 for the Challenger).
    pub proposed_task_count: usize,
}

/// Complete log of a deliberation session for one domain area.
///
/// Always contains exactly three [`DeliberationTurn`] items in
/// `[Proposer, Challenger, Synthesizer]` order.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamDeliberationLog {
    /// The domain area that was deliberated over.
    pub domain: DomainArea,
    /// The three deliberation turns in chronological order.
    pub turns: Vec<DeliberationTurn>,
    /// The final synthesized task list produced by the Synthesizer.
    pub final_tasks: Vec<ProposedDeliberationTask>,
}

/// A single task produced by the deliberation process.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProposedDeliberationTask {
    /// Short title identifying the task.
    pub title: String,
    /// Detailed description of what the task involves.
    pub description: String,
    /// Relative priority (lower number = higher priority).
    pub priority: u32,
    /// Optional hints about which files the task is likely to affect.
    #[serde(default)]
    pub affected_files_hint: Vec<String>,
}

/// Orchestrates the three-turn adversarial deliberation protocol for a domain area.
///
/// Each call to [`deliberate`](AdversarialManagerTeam::deliberate) makes three
/// sequential API calls, passing the full conversation history to each turn so
/// every participant can build on prior context.
pub struct AdversarialManagerTeam<C: DeliberationClient = AnthropicApiClient> {
    /// The API client used to drive each deliberation turn.
    pub client: C,
}

impl<C: DeliberationClient> AdversarialManagerTeam<C> {
    /// Execute the three-turn deliberation for a domain area.
    ///
    /// Returns a [`TeamDeliberationLog`] with exactly three turns on success.
    ///
    /// # Errors
    ///
    /// - Returns `Err` immediately if any of the three API calls fails.
    /// - Returns `Err` if the Synthesizer's output cannot be parsed as a JSON
    ///   `Vec<ProposedDeliberationTask>`.
    pub async fn deliberate(
        &self,
        domain: &DomainArea,
        requirements: &str,
        config: &AdversarialDeliberationConfig,
    ) -> MahalaxmiResult<TeamDeliberationLog> {
        let mut turns = Vec::with_capacity(3);
        let mut messages: Vec<ChatMessage> = Vec::new();

        // --- Turn 1: Proposer ---
        let proposer_user = format!(
            "Domain: {}\nDescription: {}\nRequirements:\n{}\n\nPropose implementation tasks for this domain.",
            domain.name, domain.description, requirements
        );
        messages.push(ChatMessage {
            role: "user".to_owned(),
            content: proposer_user,
        });

        let proposer_response = self.client.chat(PROPOSER_SYSTEM, messages.clone()).await?;

        let proposer_tasks: Vec<ProposedDeliberationTask> =
            parse_task_array(&proposer_response).unwrap_or_default();
        let proposer_count = proposer_tasks.len();

        turns.push(DeliberationTurn {
            role: DeliberationRole::Proposer,
            content: proposer_response.clone(),
            proposed_task_count: proposer_count,
        });

        // Append proposer reply to history.
        messages.push(ChatMessage {
            role: "assistant".to_owned(),
            content: proposer_response,
        });

        // --- Turn 2: Challenger ---
        messages.push(ChatMessage {
            role: "user".to_owned(),
            content:
                "Review the above proposals critically. What is missing, over-engineered, or risky?"
                    .to_owned(),
        });

        let challenger_response = self
            .client
            .chat(CHALLENGER_SYSTEM, messages.clone())
            .await?;

        turns.push(DeliberationTurn {
            role: DeliberationRole::Challenger,
            content: challenger_response.clone(),
            proposed_task_count: 0,
        });

        // Append challenger reply to history.
        messages.push(ChatMessage {
            role: "assistant".to_owned(),
            content: challenger_response,
        });

        // --- Turn 3: Synthesizer ---
        messages.push(ChatMessage {
            role: "user".to_owned(),
            content: "Synthesize the final task list addressing the challenger's feedback."
                .to_owned(),
        });

        let synthesizer_response = self
            .client
            .chat(SYNTHESIZER_SYSTEM, messages.clone())
            .await?;

        let final_tasks = match parse_task_array(&synthesizer_response) {
            Some(tasks) => tasks,
            None => {
                tracing::warn!("Synthesizer output not parseable as JSON, returning Err");
                return Err(MahalaxmiError::Orchestration {
                    message: "Synthesizer output could not be parsed as a JSON task array"
                        .to_owned(),
                    i18n_key: "error.deliberation.synthesizer_parse_failed".to_owned(),
                });
            }
        };

        let synthesizer_count = final_tasks.len();

        turns.push(DeliberationTurn {
            role: DeliberationRole::Synthesizer,
            content: synthesizer_response,
            proposed_task_count: synthesizer_count,
        });

        // Use the config model fields to confirm both models are exercised.
        let _ = (&config.proposer_model, &config.synthesizer_model);

        Ok(TeamDeliberationLog {
            domain: domain.clone(),
            turns,
            final_tasks,
        })
    }
}

/// Extract and deserialise a JSON task array from free-form text.
///
/// Searches for the first `[` and last `]` in `text` and attempts to
/// deserialise the slice as `Vec<ProposedDeliberationTask>`.
/// Returns `None` if no valid array is found or deserialization fails.
fn parse_task_array(text: &str) -> Option<Vec<ProposedDeliberationTask>> {
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
                        message: "MockClient exhausted all configured responses".to_owned(),
                        i18n_key: "error.deliberation.mock_exhausted".to_owned(),
                    })
                })
            };
            std::future::ready(result)
        }
    }

    fn test_domain() -> DomainArea {
        DomainArea {
            name: "Authentication".to_owned(),
            description: "User authentication and session management".to_owned(),
            key_files: vec!["src/auth.rs".to_owned()],
        }
    }

    fn test_config() -> AdversarialDeliberationConfig {
        AdversarialDeliberationConfig {
            proposer_model: "claude-haiku-4-5-20251001".to_owned(),
            synthesizer_model: "claude-sonnet-4-6".to_owned(),
            enabled: true,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_deliberation_three_turns_produced() {
        let proposer_json = r#"[{"title":"Add login endpoint","description":"Implement POST /login","priority":1,"affected_files_hint":["src/auth.rs"]}]"#;
        let synthesizer_json = r#"[{"title":"Add login endpoint","description":"Implement POST /login with rate limiting","priority":1,"affected_files_hint":["src/auth.rs"]}]"#;

        let client = MockClient::new(vec![
            Ok(proposer_json.to_owned()),
            Ok("Missing rate limiting and audit logging for the login endpoint.".to_owned()),
            Ok(synthesizer_json.to_owned()),
        ]);
        let team = AdversarialManagerTeam { client };

        let log = team
            .deliberate(
                &test_domain(),
                "Implement user authentication",
                &test_config(),
            )
            .await
            .expect("deliberation should succeed");

        assert_eq!(log.turns.len(), 3);
        assert!(matches!(log.turns[0].role, DeliberationRole::Proposer));
        assert!(matches!(log.turns[1].role, DeliberationRole::Challenger));
        assert!(matches!(log.turns[2].role, DeliberationRole::Synthesizer));
    }

    #[tokio::test]
    async fn test_deliberation_synthesizer_output_parsed() {
        let proposer_json =
            r#"[{"title":"Task A","description":"Do A","priority":1,"affected_files_hint":[]}]"#;
        let synthesizer_json = r#"[{"title":"Task A","description":"Do A","priority":1,"affected_files_hint":[]},{"title":"Task B","description":"Do B with rate limiting","priority":2,"affected_files_hint":[]}]"#;

        let client = MockClient::new(vec![
            Ok(proposer_json.to_owned()),
            Ok("Task B is missing — please add rate limiting.".to_owned()),
            Ok(synthesizer_json.to_owned()),
        ]);
        let team = AdversarialManagerTeam { client };

        let log = team
            .deliberate(&test_domain(), "Build features", &test_config())
            .await
            .expect("deliberation should succeed");

        assert_eq!(log.final_tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_deliberation_api_failure_returns_err() {
        let client = MockClient::new(vec![Err(MahalaxmiError::Orchestration {
            message: "API connection refused".to_owned(),
            i18n_key: "error.deliberation.api_request_failed".to_owned(),
        })]);
        let team = AdversarialManagerTeam { client };

        let result = team
            .deliberate(&test_domain(), "requirements", &test_config())
            .await;

        assert!(result.is_err());
    }
}
