use std::collections::HashMap;

use crate::consensus::normalizer::TaskGroup;
use crate::consensus::strategy::ConsensusStrategyImpl;
use crate::models::{
    ConsensusConfiguration, ConsensusMetrics, ConsensusResult, ConsensusTask, DissentingTask,
    ManagerProposal,
};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::ManagerId;
use mahalaxmi_core::MahalaxmiResult;

/// Weighted voting strategy: include tasks meeting a minimum weighted-agreement threshold.
///
/// Each manager proposal carries a `developer_weight` (default `1.0`) that was
/// resolved from the `DeveloperRegistry` at submission time by the orchestration
/// service. A task's score is the sum of weights of all proposing managers divided
/// by the total weight of all successful proposals. A task is included when its
/// weighted score meets or exceeds `minimum_agreement_threshold`.
///
/// When all proposals use the default weight of `1.0` this degenerates to the
/// original vote-count ratio, maintaining full backward compatibility.
pub struct WeightedVotingStrategy;

impl WeightedVotingStrategy {
    /// Compute the per-manager weight map from a proposal slice.
    ///
    /// Maps each successful manager's ID to its `developer_weight`. The weight
    /// was set by the orchestration service via `DeveloperRegistry::get(id)` at
    /// `submit_manager_output()` time; this function merely reads the cached value.
    fn build_weight_map(proposals: &[ManagerProposal]) -> HashMap<ManagerId, f32> {
        proposals
            .iter()
            .filter(|p| p.completed)
            .map(|p| (p.manager_id.clone(), p.developer_weight))
            .collect()
    }

    /// Compute the weighted score for a single task group.
    ///
    /// Returns the sum of weights of all managers in the group that are present
    /// in `weight_map`. Managers not found in the map (e.g. failed proposals
    /// that were filtered out) contribute zero.
    fn weighted_score(group_managers: &[ManagerId], weight_map: &HashMap<ManagerId, f32>) -> f32 {
        group_managers
            .iter()
            .filter_map(|mid| weight_map.get(mid))
            .sum()
    }
}

impl ConsensusStrategyImpl for WeightedVotingStrategy {
    fn name(&self) -> &str {
        "WeightedVoting"
    }

    fn evaluate_groups(
        &self,
        groups: Vec<TaskGroup>,
        config: &ConsensusConfiguration,
        _i18n: &I18nService,
        total_proposals: u32,
        successful_proposals: u32,
    ) -> MahalaxmiResult<ConsensusResult> {
        // For weighted voting, we need weight information from proposals.
        // Since evaluate_groups receives pre-grouped data, we use uniform
        // weight (1.0 per manager) as the default. The full weight map is
        // only available when evaluate() is called with proposals.
        // Build a synthetic weight map: each manager entry in the group gets weight 1.0.
        let total_weight = successful_proposals as f32;
        let total_unique = groups.len() as u32;

        let mut agreed_tasks = Vec::new();
        let mut dissenting_tasks = Vec::new();

        for group in &groups {
            let vote_ratio = if total_weight > 0.0 {
                group.vote_count() as f64 / total_weight as f64
            } else {
                0.0
            };

            if vote_ratio >= config.minimum_agreement_threshold {
                agreed_tasks.push(ConsensusTask {
                    normalized_key: group.normalized_key.clone(),
                    title: group.representative_title().to_owned(),
                    description: group.best_description().to_owned(),
                    average_complexity: group.average_complexity(),
                    vote_count: group.vote_count(),
                    total_managers: successful_proposals,
                    proposed_by: group.proposing_managers(),
                    dependencies: group.merged_dependencies(),
                    affected_files: group.merged_affected_files(),
                    acceptance_criteria: group.merged_acceptance_criteria(),
                });
            } else {
                dissenting_tasks.push(DissentingTask {
                    normalized_key: group.normalized_key.clone(),
                    title: group.representative_title().to_owned(),
                    vote_count: group.vote_count(),
                    reason: format!(
                        "Vote ratio {:.2} below threshold {:.2}",
                        vote_ratio, config.minimum_agreement_threshold
                    ),
                });
            }
        }

        let agreed_count = agreed_tasks.len() as u32;
        let dissenting_count = dissenting_tasks.len() as u32;

        let unanimity = groups
            .iter()
            .filter(|g| g.vote_count() == successful_proposals)
            .count() as u32;

        let avg_complexity = if agreed_tasks.is_empty() {
            0.0
        } else {
            agreed_tasks
                .iter()
                .map(|t| t.average_complexity)
                .sum::<f64>()
                / agreed_tasks.len() as f64
        };

        let overlap = if total_unique > 0 && successful_proposals > 1 {
            unanimity as f64 / total_unique as f64
        } else if successful_proposals <= 1 {
            1.0
        } else {
            0.0
        };

        Ok(ConsensusResult {
            strategy: mahalaxmi_core::types::ConsensusStrategy::WeightedVoting,
            agreed_tasks,
            dissenting_tasks,
            metrics: ConsensusMetrics {
                total_proposals,
                successful_proposals,
                total_unique_tasks: total_unique,
                agreed_task_count: agreed_count,
                dissenting_task_count: dissenting_count,
                unanimity_count: unanimity,
                average_complexity: avg_complexity,
                overlap_ratio: overlap,
            },
            cycle_label: None, // Resolved by ConsensusEngine after strategy returns
        })
    }

    /// Override evaluate() to use actual developer weights from proposals.
    ///
    /// This override preserves the original weighted-voting behavior where each
    /// manager's `developer_weight` field drives inclusion decisions. When called
    /// via the default trait path (through evaluate_groups), uniform weights are
    /// used instead.
    fn evaluate(
        &self,
        proposals: &[ManagerProposal],
        config: &ConsensusConfiguration,
        _i18n: &I18nService,
    ) -> MahalaxmiResult<ConsensusResult> {
        use crate::consensus::normalizer::group_matching_tasks;

        let weight_map = Self::build_weight_map(proposals);
        let total_weight: f32 = weight_map.values().sum();
        let successful = proposals.iter().filter(|p| p.completed).count() as u32;

        let groups = group_matching_tasks(proposals);
        let total_unique = groups.len() as u32;

        let mut agreed_tasks = Vec::new();
        let mut dissenting_tasks = Vec::new();

        for group in &groups {
            let group_managers: Vec<ManagerId> =
                group.entries.iter().map(|(m, _)| m.clone()).collect();
            let group_weight = Self::weighted_score(&group_managers, &weight_map);

            let vote_ratio = if total_weight > 0.0 {
                group_weight as f64 / total_weight as f64
            } else {
                0.0
            };

            if vote_ratio >= config.minimum_agreement_threshold {
                agreed_tasks.push(ConsensusTask {
                    normalized_key: group.normalized_key.clone(),
                    title: group.representative_title().to_owned(),
                    description: group.best_description().to_owned(),
                    average_complexity: group.average_complexity(),
                    vote_count: group.vote_count(),
                    total_managers: successful,
                    proposed_by: group.proposing_managers(),
                    dependencies: group.merged_dependencies(),
                    affected_files: group.merged_affected_files(),
                    acceptance_criteria: group.merged_acceptance_criteria(),
                });
            } else {
                dissenting_tasks.push(DissentingTask {
                    normalized_key: group.normalized_key.clone(),
                    title: group.representative_title().to_owned(),
                    vote_count: group.vote_count(),
                    reason: format!(
                        "Weighted score {:.2} below threshold {:.2}",
                        vote_ratio, config.minimum_agreement_threshold
                    ),
                });
            }
        }

        let agreed_count = agreed_tasks.len() as u32;
        let dissenting_count = dissenting_tasks.len() as u32;

        // Unanimity: all successful managers proposed this task (count-based,
        // independent of weights so that the metric retains its original meaning).
        let unanimity = groups
            .iter()
            .filter(|g| g.vote_count() == successful)
            .count() as u32;

        let avg_complexity = if agreed_tasks.is_empty() {
            0.0
        } else {
            agreed_tasks
                .iter()
                .map(|t| t.average_complexity)
                .sum::<f64>()
                / agreed_tasks.len() as f64
        };

        let overlap = if total_unique > 0 && successful > 1 {
            unanimity as f64 / total_unique as f64
        } else if successful <= 1 {
            1.0
        } else {
            0.0
        };

        Ok(ConsensusResult {
            strategy: mahalaxmi_core::types::ConsensusStrategy::WeightedVoting,
            agreed_tasks,
            dissenting_tasks,
            metrics: ConsensusMetrics {
                total_proposals: proposals.len() as u32,
                successful_proposals: successful,
                total_unique_tasks: total_unique,
                agreed_task_count: agreed_count,
                dissenting_task_count: dissenting_count,
                unanimity_count: unanimity,
                average_complexity: avg_complexity,
                overlap_ratio: overlap,
            },
            cycle_label: None, // Resolved by ConsensusEngine after strategy returns
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::strategy::ConsensusStrategyImpl;
    use crate::models::proposal::{ManagerProposal, ProposedTask};
    use crate::models::ConsensusConfiguration;
    use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
    use mahalaxmi_core::types::{ConsensusStrategy, DeveloperId, ManagerId};

    fn i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    fn config_with_threshold(threshold: f64) -> ConsensusConfiguration {
        ConsensusConfiguration {
            strategy: ConsensusStrategy::WeightedVoting,
            minimum_agreement_threshold: threshold,
            ..ConsensusConfiguration::default()
        }
    }

    /// Creates a minimal successful `ManagerProposal` with a single task and
    /// an explicit `developer_weight`.
    fn proposal_with_weight(
        manager_id: &str,
        task_title: &str,
        developer_id: &str,
        developer_weight: f32,
    ) -> ManagerProposal {
        let mut p = ManagerProposal::new(
            ManagerId::new(manager_id),
            vec![ProposedTask::new(task_title, "description")],
            100,
        );
        p.developer_id = Some(DeveloperId::from(developer_id));
        p.developer_weight = developer_weight;
        p
    }

    /// Tasks from a high-weight developer score above the threshold while an
    /// equal-count task from a low-weight developer falls below it, confirming
    /// that `developer_weight` drives inclusion decisions.
    #[test]
    fn test_weighted_voting_applies_developer_weight() {
        // alice: weight 2.0 → database migration gets weighted score 2.0 / 2.5 = 0.8
        let proposal_alice =
            proposal_with_weight("manager-alice", "Migrate Database Schema", "alice", 2.0);

        // bob: weight 0.5 → refactor logging gets weighted score 0.5 / 2.5 = 0.2
        let proposal_bob = proposal_with_weight("manager-bob", "Refactor Logging", "bob", 0.5);

        // threshold 0.5: database migration (0.8) passes, refactor logging (0.2) does not
        let config = config_with_threshold(0.5);
        let result = WeightedVotingStrategy
            .evaluate(&[proposal_alice, proposal_bob], &config, &i18n())
            .unwrap();

        assert_eq!(
            result.agreed_tasks.len(),
            1,
            "Only the high-weight task should be agreed"
        );
        assert!(
            result.agreed_tasks[0].title.contains("Migrate Database"),
            "Database migration (alice, weight 2.0) must be agreed; got: {:?}",
            result.agreed_tasks[0].title
        );
        assert_eq!(
            result.dissenting_tasks.len(),
            1,
            "Refactor Logging should be dissenting"
        );
        assert!(
            result.dissenting_tasks[0]
                .title
                .contains("Refactor Logging"),
            "Refactor Logging (bob, weight 0.5) must be dissenting"
        );
    }

    /// With default weight (1.0 for all proposals) the strategy behaves
    /// identically to the original unweighted ratio, preserving backward
    /// compatibility.
    #[test]
    fn test_weighted_voting_default_weight_backward_compat() {
        let p1 = ManagerProposal::new(
            ManagerId::new("m0"),
            vec![
                ProposedTask::new("shared-task", "shared"),
                ProposedTask::new("m0-only", "exclusive"),
            ],
            100,
        );
        let p2 = ManagerProposal::new(
            ManagerId::new("m1"),
            vec![ProposedTask::new("shared-task", "shared")],
            100,
        );

        // threshold 0.5 with equal weights: shared-task (2/2=1.0) passes,
        // m0-only (1/2=0.5) also passes at exactly 0.5 (>=).
        let config = config_with_threshold(0.5);
        let result = WeightedVotingStrategy
            .evaluate(&[p1, p2], &config, &i18n())
            .unwrap();

        assert_eq!(result.agreed_tasks.len(), 2);
    }
}
