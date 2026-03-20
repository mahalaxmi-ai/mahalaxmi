use crate::consensus::normalizer::TaskGroup;
use crate::consensus::strategy::ConsensusStrategyImpl;
use crate::models::{
    ConsensusConfiguration, ConsensusMetrics, ConsensusResult, ConsensusTask, DissentingTask,
};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;

/// Complexity-weighted strategy: score = frequency_weight * vote_ratio + complexity_weight * (1/avg_complexity).
///
/// Tasks with higher vote ratios AND lower complexity (simpler tasks) score higher.
/// The threshold is `minimum_agreement_threshold` from the configuration.
pub struct ComplexityWeightedStrategy;

impl ConsensusStrategyImpl for ComplexityWeightedStrategy {
    fn name(&self) -> &str {
        "ComplexityWeighted"
    }

    fn evaluate_groups(
        &self,
        groups: Vec<TaskGroup>,
        config: &ConsensusConfiguration,
        _i18n: &I18nService,
        total_proposals: u32,
        successful_proposals: u32,
    ) -> MahalaxmiResult<ConsensusResult> {
        let total_unique = groups.len() as u32;

        let mut agreed_tasks = Vec::new();
        let mut dissenting_tasks = Vec::new();

        for group in &groups {
            let vote_ratio = if successful_proposals > 0 {
                group.vote_count() as f64 / successful_proposals as f64
            } else {
                0.0
            };

            let avg_complexity = group.average_complexity().max(1.0);
            let complexity_factor = 1.0 / avg_complexity;

            let score =
                config.frequency_weight * vote_ratio + config.complexity_weight * complexity_factor;

            if score >= config.minimum_agreement_threshold {
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
                        "Score {:.3} below threshold {:.2}",
                        score, config.minimum_agreement_threshold
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
            strategy: mahalaxmi_core::types::ConsensusStrategy::ComplexityWeighted,
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
}
