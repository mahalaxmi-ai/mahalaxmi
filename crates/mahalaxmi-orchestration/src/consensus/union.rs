use crate::consensus::normalizer::TaskGroup;
use crate::consensus::strategy::ConsensusStrategyImpl;
use crate::models::{ConsensusConfiguration, ConsensusMetrics, ConsensusResult, ConsensusTask};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;

/// Union strategy: include all unique tasks from all managers.
///
/// Every task proposed by at least one manager is included. No tasks are dissenting.
pub struct UnionStrategy;

impl ConsensusStrategyImpl for UnionStrategy {
    fn name(&self) -> &str {
        "Union"
    }

    fn evaluate_groups(
        &self,
        groups: Vec<TaskGroup>,
        _config: &ConsensusConfiguration,
        _i18n: &I18nService,
        total_proposals: u32,
        successful_proposals: u32,
    ) -> MahalaxmiResult<ConsensusResult> {
        let total_unique = groups.len() as u32;

        let agreed_tasks: Vec<ConsensusTask> = groups
            .iter()
            .map(|group| ConsensusTask {
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
            })
            .collect();

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
        } else {
            1.0
        };

        Ok(ConsensusResult {
            strategy: mahalaxmi_core::types::ConsensusStrategy::Union,
            agreed_tasks,
            dissenting_tasks: Vec::new(),
            metrics: ConsensusMetrics {
                total_proposals,
                successful_proposals,
                total_unique_tasks: total_unique,
                agreed_task_count: total_unique,
                dissenting_task_count: 0,
                unanimity_count: unanimity,
                average_complexity: avg_complexity,
                overlap_ratio: overlap,
            },
            cycle_label: None, // Resolved by ConsensusEngine after strategy returns
        })
    }
}
