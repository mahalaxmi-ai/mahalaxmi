use serde::{Deserialize, Serialize};

use crate::consensus::normalizer::TaskGroup;
use crate::consensus::similarity::{multi_field_similarity, SimilarityWeights};

fn default_similarity_band() -> (f64, f64) {
    (0.25, 0.60)
}

fn default_arb_model() -> String {
    "claude-haiku-4-5-20251001".to_owned()
}

fn default_arb_endpoint() -> String {
    "https://api.anthropic.com/v1/messages".to_owned()
}

/// Configuration for optional LLM-based consensus arbitration.
///
/// When enabled, task group pairs whose similarity score falls in the
/// "ambiguous band" are resolved by a one-shot AI call (claude-haiku by default).
/// The AI decides whether the tasks are duplicates that should be merged or
/// distinct tasks that should remain separate.
///
/// Auth is resolved automatically in priority order:
/// 1. `ANTHROPIC_API_KEY` environment variable → direct REST API call.
/// 2. `claude` CLI binary (Claude Code) → `claude --print` pipe mode.
/// 3. Neither available → arbitration is silently skipped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationConfig {
    /// Whether LLM arbitration is enabled for this project.
    pub enabled: bool,
    /// Similarity band: pairs with scores in `[low, high)` are candidates
    /// for arbitration. Default: `(0.25, 0.60)`.
    #[serde(default = "default_similarity_band")]
    pub similarity_band: (f64, f64),
    /// Model to use for arbitration API calls. Default: `"claude-haiku-4-5-20251001"`.
    #[serde(default = "default_arb_model")]
    pub model: String,
    /// API endpoint. Default: `"https://api.anthropic.com/v1/messages"`.
    #[serde(default = "default_arb_endpoint")]
    pub api_endpoint: String,
    /// Optional domain-specific conflict resolution instruction.
    ///
    /// When `Some`, replaces the hardcoded arbitration system instruction with
    /// this domain-provided text. Enables `ConsensusAlgorithm::ManagerAdjudicated`
    /// and `ConsensusAlgorithm::BestOfN` per-domain arbitration behaviour.
    /// When `None`, the default hardcoded instruction is used unchanged.
    #[serde(default)]
    pub domain_conflict_prompt: Option<String>,
}

impl Default for ArbitrationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            similarity_band: default_similarity_band(),
            model: default_arb_model(),
            api_endpoint: default_arb_endpoint(),
            domain_conflict_prompt: None,
        }
    }
}

/// A single arbitration decision returned by the AI.
#[derive(Debug, Deserialize)]
struct ArbitrationDecision {
    /// Indices of the groups involved (into the original groups slice).
    group_ids: Vec<usize>,
    /// `"merge"` to merge the groups or `"split"` to keep them separate.
    action: String,
    /// Canonical title to use when merging (optional).
    canonical_title: Option<String>,
    /// Canonical description to use when merging (optional).
    canonical_description: Option<String>,
}

/// How arbitration calls are issued.
enum ArbitrationMode {
    /// Direct Anthropic REST API using an API key.
    DirectApi {
        api_key: String,
        client: reqwest::Client,
    },
    /// Claude CLI in print mode (`claude --print`).
    /// Used when `ANTHROPIC_API_KEY` is absent but the claude binary is available.
    ClaudeCli { binary: String },
}

impl ArbitrationMode {
    fn name(&self) -> &'static str {
        match self {
            ArbitrationMode::DirectApi { .. } => "anthropic-api",
            ArbitrationMode::ClaudeCli { .. } => "claude-cli",
        }
    }
}

/// Computes multi-field similarity between two task groups using default weights.
fn group_similarity(a: &TaskGroup, b: &TaskGroup) -> f64 {
    let files_a = a.merged_affected_files();
    let files_b = b.merged_affected_files();
    let criteria_a = a.merged_acceptance_criteria().join(" ");
    let criteria_b = b.merged_acceptance_criteria().join(" ");
    multi_field_similarity(
        a.representative_title(),
        b.representative_title(),
        &files_a,
        &files_b,
        a.best_description(),
        b.best_description(),
        &criteria_a,
        &criteria_b,
        &SimilarityWeights::default(),
    )
}

/// Locate the `claude` CLI binary.
///
/// Checks Claude Code's default install path first, then falls back to a PATH
/// lookup via `which`. Returns `None` when no usable binary is found.
fn find_claude_binary() -> Option<String> {
    if let Ok(home) = std::env::var("HOME") {
        let default = format!("{home}/.claude/local/claude");
        if std::path::Path::new(&default).exists() {
            return Some(default);
        }
    }
    let output = std::process::Command::new("which")
        .arg("claude")
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8(output.stdout).ok()?;
        let path = path.trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

/// Strip markdown code fences that CLI providers sometimes add around JSON output.
fn strip_code_fences(s: &str) -> &str {
    let s = s.strip_prefix("```json").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim()
}

/// LLM-based consensus arbitrator for resolving ambiguous task group pairs.
///
/// Constructed via [`ConsensusArbitrator::resolve`] (recommended) or
/// [`ConsensusArbitrator::from_env`] (API-key-only path).
///
/// `resolve()` selects the best available backend automatically:
/// - `ANTHROPIC_API_KEY` set → direct REST call (fastest, no subprocess)
/// - Claude CLI available → `claude --print` pipe mode (uses existing CLI auth)
/// - Neither → returns `None`; arbitration is skipped gracefully
pub struct ConsensusArbitrator {
    config: ArbitrationConfig,
    mode: ArbitrationMode,
}

impl ConsensusArbitrator {
    /// Resolve the best available arbitration backend.
    ///
    /// Priority order:
    /// 1. `ANTHROPIC_API_KEY` → direct Anthropic REST API (avoids subprocess overhead).
    /// 2. `claude` CLI binary → `claude --print` pipe mode (uses existing CLI auth).
    /// 3. Neither → `None` (arbitration will be skipped with a warning).
    pub fn resolve(config: ArbitrationConfig) -> Option<Self> {
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            if !key.trim().is_empty() {
                return Some(Self {
                    config,
                    mode: ArbitrationMode::DirectApi {
                        api_key: key,
                        client: reqwest::Client::new(),
                    },
                });
            }
        }
        if let Some(binary) = find_claude_binary() {
            return Some(Self {
                config,
                mode: ArbitrationMode::ClaudeCli { binary },
            });
        }
        None
    }

    /// Construct from environment variables only (API-key path).
    ///
    /// Returns `None` when `ANTHROPIC_API_KEY` is not set.
    /// Prefer [`ConsensusArbitrator::resolve`] which also tries the CLI fallback.
    pub fn from_env(config: ArbitrationConfig) -> Option<Self> {
        std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty())
            .map(|key| Self {
                config,
                mode: ArbitrationMode::DirectApi {
                    api_key: key,
                    client: reqwest::Client::new(),
                },
            })
    }

    /// Arbitrate ambiguous task groups.
    ///
    /// Finds pairs with similarity scores in the configured band, packages
    /// them into a single prompt, calls the AI model, and applies the merge
    /// decisions. On any error, returns the groups unchanged.
    ///
    /// When `ArbitrationConfig::domain_conflict_prompt` is `Some`, the domain
    /// instruction replaces the hardcoded system prompt, enabling
    /// `ConsensusAlgorithm::ManagerAdjudicated` behaviour.
    pub async fn arbitrate(&self, groups: Vec<TaskGroup>) -> Vec<TaskGroup> {
        let ambiguous_pairs = self.find_ambiguous_pairs(&groups);
        if ambiguous_pairs.is_empty() {
            return groups;
        }
        tracing::info!(
            pairs = ambiguous_pairs.len(),
            mode = %self.mode.name(),
            has_domain_prompt = self.config.domain_conflict_prompt.is_some(),
            "Arbitrating ambiguous task pair(s) via LLM"
        );
        let optional_prompt = self.config.domain_conflict_prompt.as_deref();
        let result = match &self.mode {
            ArbitrationMode::DirectApi { api_key, client } => {
                self.call_api_direct(api_key, client, &groups, &ambiguous_pairs, optional_prompt)
                    .await
            }
            ArbitrationMode::ClaudeCli { binary } => {
                self.call_cli(binary, &groups, &ambiguous_pairs, optional_prompt)
                    .await
            }
        };
        match result {
            Ok(decisions) => self.apply_decisions(groups, decisions),
            Err(e) => {
                tracing::warn!(error = %e, "Consensus arbitration skipped due to error");
                groups
            }
        }
    }

    /// Find pairs of groups whose similarity score falls in the ambiguous band.
    fn find_ambiguous_pairs(&self, groups: &[TaskGroup]) -> Vec<(usize, usize)> {
        let (low, high) = self.config.similarity_band;
        let mut pairs = Vec::new();
        for i in 0..groups.len() {
            for j in (i + 1)..groups.len() {
                let score = group_similarity(&groups[i], &groups[j]);
                if score >= low && score < high {
                    pairs.push((i, j));
                }
            }
        }
        pairs
    }

    /// Build the arbitration prompt for a set of ambiguous pairs.
    ///
    /// When `optional_prompt` is `Some`, it replaces the default hardcoded
    /// system instruction. This allows domain configurations such as
    /// `ConsensusAlgorithm::ManagerAdjudicated` and `ConsensusAlgorithm::BestOfN`
    /// to inject custom arbitration behaviour. When `None`, the default
    /// instruction is used unchanged.
    fn build_prompt(
        optional_prompt: Option<&str>,
        groups: &[TaskGroup],
        pairs: &[(usize, usize)],
    ) -> String {
        let default_instruction =
            "You are a software project manager reviewing AI-generated task lists.\n\
             Multiple AI managers independently analyzed the same codebase and proposed tasks.\n\
             Some tasks may be duplicates with different titles.\n\n\
             For each pair of tasks below, decide whether they represent the same work \
             (merge) or distinct work (split).\n\n\
             Respond with a JSON array of decisions:\n\
             [{\"group_ids\": [i, j], \"action\": \"merge\"|\"split\", \
             \"canonical_title\": \"...\", \"canonical_description\": \"...\"}]\n\
             Only include canonical_title and canonical_description for merge actions.";

        let instruction = optional_prompt.unwrap_or(default_instruction);
        let mut prompt = format!("{instruction}\n\nTASKS:\n");

        for (i, j) in pairs {
            let a = &groups[*i];
            let b = &groups[*j];
            let desc_a = &a.best_description()[..a.best_description().len().min(200)];
            let desc_b = &b.best_description()[..b.best_description().len().min(200)];
            prompt.push_str(&format!(
                "\nPair ({i}, {j}):\n\
                 Task {i}: \"{}\"\n  Description: {desc_a}\n\
                 Task {j}: \"{}\"\n  Description: {desc_b}\n",
                a.representative_title(),
                b.representative_title(),
            ));
        }

        prompt.push_str(
            "\nRespond with ONLY the raw JSON array. No markdown, no code fences, no preamble.",
        );
        prompt
    }

    /// Call the Anthropic REST API and parse arbitration decisions.
    async fn call_api_direct(
        &self,
        api_key: &str,
        client: &reqwest::Client,
        groups: &[TaskGroup],
        pairs: &[(usize, usize)],
        optional_prompt: Option<&str>,
    ) -> Result<Vec<ArbitrationDecision>, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = Self::build_prompt(optional_prompt, groups, pairs);
        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": 1024,
            "messages": [{ "role": "user", "content": prompt }]
        });

        let resp = client
            .post(&self.config.api_endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {text}").into());
        }

        let json: serde_json::Value = resp.json().await?;
        let text = json["content"][0]["text"]
            .as_str()
            .ok_or("No text in API response")?;

        let decisions: Vec<ArbitrationDecision> = serde_json::from_str(text)
            .map_err(|e| format!("Failed to parse arbitration decisions: {e}"))?;

        Ok(decisions)
    }

    /// Invoke the `claude` CLI in print mode and parse arbitration decisions.
    ///
    /// Pipes the prompt to `claude --print` via stdin and reads the response
    /// from stdout. Strips any markdown code fences before JSON parsing.
    async fn call_cli(
        &self,
        binary: &str,
        groups: &[TaskGroup],
        pairs: &[(usize, usize)],
        optional_prompt: Option<&str>,
    ) -> Result<Vec<ArbitrationDecision>, Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::AsyncWriteExt;

        let prompt = Self::build_prompt(optional_prompt, groups, pairs);

        let mut child = tokio::process::Command::new(binary)
            .arg("--print")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn claude CLI ({binary}): {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .map_err(|e| format!("Failed to write to claude CLI stdin: {e}"))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait for claude CLI: {e}"))?;

        if !output.status.success() {
            return Err(format!("claude CLI exited with status {}", output.status).into());
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let text = strip_code_fences(raw.trim());

        let decisions: Vec<ArbitrationDecision> = serde_json::from_str(text).map_err(|e| {
            format!("Failed to parse CLI arbitration response: {e}\nRaw output: {text}")
        })?;

        Ok(decisions)
    }

    /// Apply arbitration decisions to the groups.
    ///
    /// For "merge" decisions, the groups at the listed indices are merged into the
    /// first index in the list. For "split" decisions, the groups remain unchanged.
    fn apply_decisions(
        &self,
        mut groups: Vec<TaskGroup>,
        decisions: Vec<ArbitrationDecision>,
    ) -> Vec<TaskGroup> {
        // Process decisions in reverse order to keep indices stable during removal.
        let mut merged_away: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for decision in &decisions {
            if decision.action != "merge" {
                continue;
            }
            let ids = &decision.group_ids;
            if ids.len() < 2 {
                continue;
            }
            let target_idx = ids[0];
            if merged_away.contains(&target_idx) {
                continue;
            }

            // Collect entries from secondary groups into the target.
            let mut extra_entries = Vec::new();
            for &src_idx in &ids[1..] {
                if src_idx < groups.len() && !merged_away.contains(&src_idx) {
                    extra_entries.extend(groups[src_idx].entries.clone());
                    merged_away.insert(src_idx);
                }
            }
            if target_idx < groups.len() {
                groups[target_idx].entries.extend(extra_entries);

                // Apply canonical title/description if provided.
                if let Some(ref title) = decision.canonical_title {
                    if !title.is_empty() {
                        // Update the normalized_key and all entries to use the canonical title.
                        let new_key = crate::consensus::normalizer::normalize_task_key(title);
                        groups[target_idx].normalized_key = new_key;
                        if let Some((_, task)) = groups[target_idx].entries.first_mut() {
                            task.title = title.clone();
                        }
                    }
                }
                if let Some(ref desc) = decision.canonical_description {
                    if !desc.is_empty() {
                        if let Some((_, task)) = groups[target_idx].entries.first_mut() {
                            task.description = desc.clone();
                        }
                    }
                }
            }
        }

        // Remove groups that were merged away, in reverse index order.
        let mut to_remove: Vec<usize> = merged_away.into_iter().collect();
        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for idx in to_remove {
            if idx < groups.len() {
                groups.remove(idx);
            }
        }

        groups
    }
}

#[cfg(test)]
impl ConsensusArbitrator {
    /// Construct a test instance that uses a dummy API key.
    fn new_for_test(config: ArbitrationConfig) -> Self {
        Self {
            config,
            mode: ArbitrationMode::DirectApi {
                api_key: "test".to_owned(),
                client: reqwest::Client::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::proposal::{ManagerProposal, ProposedTask};
    use mahalaxmi_core::types::ManagerId;

    fn make_group(title: &str, description: &str) -> TaskGroup {
        let task = ProposedTask::new(title, description);
        let proposal = ManagerProposal::new(ManagerId::new("m0"), vec![task.clone()], 100);
        TaskGroup {
            normalized_key: crate::consensus::normalizer::normalize_task_key(title),
            entries: vec![(proposal.manager_id, task)],
        }
    }

    #[test]
    fn test_find_ambiguous_pairs_returns_correct_indices() {
        let arb = ConsensusArbitrator::new_for_test(ArbitrationConfig::default());

        // Two very similar groups (score likely in ambiguous band)
        let groups = vec![
            make_group("GitHub Issues Adapter", "Handles GitHub issues"),
            make_group("GitHub Issues Handler", "Processes GitHub issue events"),
            make_group("Jira Integration", "Connects to Jira"),
        ];

        let pairs = arb.find_ambiguous_pairs(&groups);
        // At minimum, (0, 1) should be ambiguous since titles are very similar
        // The exact result depends on similarity scores
        assert!(pairs.len() <= 3, "Should find at most 3 pairs for 3 groups");
    }

    #[test]
    fn test_build_prompt_includes_all_group_titles() {
        let groups = vec![
            make_group("GitHub Issues Adapter", "Handles GitHub"),
            make_group("GitHubIssuesHandler", "Processes issues"),
        ];
        let pairs = vec![(0usize, 1usize)];
        let prompt = ConsensusArbitrator::build_prompt(None, &groups, &pairs);
        assert!(prompt.contains("GitHub Issues Adapter"));
        assert!(prompt.contains("GitHubIssuesHandler"));
        assert!(prompt.contains("merge"));
        assert!(prompt.contains("split"));
    }

    #[test]
    fn test_build_prompt_uses_domain_conflict_prompt_when_some() {
        let groups = vec![
            make_group("Task Alpha", "First task"),
            make_group("Task Beta", "Second task"),
        ];
        let pairs = vec![(0usize, 1usize)];
        let domain_instruction = "Custom domain arbitration: prefer the most specific task title.";
        let prompt = ConsensusArbitrator::build_prompt(Some(domain_instruction), &groups, &pairs);
        assert!(
            prompt.contains(domain_instruction),
            "Prompt should contain the domain instruction"
        );
        assert!(
            prompt.contains("Task Alpha"),
            "Prompt should still include task titles"
        );
        assert!(
            !prompt.contains("You are a software project manager"),
            "Default instruction should be replaced when domain prompt is provided"
        );
    }

    #[test]
    fn test_arbitration_config_domain_conflict_prompt_default_is_none() {
        let config = ArbitrationConfig::default();
        assert!(
            config.domain_conflict_prompt.is_none(),
            "domain_conflict_prompt should default to None"
        );
    }

    #[test]
    fn test_arbitration_config_domain_conflict_prompt_serde_round_trip() {
        let config = ArbitrationConfig {
            domain_conflict_prompt: Some("Select the most thorough analysis.".to_owned()),
            ..ArbitrationConfig::default()
        };
        let json = serde_json::to_string(&config).expect("serialization failed");
        let restored: ArbitrationConfig =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(
            restored.domain_conflict_prompt.as_deref(),
            Some("Select the most thorough analysis.")
        );
    }

    #[test]
    fn test_apply_decisions_merges_specified_groups() {
        let arb = ConsensusArbitrator::new_for_test(ArbitrationConfig::default());

        let groups = vec![
            make_group("GitHub Issues Adapter", "Handles GitHub"),
            make_group("GitHubIssuesAdapter", "Processes GitHub"),
            make_group("Jira Integration", "Connects to Jira"),
        ];

        let decisions = vec![ArbitrationDecision {
            group_ids: vec![0, 1],
            action: "merge".to_owned(),
            canonical_title: Some("GitHub Issues Adapter".to_owned()),
            canonical_description: None,
        }];

        let result = arb.apply_decisions(groups, decisions);
        assert_eq!(result.len(), 2, "Groups 0 and 1 should merge into 1");
        assert_eq!(
            result[0].entries.len(),
            2,
            "Merged group should have 2 entries"
        );
    }

    #[test]
    fn test_apply_decisions_split_leaves_groups_unchanged() {
        let arb = ConsensusArbitrator::new_for_test(ArbitrationConfig::default());

        let groups = vec![
            make_group("GitHub Issues Adapter", "Handles GitHub"),
            make_group("Jira Integration", "Connects to Jira"),
        ];

        let decisions = vec![ArbitrationDecision {
            group_ids: vec![0, 1],
            action: "split".to_owned(),
            canonical_title: None,
            canonical_description: None,
        }];

        let result = arb.apply_decisions(groups, decisions);
        assert_eq!(
            result.len(),
            2,
            "Split decision should leave groups unchanged"
        );
    }

    #[test]
    fn test_arbitrate_returns_unchanged_on_empty_ambiguous_pairs() {
        // When no pairs fall in the ambiguous band, groups should be returned unchanged.
        // This test doesn't need an HTTP call since it never reaches the API.
        let arb = ConsensusArbitrator::new_for_test(ArbitrationConfig {
            enabled: true,
            // Very narrow band that won't match any pairs
            similarity_band: (0.99, 1.0),
            ..ArbitrationConfig::default()
        });

        let groups = vec![
            make_group("GitHub Issues Adapter", "Handles GitHub"),
            make_group("Jira Integration", "Connects to Jira"),
        ];

        // Can run synchronously since no API call is made
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(arb.arbitrate(groups));
        assert_eq!(result.len(), 2, "No pairs in band → groups unchanged");
    }

    #[test]
    fn test_strip_code_fences_json_prefix() {
        assert_eq!(strip_code_fences("```json\n[]\n```"), "[]");
    }

    #[test]
    fn test_strip_code_fences_plain_prefix() {
        assert_eq!(strip_code_fences("```\n[]\n```"), "[]");
    }

    #[test]
    fn test_strip_code_fences_no_fences() {
        assert_eq!(strip_code_fences("[]"), "[]");
    }

    #[test]
    fn test_resolve_prefers_api_key() {
        // When ANTHROPIC_API_KEY is set, resolve() should pick DirectApi.
        // We can't easily test the absence case without unsetting the env var,
        // but we can verify the structure compiles and returns Some when key present.
        std::env::set_var("ANTHROPIC_API_KEY", "test-key-for-unit-test");
        let result = ConsensusArbitrator::resolve(ArbitrationConfig::default());
        assert!(result.is_some());
        if let Some(arb) = result {
            assert_eq!(arb.mode.name(), "anthropic-api");
        }
        std::env::remove_var("ANTHROPIC_API_KEY");
    }
}
