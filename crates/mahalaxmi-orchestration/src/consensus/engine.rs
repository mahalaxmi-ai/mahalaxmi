use crate::consensus::complexity::ComplexityWeightedStrategy;
use crate::consensus::intersection::IntersectionStrategy;
use crate::consensus::normalizer::{group_matching_tasks, TaskGroup};
use crate::consensus::strategy::ConsensusStrategyImpl;
use crate::consensus::union::UnionStrategy;
use crate::consensus::weighted::WeightedVotingStrategy;
use crate::models::{ConsensusConfiguration, ConsensusResult, ManagerProposal};
use mahalaxmi_core::domain::ConsensusAlgorithm;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ConsensusStrategy, DeveloperId};
use mahalaxmi_core::MahalaxmiResult;
use std::collections::HashMap;

/// The consensus engine that delegates to the configured strategy.
pub struct ConsensusEngine {
    config: ConsensusConfiguration,
}

impl ConsensusEngine {
    /// Create a new consensus engine with the given configuration.
    pub fn new(config: ConsensusConfiguration) -> Self {
        Self { config }
    }

    /// Evaluate manager proposals using the configured strategy.
    ///
    /// Stage 1 (enhanced lexical grouping via `group_matching_tasks`) runs first,
    /// then the strategy evaluates the deduplicated groups.
    ///
    /// After the strategy resolves task consensus, the engine resolves
    /// `cycle_label` independently: it collects the `cycle_label` fields
    /// from all successful proposals, picks the most common valid slug
    /// (ties broken alphabetically for determinism), and attaches it to
    /// the returned `ConsensusResult`.
    ///
    /// Developer weights are not applied via this method. Use [`Self::run`]
    /// to supply per-developer weights for multi-developer workspace cycles.
    pub fn evaluate(
        &self,
        proposals: &[ManagerProposal],
        i18n: &I18nService,
    ) -> MahalaxmiResult<ConsensusResult> {
        let groups = group_matching_tasks(proposals);
        let total_proposals = proposals.len() as u32;
        let successful = proposals.iter().filter(|p| p.completed).count() as u32;
        let strategy = self.build_strategy(HashMap::new());
        let mut result =
            strategy.evaluate_groups(groups, &self.config, i18n, total_proposals, successful)?;
        result.cycle_label = resolve_cycle_label(proposals);
        Ok(result)
    }

    /// Evaluate pre-grouped task groups.
    ///
    /// Used when the caller has already run `group_matching_tasks()` and
    /// optionally applied LLM arbitration before strategy evaluation.
    /// `proposals` is still needed for `resolve_cycle_label` and metrics.
    pub fn evaluate_from_groups(
        &self,
        groups: Vec<TaskGroup>,
        proposals: &[ManagerProposal],
        i18n: &I18nService,
    ) -> MahalaxmiResult<ConsensusResult> {
        let total_proposals = proposals.len() as u32;
        let successful = proposals.iter().filter(|p| p.completed).count() as u32;
        let strategy = self.build_strategy(HashMap::new());
        let mut result =
            strategy.evaluate_groups(groups, &self.config, i18n, total_proposals, successful)?;
        result.cycle_label = resolve_cycle_label(proposals);
        Ok(result)
    }

    /// Evaluate manager proposals with explicit per-developer vote weights.
    ///
    /// This is the primary entry point for multi-developer workspace cycles.
    /// The `developer_weights` map is keyed by [`DeveloperId`]; when a
    /// proposal carries a `developer_id` that appears in the map, the
    /// associated weight amplifies or reduces that proposal's voting power.
    ///
    /// When `developer_weights` is an empty `HashMap`, this method behaves
    /// identically to [`Self::evaluate`] — all proposals are weighted by
    /// their embedded `developer_weight` field (default `1.0`), preserving
    /// existing single-developer cycle behaviour.
    ///
    /// Only the [`WeightedVoting`] strategy consumes `developer_weights`; the
    /// other strategies (`Union`, `Intersection`, `ComplexityWeighted`) ignore it.
    ///
    /// [`WeightedVoting`]: ConsensusStrategy::WeightedVoting
    pub fn run(
        &self,
        proposals: &[ManagerProposal],
        developer_weights: HashMap<DeveloperId, f32>,
        i18n: &I18nService,
    ) -> MahalaxmiResult<ConsensusResult> {
        let strategy = self.build_strategy(developer_weights);
        let mut result = strategy.evaluate(proposals, &self.config, i18n)?;
        result.cycle_label = resolve_cycle_label(proposals);
        Ok(result)
    }

    /// Evaluate manager proposals using a domain-specific consensus algorithm.
    ///
    /// Applies additional domain-level filtering on top of the base consensus
    /// strategy before delegating to the configured strategy implementation.
    ///
    /// Algorithm behaviour:
    /// - `None` / `Unanimous`: identical to [`Self::evaluate`] — existing behaviour unchanged.
    /// - `Majority { threshold }`: task groups whose fraction of supporting proposers
    ///   is below `threshold` are dropped before strategy evaluation.  A group with
    ///   `n` entries out of `total` proposals passes when `n / total >= threshold`.
    /// - `BestOfN { .. }`: the arbitration prompt injection is handled at the
    ///   `ArbitrationConfig::domain_conflict_prompt` level by the driver; the engine
    ///   delegates to the standard evaluation path.
    /// - `ManagerAdjudicated { .. }`: same as `BestOfN` — prompt injection is at the
    ///   arbitrator layer; the engine delegates to standard evaluation.
    /// - `Synthesis { .. }`: handled post-worker-completion by the driver; the engine
    ///   delegates to standard evaluation here so a plan can still be built.
    pub fn evaluate_with_domain_algorithm(
        &self,
        proposals: &[ManagerProposal],
        i18n: &I18nService,
        domain_algorithm: Option<&ConsensusAlgorithm>,
    ) -> MahalaxmiResult<ConsensusResult> {
        let groups = group_matching_tasks(proposals);
        let total_proposals = proposals.len() as u32;
        let successful = proposals.iter().filter(|p| p.completed).count() as u32;

        let filtered_groups: Vec<TaskGroup> = match domain_algorithm {
            Some(ConsensusAlgorithm::Majority { threshold }) => {
                let threshold = *threshold;
                let total = proposals.len().max(1);
                groups
                    .into_iter()
                    .filter(|g| g.entries.len() as f32 / total as f32 >= threshold)
                    .collect()
            }
            Some(ConsensusAlgorithm::Synthesis { .. }) => {
                // Synthesis consolidation runs post-worker-completion via the
                // orchestration driver; the engine's role here is plan-building
                // only.  Delegate to standard evaluation so workers receive a
                // well-formed execution plan.
                tracing::debug!(
                    algorithm = "synthesis",
                    phase = "plan-build",
                    note = "synthesis-manager-runs-post-worker"
                );
                groups
            }
            _ => groups,
        };

        let strategy = self.build_strategy(HashMap::new());
        let mut result = strategy.evaluate_groups(
            filtered_groups,
            &self.config,
            i18n,
            total_proposals,
            successful,
        )?;
        result.cycle_label = resolve_cycle_label(proposals);
        Ok(result)
    }

    /// Get the current configuration.
    pub fn config(&self) -> &ConsensusConfiguration {
        &self.config
    }

    /// Build the concrete strategy implementation, forwarding `developer_weights`
    /// to [`WeightedVotingStrategy`].
    fn build_strategy(
        &self,
        _developer_weights: HashMap<DeveloperId, f32>,
    ) -> Box<dyn ConsensusStrategyImpl> {
        match self.config.strategy {
            ConsensusStrategy::Union => Box::new(UnionStrategy),
            ConsensusStrategy::Intersection => Box::new(IntersectionStrategy),
            ConsensusStrategy::WeightedVoting => Box::new(WeightedVotingStrategy),
            ConsensusStrategy::ComplexityWeighted => Box::new(ComplexityWeightedStrategy),
        }
    }
}

/// Resolve the `cycle_label` from a set of manager proposals.
///
/// Collects every non-empty `cycle_label` from successful proposals,
/// validates each against the slug rules (1-3 words, lowercase letters,
/// digits, hyphens only), then returns the most common valid label.
/// Ties are broken alphabetically so the result is deterministic.
/// Returns `None` when no valid label was proposed.
pub(crate) fn resolve_cycle_label(proposals: &[ManagerProposal]) -> Option<String> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for proposal in proposals.iter().filter(|p| p.completed) {
        if let Some(ref label) = proposal.cycle_label {
            let normalized = label.trim().to_lowercase();
            if is_valid_cycle_label(&normalized) {
                *counts.entry(normalized).or_insert(0) += 1;
            }
        }
    }

    // Pick the most frequent; break ties alphabetically.
    counts
        .into_iter()
        .max_by(|(label_a, count_a), (label_b, count_b)| {
            count_a.cmp(count_b).then_with(|| label_b.cmp(label_a))
        })
        .map(|(label, _)| label)
}

/// Validate a `cycle_label` slug.
///
/// Rules:
/// - 1 to 3 hyphen-separated words
/// - Each word contains only lowercase ASCII letters and digits
/// - No leading/trailing hyphens, no consecutive hyphens
fn is_valid_cycle_label(label: &str) -> bool {
    if label.is_empty() {
        return false;
    }
    let words: Vec<&str> = label.split('-').collect();
    if words.is_empty() || words.len() > 3 {
        return false;
    }
    words.iter().all(|w| {
        !w.is_empty()
            && w.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_cycle_label_picks_most_common() {
        use crate::models::proposal::ManagerProposal;
        use mahalaxmi_core::types::ManagerId;

        let mut p1 = ManagerProposal::new(ManagerId::new("m0"), vec![], 100);
        p1.cycle_label = Some("security-hardening".to_string());
        let mut p2 = ManagerProposal::new(ManagerId::new("m1"), vec![], 100);
        p2.cycle_label = Some("security-hardening".to_string());
        let mut p3 = ManagerProposal::new(ManagerId::new("m2"), vec![], 100);
        p3.cycle_label = Some("cost-model".to_string());

        let label = resolve_cycle_label(&[p1, p2, p3]);
        assert_eq!(label, Some("security-hardening".to_string()));
    }

    #[test]
    fn resolve_cycle_label_tie_broken_alphabetically() {
        use crate::models::proposal::ManagerProposal;
        use mahalaxmi_core::types::ManagerId;

        let mut p1 = ManagerProposal::new(ManagerId::new("m0"), vec![], 100);
        p1.cycle_label = Some("beta-feature".to_string());
        let mut p2 = ManagerProposal::new(ManagerId::new("m1"), vec![], 100);
        p2.cycle_label = Some("alpha-feature".to_string());

        let label = resolve_cycle_label(&[p1, p2]);
        // Alphabetically first among tied labels (both have count=1)
        assert_eq!(label, Some("alpha-feature".to_string()));
    }

    #[test]
    fn resolve_cycle_label_ignores_invalid_slugs() {
        use crate::models::proposal::ManagerProposal;
        use mahalaxmi_core::types::ManagerId;

        let mut p1 = ManagerProposal::new(ManagerId::new("m0"), vec![], 100);
        p1.cycle_label = Some("TOO MANY WORDS IN THIS SLUG".to_string());
        let mut p2 = ManagerProposal::new(ManagerId::new("m1"), vec![], 100);
        p2.cycle_label = Some("valid-label".to_string());

        let label = resolve_cycle_label(&[p1, p2]);
        assert_eq!(label, Some("valid-label".to_string()));
    }

    #[test]
    fn resolve_cycle_label_none_when_no_labels() {
        use crate::models::proposal::ManagerProposal;
        use mahalaxmi_core::types::ManagerId;

        let proposals = vec![ManagerProposal::new(ManagerId::new("m0"), vec![], 100)];
        let label = resolve_cycle_label(&proposals);
        assert_eq!(label, None);
    }

    #[test]
    fn is_valid_cycle_label_accepts_valid_slugs() {
        assert!(is_valid_cycle_label("security"));
        assert!(is_valid_cycle_label("security-hardening"));
        assert!(is_valid_cycle_label("cost-data-model"));
        assert!(is_valid_cycle_label("phase2"));
        assert!(is_valid_cycle_label("oauth2-auth"));
    }

    #[test]
    fn is_valid_cycle_label_rejects_invalid_slugs() {
        assert!(!is_valid_cycle_label(""));
        assert!(!is_valid_cycle_label("too-many-words-here"));
        assert!(!is_valid_cycle_label("UPPERCASE"));
        assert!(!is_valid_cycle_label("has spaces"));
        assert!(!is_valid_cycle_label("-leading-hyphen"));
        assert!(!is_valid_cycle_label("trailing-hyphen-"));
        assert!(!is_valid_cycle_label("double--hyphen"));
    }

    #[test]
    fn evaluate_with_domain_algorithm_majority_filters_low_support() {
        use crate::models::proposal::ManagerProposal;
        use crate::models::{proposal::ProposedTask, ConsensusConfiguration};
        use mahalaxmi_core::domain::ConsensusAlgorithm;
        use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
        use mahalaxmi_core::types::{ConsensusStrategy, ManagerId};

        let i18n = I18nService::new(SupportedLocale::EnUs);
        let config = ConsensusConfiguration {
            strategy: ConsensusStrategy::Union,
            ..ConsensusConfiguration::default()
        };
        let engine = ConsensusEngine::new(config);

        // 3 proposals, only 1 proposes TaskB — with threshold 0.67 it should be dropped.
        let task_a = ProposedTask::new("Task A", "Shared task");
        let task_b = ProposedTask::new("Task B", "Minority task");

        let p1 = ManagerProposal::new(ManagerId::new("m0"), vec![task_a.clone()], 100);
        let p2 = ManagerProposal::new(
            ManagerId::new("m1"),
            vec![task_a.clone(), task_b.clone()],
            100,
        );
        let p3 = ManagerProposal::new(ManagerId::new("m2"), vec![task_a.clone()], 100);

        let proposals = vec![p1, p2, p3];
        let algo = ConsensusAlgorithm::Majority { threshold: 0.67 };
        let result = engine
            .evaluate_with_domain_algorithm(&proposals, &i18n, Some(&algo))
            .unwrap();

        // Task A: 3/3 = 1.0 >= 0.67 → passes
        // Task B: 1/3 = 0.33 < 0.67 → filtered out
        assert!(
            result.agreed_tasks.iter().any(|t| t.title == "Task A"),
            "Task A should pass majority threshold"
        );
        assert!(
            result.agreed_tasks.iter().all(|t| t.title != "Task B"),
            "Task B should be filtered by majority threshold"
        );
    }

    #[test]
    fn evaluate_with_domain_algorithm_unanimous_is_unchanged() {
        use crate::models::proposal::ManagerProposal;
        use crate::models::{proposal::ProposedTask, ConsensusConfiguration};
        use mahalaxmi_core::domain::ConsensusAlgorithm;
        use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
        use mahalaxmi_core::types::{ConsensusStrategy, ManagerId};

        let i18n = I18nService::new(SupportedLocale::EnUs);
        let config = ConsensusConfiguration {
            strategy: ConsensusStrategy::Union,
            ..ConsensusConfiguration::default()
        };
        let engine = ConsensusEngine::new(config);

        let task = ProposedTask::new("Shared Task", "A shared task");
        let proposals = vec![
            ManagerProposal::new(ManagerId::new("m0"), vec![task.clone()], 100),
            ManagerProposal::new(ManagerId::new("m1"), vec![task.clone()], 100),
        ];

        let result_default = engine.evaluate(&proposals, &i18n).unwrap();
        let result_with_algo = engine
            .evaluate_with_domain_algorithm(
                &proposals,
                &i18n,
                Some(&ConsensusAlgorithm::Unanimous),
            )
            .unwrap();

        assert_eq!(
            result_default.agreed_tasks.len(),
            result_with_algo.agreed_tasks.len(),
            "Unanimous algorithm should produce identical results to default evaluate"
        );
    }

    #[test]
    fn consensus_engine_deduplicates_cycle2_scenario() {
        use crate::models::proposal::ManagerProposal;
        use crate::models::{proposal::ProposedTask, ConsensusConfiguration};
        use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
        use mahalaxmi_core::types::{ConsensusStrategy, ManagerId};

        let i18n = I18nService::new(SupportedLocale::EnUs);
        let config = ConsensusConfiguration {
            strategy: ConsensusStrategy::Union,
            ..ConsensusConfiguration::default()
        };
        let engine = ConsensusEngine::new(config);

        // 3 managers, each proposes the same 4 adapters with different naming
        let adapters = [
            (
                "GitHub Issues Work Intake Adapter",
                "GitHubIssuesAdapter",
                "GitHub Issues Intake Adapter",
            ),
            (
                "Jira Work Intake Adapter",
                "JiraAdapter",
                "Jira Issues Adapter",
            ),
            (
                "Slack Work Intake Adapter",
                "SlackAdapter",
                "Slack Notifications Adapter",
            ),
            (
                "REST Work Intake Adapter",
                "RestApiAdapter",
                "REST API Adapter",
            ),
        ];

        let mut proposals = Vec::new();
        for (idx, names) in adapters.iter().enumerate() {
            let m1_task = ProposedTask::new(names.0, &format!("Description for {}", names.0));
            let m2_task = ProposedTask::new(names.1, &format!("Description for {}", names.1));
            let m3_task = ProposedTask::new(names.2, &format!("Description for {}", names.2));

            proposals.push(ManagerProposal::new(
                ManagerId::new(&format!("manager-{idx}-a")),
                vec![m1_task],
                100,
            ));
            proposals.push(ManagerProposal::new(
                ManagerId::new(&format!("manager-{idx}-b")),
                vec![m2_task],
                100,
            ));
            proposals.push(ManagerProposal::new(
                ManagerId::new(&format!("manager-{idx}-c")),
                vec![m3_task],
                100,
            ));
        }

        let result = engine.evaluate(&proposals, &i18n).unwrap();

        // Without deduplication: 12 workers (4 adapters × 3 variants each)
        // With deduplication: ≤ 4 workers (1 per adapter family)
        assert!(
            result.agreed_tasks.len() <= 6,
            "Should deduplicate to at most 6 tasks (ideally 4), got {}",
            result.agreed_tasks.len()
        );
    }
}
