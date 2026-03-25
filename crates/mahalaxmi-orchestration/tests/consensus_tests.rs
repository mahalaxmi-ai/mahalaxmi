use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ConsensusStrategy, ManagerId};
use mahalaxmi_orchestration::consensus::normalizer::{group_matching_tasks, normalize_task_key};
use mahalaxmi_orchestration::consensus::similarity::{
    file_overlap_ratio, token_jaccard as jaccard_token_similarity,
};
use mahalaxmi_orchestration::models::{ConsensusConfiguration, ManagerProposal, ProposedTask};
use mahalaxmi_orchestration::ConsensusEngine;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

fn make_proposal(manager_name: &str, tasks: Vec<ProposedTask>) -> ManagerProposal {
    ManagerProposal::new(ManagerId::new(manager_name), tasks, 1000)
}

fn make_failed_proposal(manager_name: &str) -> ManagerProposal {
    ManagerProposal::failed(ManagerId::new(manager_name), "timeout", 5000)
}

fn task(title: &str) -> ProposedTask {
    ProposedTask::new(title, format!("Description for {}", title))
}

fn task_with_complexity(title: &str, complexity: u32) -> ProposedTask {
    ProposedTask::new(title, format!("Description for {}", title)).with_complexity(complexity)
}

fn task_with_files(title: &str, files: &[&str]) -> ProposedTask {
    let mut t = ProposedTask::new(title, format!("Description for {}", title));
    for f in files {
        t = t.with_affected_file(*f);
    }
    t
}

fn union_config() -> ConsensusConfiguration {
    ConsensusConfiguration {
        strategy: ConsensusStrategy::Union,
        minimum_agreement_threshold: 0.5,
        frequency_weight: 0.7,
        complexity_weight: 0.3,
    }
}

fn intersection_config() -> ConsensusConfiguration {
    ConsensusConfiguration {
        strategy: ConsensusStrategy::Intersection,
        minimum_agreement_threshold: 0.5,
        frequency_weight: 0.7,
        complexity_weight: 0.3,
    }
}

fn weighted_config(threshold: f64) -> ConsensusConfiguration {
    ConsensusConfiguration {
        strategy: ConsensusStrategy::WeightedVoting,
        minimum_agreement_threshold: threshold,
        frequency_weight: 0.7,
        complexity_weight: 0.3,
    }
}

fn complexity_config(
    threshold: f64,
    frequency_weight: f64,
    complexity_weight: f64,
) -> ConsensusConfiguration {
    ConsensusConfiguration {
        strategy: ConsensusStrategy::ComplexityWeighted,
        minimum_agreement_threshold: threshold,
        frequency_weight,
        complexity_weight,
    }
}

// ========== Union Strategy Tests ==========

#[test]
fn union_includes_all_unique_tasks() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1"), task("task2")]),
        make_proposal("mgr-b", vec![task("task2"), task("task3")]),
    ];
    let engine = ConsensusEngine::new(union_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    assert_eq!(result.agreed_tasks.len(), 3);
    assert_eq!(result.dissenting_tasks.len(), 0);
}

#[test]
fn union_single_proposal() {
    let i18n = i18n();
    let proposals = vec![make_proposal(
        "mgr-a",
        vec![task("task1"), task("task2"), task("task3")],
    )];
    let engine = ConsensusEngine::new(union_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    assert_eq!(result.agreed_tasks.len(), 3);
}

#[test]
fn union_no_proposals() {
    let i18n = i18n();
    let proposals: Vec<ManagerProposal> = Vec::new();
    let engine = ConsensusEngine::new(union_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    assert_eq!(result.agreed_tasks.len(), 0);
}

#[test]
fn union_failed_proposals_excluded() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1"), task("task2")]),
        make_failed_proposal("mgr-b"),
    ];
    let engine = ConsensusEngine::new(union_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // Only the successful proposal's tasks should be included
    assert_eq!(result.agreed_tasks.len(), 2);
    assert_eq!(result.metrics.total_proposals, 2);
    assert_eq!(result.metrics.successful_proposals, 1);
}

#[test]
fn union_metrics_correct() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1"), task("task2"), task("task3")]),
        make_proposal("mgr-b", vec![task("task2"), task("task3")]),
    ];
    let engine = ConsensusEngine::new(union_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    assert_eq!(result.metrics.total_proposals, 2);
    assert_eq!(result.metrics.successful_proposals, 2);
    assert_eq!(result.metrics.total_unique_tasks, 3);
    assert_eq!(result.metrics.agreed_task_count, 3);
    assert_eq!(result.metrics.dissenting_task_count, 0);
    // task2 and task3 are unanimous (2/2), task1 is only from mgr-a
    assert_eq!(result.metrics.unanimity_count, 2);
    // overlap = unanimity / total_unique = 2/3
    let expected_overlap = 2.0 / 3.0;
    assert!((result.metrics.overlap_ratio - expected_overlap).abs() < 0.001);
}

// ========== Intersection Strategy Tests ==========

#[test]
fn intersection_only_unanimous_tasks() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1"), task("task2")]),
        make_proposal("mgr-b", vec![task("task1"), task("task3")]),
        make_proposal("mgr-c", vec![task("task1"), task("task2")]),
    ];
    let engine = ConsensusEngine::new(intersection_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // Only task1 is proposed by all 3 managers
    assert_eq!(result.agreed_tasks.len(), 1);
    assert_eq!(result.agreed_tasks[0].normalized_key, "task1");

    // task2 (2/3) and task3 (1/3) are dissenting
    assert_eq!(result.dissenting_tasks.len(), 2);
}

#[test]
fn intersection_single_proposal_includes_all() {
    let i18n = i18n();
    let proposals = vec![make_proposal(
        "mgr-a",
        vec![task("task1"), task("task2"), task("task3")],
    )];
    let engine = ConsensusEngine::new(intersection_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // With 1 manager, 1/1 == unanimous, so all tasks agreed
    assert_eq!(result.agreed_tasks.len(), 3);
    assert_eq!(result.dissenting_tasks.len(), 0);
}

#[test]
fn intersection_no_overlap() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1")]),
        make_proposal("mgr-b", vec![task("task2")]),
    ];
    let engine = ConsensusEngine::new(intersection_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    assert_eq!(result.agreed_tasks.len(), 0);
    assert_eq!(result.dissenting_tasks.len(), 2);
}

#[test]
fn intersection_all_overlap() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1"), task("task2")]),
        make_proposal("mgr-b", vec![task("task1"), task("task2")]),
    ];
    let engine = ConsensusEngine::new(intersection_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    assert_eq!(result.agreed_tasks.len(), 2);
    assert_eq!(result.dissenting_tasks.len(), 0);
}

#[test]
fn intersection_metrics_correct() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1"), task("task2")]),
        make_proposal("mgr-b", vec![task("task1"), task("task3")]),
    ];
    let engine = ConsensusEngine::new(intersection_config());
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    assert_eq!(result.metrics.total_proposals, 2);
    assert_eq!(result.metrics.successful_proposals, 2);
    assert_eq!(result.metrics.total_unique_tasks, 3);
    assert_eq!(result.metrics.agreed_task_count, 1); // only task1
    assert_eq!(result.metrics.dissenting_task_count, 2); // task2, task3
    assert_eq!(result.metrics.unanimity_count, 1);
    // overlap = agreed / total = 1/3
    let expected_overlap = 1.0 / 3.0;
    assert!((result.metrics.overlap_ratio - expected_overlap).abs() < 0.001);
}

// ========== Weighted Voting Strategy Tests ==========

#[test]
fn weighted_voting_at_50_threshold() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1")]),
        make_proposal("mgr-b", vec![task("task1")]),
        make_proposal("mgr-c", vec![task("task2")]),
        make_proposal("mgr-d", vec![task("task2")]),
    ];
    let engine = ConsensusEngine::new(weighted_config(0.5));
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // task1: 2/4=0.5 >= 0.5 -> agreed; task2: 2/4=0.5 >= 0.5 -> agreed
    assert_eq!(result.agreed_tasks.len(), 2);
    assert_eq!(result.dissenting_tasks.len(), 0);
}

#[test]
fn weighted_voting_below_threshold() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("lonely-task")]),
        make_proposal("mgr-b", vec![task("other")]),
        make_proposal("mgr-c", vec![task("other")]),
        make_proposal("mgr-d", vec![task("other")]),
    ];
    let engine = ConsensusEngine::new(weighted_config(0.5));
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // lonely-task: 1/4=0.25 < 0.5 -> dissenting
    let lonely = result
        .dissenting_tasks
        .iter()
        .find(|t| t.normalized_key == "lonely-task");
    assert!(lonely.is_some());

    // other: 3/4=0.75 >= 0.5 -> agreed
    let other = result
        .agreed_tasks
        .iter()
        .find(|t| t.normalized_key == "other");
    assert!(other.is_some());
}

#[test]
fn weighted_voting_above_threshold() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("popular")]),
        make_proposal("mgr-b", vec![task("popular")]),
        make_proposal("mgr-c", vec![task("popular")]),
        make_proposal("mgr-d", vec![task("unpopular")]),
    ];
    let engine = ConsensusEngine::new(weighted_config(0.5));
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // popular: 3/4=0.75 >= 0.5 -> agreed
    let popular = result
        .agreed_tasks
        .iter()
        .find(|t| t.normalized_key == "popular");
    assert!(popular.is_some());
    assert_eq!(popular.unwrap().vote_count, 3);
}

#[test]
fn weighted_voting_threshold_1_0_is_intersection() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("shared-task"), task("alpha-exclusive")]),
        make_proposal("mgr-b", vec![task("shared-task"), task("beta-exclusive")]),
    ];
    let engine = ConsensusEngine::new(weighted_config(1.0));
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // At threshold 1.0, only tasks with 100% votes pass -> only "shared-task" (2/2)
    assert_eq!(result.agreed_tasks.len(), 1);
    assert_eq!(result.agreed_tasks[0].normalized_key, "shared-task");
    assert_eq!(result.dissenting_tasks.len(), 2); // alpha-exclusive, beta-exclusive
}

#[test]
fn weighted_voting_threshold_near_zero_is_union() {
    let i18n = i18n();
    let proposals = vec![
        make_proposal("mgr-a", vec![task("task1"), task("task2")]),
        make_proposal("mgr-b", vec![task("task3")]),
    ];
    let engine = ConsensusEngine::new(weighted_config(0.01));
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // All tasks have vote ratio >= 0.01 (minimum is 1/2 = 0.5)
    assert_eq!(result.agreed_tasks.len(), 3);
    assert_eq!(result.dissenting_tasks.len(), 0);
}

// ========== Complexity Weighted Strategy Tests ==========

#[test]
fn complexity_weighted_favors_simple_high_vote() {
    let i18n = i18n();
    // Simple task (complexity=2) with 3/3 votes: score = 0.7*1.0 + 0.3*(1/2) = 0.7 + 0.15 = 0.85
    // Complex task (complexity=9) with 1/3 votes: score = 0.7*(1/3) + 0.3*(1/9) = 0.233 + 0.033 = 0.267
    let proposals = vec![
        make_proposal(
            "mgr-a",
            vec![
                task_with_complexity("simple", 2),
                task_with_complexity("complex", 9),
            ],
        ),
        make_proposal("mgr-b", vec![task_with_complexity("simple", 2)]),
        make_proposal("mgr-c", vec![task_with_complexity("simple", 2)]),
    ];
    let config = complexity_config(0.5, 0.7, 0.3);
    let engine = ConsensusEngine::new(config);
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // Simple task should be agreed (score ~0.85 >= 0.5)
    let simple = result
        .agreed_tasks
        .iter()
        .find(|t| t.normalized_key == "simple");
    assert!(simple.is_some());

    // Complex task should be dissenting (score ~0.267 < 0.5)
    let complex = result
        .dissenting_tasks
        .iter()
        .find(|t| t.normalized_key == "complex");
    assert!(complex.is_some());
}

#[test]
fn complexity_weighted_adjusts_by_weight() {
    let i18n = i18n();
    // With frequency_weight=0.9, vote ratio dominates
    // Task with 1/2 votes, complexity 5: score = 0.9*0.5 + 0.1*(1/5) = 0.45 + 0.02 = 0.47
    // With threshold 0.45, it should pass
    let proposals = vec![
        make_proposal("mgr-a", vec![task_with_complexity("moderate", 5)]),
        make_proposal("mgr-b", vec![task("other-task")]),
    ];
    let config = complexity_config(0.45, 0.9, 0.1);
    let engine = ConsensusEngine::new(config);
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    let moderate = result
        .agreed_tasks
        .iter()
        .find(|t| t.normalized_key == "moderate");
    assert!(moderate.is_some());
}

#[test]
fn complexity_weighted_min_complexity_is_1() {
    let i18n = i18n();
    // complexity 0 -> clamped to 1 via .max(1.0), so complexity_factor = 1.0/1.0 = 1.0
    // score = 0.7 * (1/1) + 0.3 * (1/1) = 0.7 + 0.3 = 1.0
    let proposals = vec![make_proposal(
        "mgr-a",
        vec![task_with_complexity("zero-complex", 0)],
    )];
    let config = complexity_config(0.5, 0.7, 0.3);
    let engine = ConsensusEngine::new(config);
    let result = engine.evaluate(&proposals, &i18n).unwrap();

    // With 1/1 votes and complexity clamped to 1, score = 1.0 >= 0.5
    assert_eq!(result.agreed_tasks.len(), 1);
    assert_eq!(result.agreed_tasks[0].normalized_key, "zero-complex");
}

// ========== Engine + Normalizer Tests ==========

#[test]
fn engine_dispatches_to_correct_strategy() {
    let i18n = i18n();
    let proposals = vec![make_proposal("mgr-a", vec![task("task1")])];

    // Union
    let union_engine = ConsensusEngine::new(union_config());
    let union_result = union_engine.evaluate(&proposals, &i18n).unwrap();
    assert_eq!(union_result.strategy, ConsensusStrategy::Union);

    // Intersection
    let int_engine = ConsensusEngine::new(intersection_config());
    let int_result = int_engine.evaluate(&proposals, &i18n).unwrap();
    assert_eq!(int_result.strategy, ConsensusStrategy::Intersection);

    // WeightedVoting
    let wv_engine = ConsensusEngine::new(weighted_config(0.5));
    let wv_result = wv_engine.evaluate(&proposals, &i18n).unwrap();
    assert_eq!(wv_result.strategy, ConsensusStrategy::WeightedVoting);

    // ComplexityWeighted
    let cw_engine = ConsensusEngine::new(complexity_config(0.5, 0.7, 0.3));
    let cw_result = cw_engine.evaluate(&proposals, &i18n).unwrap();
    assert_eq!(cw_result.strategy, ConsensusStrategy::ComplexityWeighted);
}

#[test]
fn normalize_task_key_variations() {
    let k1 = normalize_task_key("Fix Bug");
    let k2 = normalize_task_key("fix-bug");
    let k3 = normalize_task_key("FIX  BUG!");

    assert_eq!(k1, k2);
    assert_eq!(k2, k3);
    assert_eq!(k1, "fix-bug");
}

#[test]
fn group_matching_tasks_groups_correctly() {
    let proposals = vec![
        make_proposal("mgr-a", vec![task("Fix Bug"), task("Add Feature")]),
        make_proposal("mgr-b", vec![task("fix-bug"), task("Remove Legacy")]),
    ];
    let groups = group_matching_tasks(&proposals);

    // "Fix Bug" and "fix-bug" should normalize to the same key
    let fix_group = groups
        .iter()
        .find(|g| g.normalized_key == "fix-bug")
        .unwrap();
    assert_eq!(fix_group.vote_count(), 2);

    let add_group = groups
        .iter()
        .find(|g| g.normalized_key == "add-feature")
        .unwrap();
    assert_eq!(add_group.vote_count(), 1);

    let remove_group = groups
        .iter()
        .find(|g| g.normalized_key == "remove-legacy")
        .unwrap();
    assert_eq!(remove_group.vote_count(), 1);

    // Total groups: fix-bug, add-feature, remove-legacy
    assert_eq!(groups.len(), 3);
}

// ========== Fuzzy Deduplication Tests ==========

#[test]
fn jaccard_similarity_basic() {
    // Identical keys
    assert!((jaccard_token_similarity("foo-bar-baz", "foo-bar-baz") - 1.0).abs() < f64::EPSILON);

    // Completely disjoint
    assert!((jaccard_token_similarity("foo-bar", "baz-qux") - 0.0).abs() < f64::EPSILON);

    // Partial overlap: {foo, bar} ∩ {foo, baz} = {foo}, union = {foo, bar, baz} → 1/3
    let sim = jaccard_token_similarity("foo-bar", "foo-baz");
    assert!((sim - 1.0 / 3.0).abs() < 0.001);

    // Empty keys
    assert!((jaccard_token_similarity("", "") - 0.0).abs() < f64::EPSILON);
}

#[test]
fn file_overlap_ratio_basic() {
    // Identical file sets
    let a = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
    let b = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
    assert!((file_overlap_ratio(&a, &b) - 1.0).abs() < f64::EPSILON);

    // Completely disjoint
    let c = vec!["src/c.rs".to_string()];
    assert!((file_overlap_ratio(&a, &c) - 0.0).abs() < f64::EPSILON);

    // Subset: {a} ⊂ {a, b} → intersection=1, min_size=1 → 1.0
    let d = vec!["src/a.rs".to_string()];
    assert!((file_overlap_ratio(&a, &d) - 1.0).abs() < f64::EPSILON);

    // Empty sets
    let empty: Vec<String> = vec![];
    assert!((file_overlap_ratio(&empty, &a) - 0.0).abs() < f64::EPSILON);
    assert!((file_overlap_ratio(&empty, &empty) - 0.0).abs() < f64::EPSILON);
}

#[test]
fn merge_identical_files_similar_titles() {
    // 3 managers propose the same task with slightly different titles but same files
    let proposals = vec![
        make_proposal(
            "mgr-0",
            vec![task_with_files(
                "AcceptanceThreshold Config Struct",
                &["src/config/mod.rs", "src/config/threshold.rs"],
            )],
        ),
        make_proposal(
            "mgr-1",
            vec![task_with_files(
                "AcceptanceThreshold Config Struct and Validation Error Variant",
                &["src/config/mod.rs", "src/config/threshold.rs"],
            )],
        ),
        make_proposal(
            "mgr-2",
            vec![task_with_files(
                "AcceptanceThreshold Config Struct in mahalaxmi-core",
                &["src/config/mod.rs", "src/config/threshold.rs"],
            )],
        ),
    ];
    let groups = group_matching_tasks(&proposals);

    // All 3 should merge into 1 group (same files + overlapping title tokens)
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].entries.len(), 3);
}

#[test]
fn no_merge_different_files() {
    // Same title tokens but completely different files → should NOT merge
    let proposals = vec![
        make_proposal(
            "mgr-0",
            vec![task_with_files(
                "Config Struct Update",
                &["src/config/mod.rs"],
            )],
        ),
        make_proposal(
            "mgr-1",
            vec![task_with_files(
                "Config Struct Refactor",
                &["src/database/schema.rs"],
            )],
        ),
    ];
    let groups = group_matching_tasks(&proposals);

    // Different normalized keys, different files → 2 separate groups
    assert_eq!(groups.len(), 2);
}

#[test]
fn no_merge_different_titles_same_files() {
    // Same files but completely different titles (jaccard < 0.3) → should NOT merge
    let proposals = vec![
        make_proposal(
            "mgr-0",
            vec![task_with_files(
                "Add new validation field",
                &["src/config/mod.rs"],
            )],
        ),
        make_proposal(
            "mgr-1",
            vec![task_with_files(
                "Fix serialization bug",
                &["src/config/mod.rs"],
            )],
        ),
    ];
    let groups = group_matching_tasks(&proposals);

    // Same files but jaccard("add-new-validation-field", "fix-serialization-bug") = 0/8 = 0.0
    assert_eq!(groups.len(), 2);
}

#[test]
fn no_merge_empty_files() {
    // Semantically unrelated tasks with no affected_files should not merge.
    // Using distinct domain areas ensures title jaccard is near zero.
    let proposals = vec![
        make_proposal("mgr-0", vec![task("Authentication Token Refresh")]),
        make_proposal("mgr-1", vec![task("Database Index Optimization")]),
    ];
    let groups = group_matching_tasks(&proposals);

    // Completely different domains, empty files → 2 separate groups
    assert_eq!(groups.len(), 2);
}

#[test]
fn exact_key_still_works() {
    // Regression: exact-match deduplication must still work after adding fuzzy pass
    let proposals = vec![
        make_proposal("mgr-a", vec![task_with_files("Fix Bug", &["src/main.rs"])]),
        make_proposal("mgr-b", vec![task_with_files("fix-bug", &["src/main.rs"])]),
    ];
    let groups = group_matching_tasks(&proposals);

    // These normalize to the same key "fix-bug" → 1 group with 2 entries
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].vote_count(), 2);
}

#[test]
fn merge_chain() {
    // A overlaps B (file), B overlaps C (file) → all 3 merge into 1 group
    let proposals = vec![
        make_proposal(
            "mgr-0",
            vec![task_with_files(
                "Config Struct Alpha",
                &["src/config/mod.rs"],
            )],
        ),
        make_proposal(
            "mgr-1",
            vec![task_with_files(
                "Config Struct Beta",
                &["src/config/mod.rs", "src/config/beta.rs"],
            )],
        ),
        make_proposal(
            "mgr-2",
            vec![task_with_files(
                "Config Struct Gamma",
                &["src/config/beta.rs", "src/config/gamma.rs"],
            )],
        ),
    ];
    let groups = group_matching_tasks(&proposals);

    // A-B share mod.rs (overlap 1/1=1.0), B-C share beta.rs (overlap 1/2=0.5)
    // Title similarity: all share "config" and "struct" tokens → jaccard >= 0.3
    // So A merges with B, then that group merges with C → 1 group
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].entries.len(), 3);
}

#[test]
fn partial_file_overlap() {
    // Sharing 1 of 2 files (50% overlap) + good title similarity → merged
    let proposals = vec![
        make_proposal(
            "mgr-0",
            vec![task_with_files(
                "Config Struct Update",
                &["src/config/mod.rs", "src/config/types.rs"],
            )],
        ),
        make_proposal(
            "mgr-1",
            vec![task_with_files(
                "Config Struct Revision",
                &["src/config/mod.rs", "src/config/validate.rs"],
            )],
        ),
    ];
    let groups = group_matching_tasks(&proposals);

    // File overlap: 1/2 = 0.5 >= 0.5 ✓
    // Title: {config, struct, update} ∩ {config, struct, revision} = {config, struct}
    //        union = {config, struct, update, revision} → 2/4 = 0.5 >= 0.3 ✓
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].entries.len(), 2);
}

#[test]
fn below_file_threshold() {
    // Sharing 1 of 3 files (33% overlap) → NOT merged
    let proposals = vec![
        make_proposal(
            "mgr-0",
            vec![task_with_files(
                "Config Struct Update",
                &[
                    "src/config/mod.rs",
                    "src/config/types.rs",
                    "src/config/defaults.rs",
                ],
            )],
        ),
        make_proposal(
            "mgr-1",
            vec![task_with_files(
                "Config Struct Revision",
                &[
                    "src/config/mod.rs",
                    "src/config/validate.rs",
                    "src/config/schema.rs",
                ],
            )],
        ),
    ];
    let groups = group_matching_tasks(&proposals);

    // File overlap: 1/3 = 0.33 < 0.5 → NOT merged despite good title similarity
    assert_eq!(groups.len(), 2);
}
