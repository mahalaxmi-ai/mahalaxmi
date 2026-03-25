use crate::consensus::similarity::{
    multi_field_similarity, normalized_tokens, token_jaccard, SimilarityWeights,
};
use crate::models::proposal::ProposedTask;
use mahalaxmi_core::types::ManagerId;
use std::collections::HashMap;

/// A group of tasks from different managers that were matched as the same task.
#[derive(Debug, Clone)]
pub struct TaskGroup {
    /// The normalized key shared by all tasks in this group.
    pub normalized_key: String,
    /// The tasks grouped together, with their source manager ID.
    pub entries: Vec<(ManagerId, ProposedTask)>,
}

impl TaskGroup {
    /// Number of managers who proposed this task.
    pub fn vote_count(&self) -> u32 {
        self.entries.len() as u32
    }

    /// Average complexity across all proposals of this task.
    pub fn average_complexity(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let sum: u32 = self.entries.iter().map(|(_, t)| t.complexity).sum();
        sum as f64 / self.entries.len() as f64
    }

    /// Get the title from the first proposal (representative).
    pub fn representative_title(&self) -> &str {
        self.entries
            .first()
            .map(|(_, t)| t.title.as_str())
            .unwrap_or("")
    }

    /// Get the longest description from all proposals.
    pub fn best_description(&self) -> &str {
        self.entries
            .iter()
            .map(|(_, t)| t.description.as_str())
            .max_by_key(|d| d.len())
            .unwrap_or("")
    }

    /// Collect all unique manager IDs that proposed this task.
    pub fn proposing_managers(&self) -> Vec<ManagerId> {
        self.entries.iter().map(|(m, _)| m.clone()).collect()
    }

    /// Merge all dependencies from all proposals (deduplicated).
    pub fn merged_dependencies(&self) -> Vec<String> {
        let mut deps: Vec<String> = self
            .entries
            .iter()
            .flat_map(|(_, t)| t.dependencies.iter().cloned())
            .collect();
        deps.sort();
        deps.dedup();
        deps
    }

    /// Merge all acceptance criteria from all proposals (deduplicated).
    pub fn merged_acceptance_criteria(&self) -> Vec<String> {
        let mut criteria: Vec<String> = self
            .entries
            .iter()
            .flat_map(|(_, t)| t.acceptance_criteria.iter().cloned())
            .collect();
        criteria.sort();
        criteria.dedup();
        criteria
    }

    /// Merge all affected files from all proposals (deduplicated).
    pub fn merged_affected_files(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .entries
            .iter()
            .flat_map(|(_, t)| t.affected_files.iter().cloned())
            .collect();
        files.sort();
        files.dedup();
        files
    }
}

/// Normalize a task key for cross-proposal matching.
///
/// Uses CamelCase-aware tokenization: splits on CamelCase boundaries,
/// underscores, whitespace, and punctuation, then removes stop words.
/// This ensures "GitHubIssuesAdapter" and "GitHub Issues Adapter" produce
/// the same tokens and are matched in the initial grouping pass.
pub fn normalize_task_key(title: &str) -> String {
    let tokens = normalized_tokens(title);
    if tokens.is_empty() {
        // Fallback: plain lowercase for titles that reduce to nothing after stop-word removal
        title
            .to_lowercase()
            .trim()
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    } else {
        tokens.join("-")
    }
}

/// Determine whether two task groups should be merged based on multi-field similarity.
///
/// Uses a combined score from title, file overlap, description, and criteria signals.
/// When files are absent from both groups, the file weight is redistributed to the
/// other signals so that tasks without file lists can still be merged.
///
/// **Manager-overlap guard:** if any manager appears in both groups the groups
/// cannot represent the same logical task — the same manager would not propose
/// the same work twice.  Any such overlap returns `false` immediately.
fn should_merge(a: &TaskGroup, b: &TaskGroup) -> bool {
    // If any manager is present in both groups they are distinct tasks.
    let managers_a: std::collections::HashSet<_> = a.entries.iter().map(|(m, _)| m).collect();
    if b.entries.iter().any(|(m, _)| managers_a.contains(m)) {
        return false;
    }

    let files_a = a.merged_affected_files();
    let files_b = b.merged_affected_files();
    let criteria_a = a.merged_acceptance_criteria().join(" ");
    let criteria_b = b.merged_acceptance_criteria().join(" ");
    let weights = SimilarityWeights::default();

    // Use the title pair (ta, tb) with maximum Jaccard similarity.
    // This prevents ordering-dependent results when groups hold multiple merged entries
    // whose representative_title may not be the best comparand for the other group.
    let (best_title_a, best_title_b) = a
        .entries
        .iter()
        .flat_map(|(_, ta)| b.entries.iter().map(move |(_, tb)| (&ta.title, &tb.title)))
        .max_by(|(ta1, tb1), (ta2, tb2)| {
            let s1 = token_jaccard(ta1, tb1);
            let s2 = token_jaccard(ta2, tb2);
            s1.partial_cmp(&s2).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(ta, tb)| (ta.as_str(), tb.as_str()))
        .unwrap_or((a.representative_title(), b.representative_title()));

    let score = multi_field_similarity(
        best_title_a,
        best_title_b,
        &files_a,
        &files_b,
        a.best_description(),
        b.best_description(),
        &criteria_a,
        &criteria_b,
        &weights,
    );
    score >= weights.merge_threshold
}

/// Merge task groups that have high combined similarity.
///
/// Runs after the initial exact-key grouping to catch semantically
/// identical tasks that different managers titled differently (e.g. CamelCase
/// vs spaced, or with different suffixes).
/// Uses a fixed-point loop: keeps merging until no more pairs qualify.
fn merge_overlapping_groups(mut groups: Vec<TaskGroup>) -> Vec<TaskGroup> {
    let mut merged = true;
    while merged {
        merged = false;
        let mut i = 0;
        while i < groups.len() {
            let mut j = i + 1;
            while j < groups.len() {
                if should_merge(&groups[i], &groups[j]) {
                    let removed = groups.remove(j);
                    groups[i].entries.extend(removed.entries);
                    merged = true;
                    // Don't increment j — the next element shifted down
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }
    groups
}

/// Group matching tasks from multiple proposals by their normalized key,
/// then merge groups with high combined similarity.
///
/// Two-pass grouping:
/// 1. **Pass 1:** Group by exact normalized key (fast HashMap lookup).
///    CamelCase-aware tokenization ensures "GitHubIssuesAdapter" and
///    "GitHub Issues Adapter" produce the same key.
///    Invariant: each manager contributes **at most one entry per TaskGroup**.
///    When a normalized-key collision occurs within a single proposal (e.g.
///    "Task A" and "Task B" both reduce to "task"), a numeric suffix is
///    appended to keep the tasks distinct.
/// 2. **Pass 2:** Merge groups where multi-field similarity score ≥ 0.45
///    (catches remaining differently-titled duplicates, handles missing file lists).
pub fn group_matching_tasks(proposals: &[crate::models::ManagerProposal]) -> Vec<TaskGroup> {
    let mut groups: HashMap<String, TaskGroup> = HashMap::new();

    for proposal in proposals {
        if !proposal.completed {
            continue;
        }
        for task in &proposal.tasks {
            let base_key = normalize_task_key(&task.title);

            // Enforce the one-entry-per-manager invariant: if the current
            // manager already occupies this key (or a suffixed variant),
            // keep suffixing until we find a free slot.
            let key = if groups
                .get(&base_key)
                .map(|g| g.entries.iter().any(|(m, _)| m == &proposal.manager_id))
                .unwrap_or(false)
            {
                let mut counter = 1u32;
                loop {
                    let candidate = format!("{base_key}-{counter}");
                    let occupied = groups
                        .get(&candidate)
                        .map(|g| g.entries.iter().any(|(m, _)| m == &proposal.manager_id))
                        .unwrap_or(false);
                    if !occupied {
                        break candidate;
                    }
                    counter += 1;
                }
            } else {
                base_key.clone()
            };

            let group = groups.entry(key.clone()).or_insert_with(|| TaskGroup {
                normalized_key: key,
                entries: Vec::new(),
            });
            group
                .entries
                .push((proposal.manager_id.clone(), task.clone()));
        }
    }

    let mut result: Vec<TaskGroup> = groups.into_values().collect();
    result = merge_overlapping_groups(result);
    result.sort_by(|a, b| a.normalized_key.cmp(&b.normalized_key));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::proposal::{ManagerProposal, ProposedTask};
    use mahalaxmi_core::types::ManagerId;

    #[test]
    fn test_normalize_task_key_splits_camel_case() {
        // "GitHubIssuesAdapter" should produce the same key as "GitHub Issues Adapter"
        // since "adapter" is a stop word, both reduce to "git-hub-issues" or similar
        let key1 = normalize_task_key("GitHubIssuesAdapter");
        let key2 = normalize_task_key("GitHub Issues Adapter");
        assert_eq!(
            key1, key2,
            "CamelCase and spaced versions should normalize to the same key"
        );
    }

    #[test]
    fn test_normalize_task_key_snake_case() {
        let key = normalize_task_key("github_issues_adapter");
        assert!(
            !key.contains("adapter"),
            "stop word 'adapter' should be removed"
        );
        assert!(
            key.contains("issues"),
            "meaningful token 'issues' should remain"
        );
    }

    #[test]
    fn test_group_matching_tasks_merges_cycle2_camelcase_duplicates() {
        // Reproduce the Cycle 2 bug scenario:
        // 3 managers propose the same adapter with different naming conventions
        let m1 = ManagerId::new("manager-1");
        let m2 = ManagerId::new("manager-2");
        let m3 = ManagerId::new("manager-3");

        let p1 = ManagerProposal::new(
            m1,
            vec![ProposedTask::new(
                "GitHub Issues Work Intake Adapter",
                "Handles GitHub issues as work intake",
            )],
            100,
        );
        let p2 = ManagerProposal::new(
            m2,
            vec![ProposedTask::new(
                "GitHubIssuesAdapter",
                "Adapter for GitHub issue events",
            )],
            100,
        );
        let p3 = ManagerProposal::new(
            m3,
            vec![ProposedTask::new(
                "GitHub Issues Intake Adapter",
                "GitHub issues intake processing",
            )],
            100,
        );

        let groups = group_matching_tasks(&[p1, p2, p3]);
        assert_eq!(
            groups.len(),
            1,
            "Three variants of GitHub Issues Adapter should merge into 1 group, got {} groups: {:?}",
            groups.len(),
            groups.iter().map(|g| g.representative_title()).collect::<Vec<_>>()
        );
        assert_eq!(groups[0].vote_count(), 3, "Group should have 3 votes");
    }

    #[test]
    fn test_group_matching_tasks_keeps_distinct_tasks_separate() {
        let m1 = ManagerId::new("manager-1");
        let p1 = ManagerProposal::new(
            m1,
            vec![
                ProposedTask::new("GitHub Issues Adapter", "GitHub intake"),
                ProposedTask::new("Jira Issues Adapter", "Jira intake"),
                ProposedTask::new("Slack Notifications", "Slack notifications"),
            ],
            100,
        );

        let groups = group_matching_tasks(&[p1]);
        assert_eq!(
            groups.len(),
            3,
            "Three distinct tasks should remain separate"
        );
    }

    #[test]
    fn test_normalize_task_key_empty_after_stop_words() {
        // "Add New" after stop words → empty → fallback to raw normalization
        let key = normalize_task_key("Add New");
        assert!(
            !key.is_empty(),
            "Should fall back to raw normalization for all-stop-word titles"
        );
    }
}
