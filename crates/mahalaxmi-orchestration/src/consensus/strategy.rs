use crate::consensus::normalizer::{group_matching_tasks, TaskGroup};
use crate::models::{ConsensusConfiguration, ConsensusResult, ManagerProposal};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;

/// Trait for consensus strategy implementations.
///
/// Implementors define [`evaluate_groups`] which receives pre-grouped
/// (and optionally arbitrated) task groups.  The default [`evaluate`]
/// implementation performs the grouping step and delegates.
pub trait ConsensusStrategyImpl: Send + Sync {
    /// Returns the name of this strategy.
    fn name(&self) -> &str;

    /// Primary evaluation method — receives pre-grouped tasks after
    /// the enhanced lexical deduplication (and optional LLM arbitration).
    ///
    /// `total_proposals` is the total number of proposals (including failed ones),
    /// used for metrics.  `successful_proposals` is the count of completed proposals,
    /// used for threshold calculations and unanimity detection.
    fn evaluate_groups(
        &self,
        groups: Vec<TaskGroup>,
        config: &ConsensusConfiguration,
        i18n: &I18nService,
        total_proposals: u32,
        successful_proposals: u32,
    ) -> MahalaxmiResult<ConsensusResult>;

    /// Convenience entry point for callers that have raw proposals.
    ///
    /// Groups the proposals via the enhanced CamelCase-aware lexical grouping,
    /// then delegates to [`evaluate_groups`].  Existing callers (including tests
    /// that call the strategy directly) continue to work unchanged.
    fn evaluate(
        &self,
        proposals: &[ManagerProposal],
        config: &ConsensusConfiguration,
        i18n: &I18nService,
    ) -> MahalaxmiResult<ConsensusResult> {
        let groups = group_matching_tasks(proposals);
        let total_proposals = proposals.len() as u32;
        let successful_proposals = proposals.iter().filter(|p| p.completed).count() as u32;
        self.evaluate_groups(groups, config, i18n, total_proposals, successful_proposals)
    }
}
