use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ConsensusStrategy, DeveloperId, ManagerId};
use mahalaxmi_orchestration::models::{ConsensusConfiguration, ManagerProposal, ProposedTask};
use mahalaxmi_orchestration::ConsensusEngine;
use std::collections::HashMap;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

fn weighted_config(threshold: f64) -> ConsensusConfiguration {
    ConsensusConfiguration {
        strategy: ConsensusStrategy::WeightedVoting,
        minimum_agreement_threshold: threshold,
        frequency_weight: 0.7,
        complexity_weight: 0.3,
    }
}

fn task(title: &str) -> ProposedTask {
    ProposedTask::new(title, "")
}

/// Build a ManagerProposal attributed to a specific developer.
///
/// Sets `developer_id` and `developer_weight` on the proposal so the
/// `WeightedVotingStrategy` can use them when `developer_weights` is empty
/// (falling back to the embedded weight).
fn make_developer_proposal(
    manager_name: &str,
    dev_id: &str,
    developer_weight: f32,
    tasks: Vec<ProposedTask>,
) -> ManagerProposal {
    let mut p = ManagerProposal::new(ManagerId::new(manager_name), tasks, 1000);
    p.developer_id = Some(DeveloperId::from(dev_id));
    p.developer_weight = developer_weight;
    p
}

/// Two-developer weighted voting: alice (weight 2.0) proposes 3 unique tasks,
/// bob (weight 1.0) proposes 2 unique tasks.  With threshold 0.5 and total
/// weight 3.0, alice's tasks have weighted ratio 2/3 ≈ 0.667 (>= 0.5, agreed)
/// while bob's tasks have ratio 1/3 ≈ 0.333 (< 0.5, dissenting).
///
/// Verifies that the higher-weight developer's contributions appear in the
/// output plan and the lower-weight developer's do not.
#[test]
fn test_team_consensus_two_developers() {
    let i18n = i18n();

    let alice_proposal = make_developer_proposal(
        "alice",
        "alice",
        2.0,
        vec![
            task("alice-task-one"),
            task("alice-task-two"),
            task("alice-task-three"),
        ],
    );
    let bob_proposal = make_developer_proposal(
        "bob",
        "bob",
        1.0,
        vec![task("bob-task-one"), task("bob-task-two")],
    );
    let proposals = vec![alice_proposal, bob_proposal];

    let mut developer_weights: HashMap<DeveloperId, f32> = HashMap::new();
    developer_weights.insert(DeveloperId::from("alice"), 2.0);
    developer_weights.insert(DeveloperId::from("bob"), 1.0);

    let engine = ConsensusEngine::new(weighted_config(0.5));
    let result = engine.run(&proposals, developer_weights, &i18n).unwrap();

    // Alice's 3 tasks should be in agreed_tasks (weighted ratio = 2.0/3.0 = 0.667 >= 0.5)
    assert_eq!(
        result.agreed_tasks.len(),
        3,
        "expected alice's 3 tasks to pass consensus; got agreed: {:?}, dissenting: {:?}",
        result
            .agreed_tasks
            .iter()
            .map(|t| &t.normalized_key)
            .collect::<Vec<_>>(),
        result
            .dissenting_tasks
            .iter()
            .map(|t| &t.normalized_key)
            .collect::<Vec<_>>(),
    );

    let agreed_keys: Vec<&str> = result
        .agreed_tasks
        .iter()
        .map(|t| t.normalized_key.as_str())
        .collect();

    assert!(
        agreed_keys.contains(&"alice-task-one"),
        "alice-task-one must be in agreed tasks"
    );
    assert!(
        agreed_keys.contains(&"alice-task-two"),
        "alice-task-two must be in agreed tasks"
    );
    assert!(
        agreed_keys.contains(&"alice-task-three"),
        "alice-task-three must be in agreed tasks"
    );

    // Bob's 2 tasks should be dissenting (weighted ratio = 1.0/3.0 = 0.333 < 0.5)
    assert_eq!(
        result.dissenting_tasks.len(),
        2,
        "expected bob's 2 tasks to be dissenting"
    );

    let dissenting_keys: Vec<&str> = result
        .dissenting_tasks
        .iter()
        .map(|t| t.normalized_key.as_str())
        .collect();

    assert!(
        dissenting_keys.contains(&"bob-task-one"),
        "bob-task-one must be dissenting"
    );
    assert!(
        dissenting_keys.contains(&"bob-task-two"),
        "bob-task-two must be dissenting"
    );
}

/// Backward-compatibility test: an empty developer_weights map produces the
/// same consensus result as calling `ConsensusEngine::evaluate()` directly,
/// confirming that single-developer cycles are unaffected by the new API.
#[test]
fn test_team_consensus_single_developer_backward_compatible() {
    let i18n = i18n();

    // Two proposals: every task has a 50% vote ratio (1 of 2 managers agree).
    let proposals = vec![
        ManagerProposal::new(
            ManagerId::new("mgr-0"),
            vec![task("shared-task"), task("only-mgr0")],
            1000,
        ),
        ManagerProposal::new(
            ManagerId::new("mgr-1"),
            vec![task("shared-task"), task("only-mgr1")],
            1000,
        ),
    ];

    let engine = ConsensusEngine::new(weighted_config(0.5));

    // Reference result via the original evaluate() path (no weights).
    let reference = engine.evaluate(&proposals, &i18n).unwrap();

    // Result via run() with an empty weight map must match.
    let run_result = engine.run(&proposals, HashMap::new(), &i18n).unwrap();

    assert_eq!(
        reference.agreed_tasks.len(),
        run_result.agreed_tasks.len(),
        "agreed task count must match between evaluate() and run(empty)"
    );
    assert_eq!(
        reference.dissenting_tasks.len(),
        run_result.dissenting_tasks.len(),
        "dissenting task count must match between evaluate() and run(empty)"
    );

    // Verify the set of agreed task keys is identical.
    let mut ref_keys: Vec<&str> = reference
        .agreed_tasks
        .iter()
        .map(|t| t.normalized_key.as_str())
        .collect();
    let mut run_keys: Vec<&str> = run_result
        .agreed_tasks
        .iter()
        .map(|t| t.normalized_key.as_str())
        .collect();
    ref_keys.sort_unstable();
    run_keys.sort_unstable();

    assert_eq!(
        ref_keys, run_keys,
        "agreed task keys must be identical between evaluate() and run(empty)"
    );

    // Verify metrics are identical.
    assert_eq!(
        reference.metrics.total_proposals,
        run_result.metrics.total_proposals
    );
    assert_eq!(
        reference.metrics.successful_proposals,
        run_result.metrics.successful_proposals
    );
    assert_eq!(
        reference.metrics.agreed_task_count,
        run_result.metrics.agreed_task_count
    );
    assert_eq!(
        reference.metrics.dissenting_task_count,
        run_result.metrics.dissenting_task_count
    );
}
