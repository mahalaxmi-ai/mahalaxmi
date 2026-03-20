//! Cross-team dependency resolution for adversarial deliberation.
//!
//! [`CrossTeamDependencyPass`] performs a single AI call that reviews all
//! domain teams' proposals, identifies conflicts and gaps, and optionally
//! appends seam tasks to the final merged `Vec<ManagerProposal>`.
//!
//! # Graceful Degradation
//!
//! [`CrossTeamDependencyPass::run`] **never returns `Err`**. Any failure
//! (API error, JSON parse failure, empty array, etc.) is logged at `WARN`
//! level and the original proposals are returned unchanged. This preserves
//! the orchestration pipeline even when the cross-team analysis is
//! unavailable.

use crate::deliberation::api_client::{AnthropicApiClient, ChatMessage};
use crate::deliberation::team::{DeliberationClient, TeamDeliberationLog};
use crate::models::proposal::{ManagerProposal, ProposedTask};
use mahalaxmi_core::config::AdversarialDeliberationConfig;
use mahalaxmi_core::types::ManagerId;

/// A single gap or dependency task returned by the cross-team AI analysis.
#[derive(Debug, serde::Deserialize)]
struct CrossTeamTask {
    /// Short title identifying the task.
    title: String,
    /// Detailed description of what the task involves.
    description: String,
    /// Relative priority (lower number = higher priority). Defaults to 50.
    #[serde(default = "default_cross_team_priority")]
    priority: u32,
}

fn default_cross_team_priority() -> u32 {
    50
}

/// Cross-team dependency resolution pass.
///
/// Performs a single Haiku-class API call that reviews all domain teams'
/// final proposals and identifies:
/// - Conflicting tasks that overlap between teams.
/// - Seam gaps where no team owns a shared interface.
/// - Missing dependency tasks that no team has covered.
///
/// On success, if the AI returns any additional tasks, they are collected
/// into a new [`ManagerProposal`] (with
/// `manager_id = "cross-team-dependency-pass"`) and appended to
/// `original_proposals`. On any failure the original proposals are returned
/// unchanged — see the [module docs](self) for the graceful-degradation
/// contract.
pub struct CrossTeamDependencyPass<C: DeliberationClient = AnthropicApiClient> {
    /// The API client used to perform the dependency analysis call.
    ///
    /// Should be constructed using the `discovery_model` from
    /// [`AdversarialDeliberationConfig`] (typically `claude-haiku-4-5-20251001`).
    pub client: C,
}

impl<C: DeliberationClient> CrossTeamDependencyPass<C> {
    /// Run the cross-team dependency analysis against all domain teams' proposals.
    ///
    /// Builds a prompt summarising every team's final tasks, sends it as a
    /// single chat completion request, and parses the response as a JSON
    /// array of additional tasks. Any additional tasks are wrapped in a new
    /// [`ManagerProposal`] and appended to `original_proposals`.
    ///
    /// # Graceful Degradation
    ///
    /// Returns `original_proposals` unchanged when any of the following occur:
    /// - The API call fails.
    /// - The AI returns an empty JSON array (`[]`).
    /// - The response cannot be parsed as a valid JSON array.
    ///
    /// Errors are logged at `WARN` level; none are propagated to the caller.
    pub async fn run(
        &self,
        team_logs: &[TeamDeliberationLog],
        requirements: &str,
        config: &AdversarialDeliberationConfig,
        original_proposals: Vec<ManagerProposal>,
    ) -> Vec<ManagerProposal> {
        tracing::debug!(
            model = %config.discovery_model,
            teams = team_logs.len(),
            "Running cross-team dependency pass"
        );

        let system_prompt = "You are a cross-team dependency resolver. \
            You review proposals from multiple domain teams and identify gaps between them. \
            Return ONLY a raw JSON array of additional tasks to add (may be empty []): \
            [{\"title\": ..., \"description\": ..., \"priority\": 1}]. \
            Do NOT re-list existing tasks. Output raw JSON with no markdown or code fences.";

        let user_content = build_team_summary(team_logs, requirements);
        let messages = vec![ChatMessage {
            role: "user".to_owned(),
            content: user_content,
        }];

        let response = match self.client.chat(system_prompt, messages).await {
            Ok(text) => text,
            Err(err) => {
                tracing::warn!(
                    "CrossTeamDependencyPass failed: {err}; returning original proposals unchanged"
                );
                return original_proposals;
            }
        };

        let additional_tasks: Vec<CrossTeamTask> = match parse_cross_team_tasks(&response) {
            Some(tasks) => tasks,
            None => {
                tracing::warn!(
                    "CrossTeamDependencyPass failed: could not parse JSON task array from response; \
                     returning original proposals unchanged"
                );
                return original_proposals;
            }
        };

        if additional_tasks.is_empty() {
            return original_proposals;
        }

        let proposed_tasks: Vec<ProposedTask> = additional_tasks
            .into_iter()
            .map(|t| ProposedTask::new(t.title, t.description).with_priority(t.priority))
            .collect();

        let cross_team_proposal = ManagerProposal::new(
            ManagerId::new("cross-team-dependency-pass"),
            proposed_tasks,
            0,
        );

        let mut proposals = original_proposals;
        proposals.push(cross_team_proposal);
        proposals
    }
}

/// Build the user message content summarising all teams' proposals.
///
/// Formats each team's final tasks as a bullet list under a `### Domain`
/// heading, then appends the first 2 000 characters of `requirements`.
fn build_team_summary(team_logs: &[TeamDeliberationLog], requirements: &str) -> String {
    let n = team_logs.len();
    let mut summary = format!(
        "You are a cross-team dependency resolver. Below are proposals from {n} domain teams.\n\
         Identify: (1) conflicting tasks that overlap between teams, \
         (2) seam gaps where no team owns a shared interface, \
         (3) missing dependency tasks.\n\
         Return a JSON array of additional tasks to add (may be empty []): \
         [{{\"title\": ..., \"description\": ..., \"priority\": 1}}]\n\
         Do NOT re-list existing tasks.\n\n"
    );

    for log in team_logs {
        summary.push_str(&format!("### {}\n", log.domain.name));
        for task in &log.final_tasks {
            summary.push_str(&format!("- {}: {}\n", task.title, task.description));
        }
        summary.push('\n');
    }

    let req_preview: String = requirements.chars().take(2000).collect();
    summary.push_str(&format!("Requirements summary: {req_preview}"));

    summary
}

/// Extract and deserialize a cross-team task array from free-form AI output.
///
/// Searches for the outermost `[` and `]` in `text` and attempts JSON
/// deserialization as `Vec<CrossTeamTask>`. Returns `None` if no valid
/// array is found or parsing fails.
fn parse_cross_team_tasks(text: &str) -> Option<Vec<CrossTeamTask>> {
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
    use crate::deliberation::discovery::DomainArea;
    use crate::deliberation::team::ProposedDeliberationTask;
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
                        message: "MockClient exhausted all configured responses".to_owned(),
                        i18n_key: "error.deliberation.mock_exhausted".to_owned(),
                    })
                })
            };
            std::future::ready(result)
        }
    }

    fn test_team_log(domain_name: &str) -> TeamDeliberationLog {
        TeamDeliberationLog {
            domain: DomainArea {
                name: domain_name.to_owned(),
                description: format!("{domain_name} domain"),
                key_files: vec![],
            },
            turns: vec![],
            final_tasks: vec![ProposedDeliberationTask {
                title: "Implement core feature".to_owned(),
                description: "Core feature implementation for this domain".to_owned(),
                priority: 1,
                affected_files_hint: vec![],
            }],
        }
    }

    fn test_config() -> AdversarialDeliberationConfig {
        AdversarialDeliberationConfig::default()
    }

    fn test_proposals() -> Vec<ManagerProposal> {
        vec![ManagerProposal::new(
            ManagerId::new("manager-0"),
            vec![ProposedTask::new("Existing task", "Do something existing")],
            100,
        )]
    }

    #[tokio::test]
    async fn test_cross_team_no_conflicts_returns_unchanged() {
        let client = MockClient::new(vec![Ok("[]".to_owned())]);
        let pass = CrossTeamDependencyPass { client };

        let original = test_proposals();
        let original_len = original.len();

        let result = pass
            .run(
                &[test_team_log("Backend")],
                "Build a distributed system",
                &test_config(),
                original,
            )
            .await;

        assert_eq!(
            result.len(),
            original_len,
            "Empty JSON array should leave proposals unchanged"
        );
    }

    #[tokio::test]
    async fn test_cross_team_gap_task_added() {
        let response = r#"[{"title": "API Gateway Integration", "description": "Add a shared API gateway to route requests between backend and frontend teams", "priority": 2}]"#;
        let client = MockClient::new(vec![Ok(response.to_owned())]);
        let pass = CrossTeamDependencyPass { client };

        let original = test_proposals();
        let original_len = original.len();

        let result = pass
            .run(
                &[test_team_log("Backend"), test_team_log("Frontend")],
                "Build a microservices platform",
                &test_config(),
                original,
            )
            .await;

        assert_eq!(
            result.len(),
            original_len + 1,
            "One cross-team proposal should be appended"
        );
        let last = result.last().unwrap();
        assert!(
            last.tasks.iter().any(|t| t.title.contains("API Gateway")),
            "Last proposal should contain the API Gateway Integration task"
        );
    }

    #[tokio::test]
    async fn test_cross_team_api_failure_returns_original() {
        let client = MockClient::new(vec![Err(MahalaxmiError::Orchestration {
            message: "API connection refused".to_owned(),
            i18n_key: "error.deliberation.api_request_failed".to_owned(),
        })]);
        let pass = CrossTeamDependencyPass { client };

        let original = test_proposals();
        let original_len = original.len();

        let result = pass
            .run(
                &[test_team_log("Backend")],
                "Build a system",
                &test_config(),
                original,
            )
            .await;

        assert_eq!(
            result.len(),
            original_len,
            "API failure should return original proposals unchanged without panicking"
        );
    }
}
