use mahalaxmi_core::types::RootCauseCategory;
use mahalaxmi_detection::errors::analysis::{
    analyze_errors, categorize_error, normalize_error_message,
};
use mahalaxmi_detection::{ErrorCluster, RecurringError};

#[test]
fn recurring_error_new() {
    let error = RecurringError::new("something failed");
    assert_eq!(error.occurrence_count, 1);
    assert_eq!(error.normalized_message, "something failed");
    assert_eq!(error.occurrences.len(), 1);
    assert_eq!(error.category, RootCauseCategory::Unknown);
}

#[test]
fn recurring_error_record_occurrence() {
    let mut error = RecurringError::new("something failed");
    assert_eq!(error.occurrence_count, 1);
    error.record_occurrence();
    assert_eq!(error.occurrence_count, 2);
    assert_eq!(error.occurrences.len(), 2);
}

#[test]
fn error_cluster_total_occurrences() {
    let mut err1 = RecurringError::new("error one");
    err1.record_occurrence();
    err1.record_occurrence();
    // err1 now has count 3

    let mut err2 = RecurringError::new("error two");
    err2.record_occurrence();
    err2.record_occurrence();
    err2.record_occurrence();
    err2.record_occurrence();
    // err2 now has count 5

    let cluster = ErrorCluster::new("test-cluster")
        .with_error(err1)
        .with_error(err2);

    assert_eq!(cluster.total_occurrences(), 8);
}

#[test]
fn normalize_error_message_strips_paths() {
    let message = "Error in /home/user/file.rs: compilation failed";
    let normalized = normalize_error_message(message);
    assert!(normalized.contains("<path>"));
    assert!(!normalized.contains("/home/user/file.rs"));
}

#[test]
fn normalize_error_message_strips_line_numbers() {
    let message = "Error at line 42: unexpected token";
    let normalized = normalize_error_message(message);
    assert!(normalized.contains("line <N>"));
    assert!(!normalized.contains("line 42"));
}

#[test]
fn categorize_error_authentication() {
    let category = categorize_error("authentication failed for user");
    assert_eq!(category, RootCauseCategory::Authentication);
}

#[test]
fn categorize_error_unknown() {
    let category = categorize_error("something weird happened");
    assert_eq!(category, RootCauseCategory::Unknown);
}

#[test]
fn analyze_errors_groups_recurring() {
    let messages: Vec<&str> = vec![
        "Error in /home/user/a.rs: compilation failed",
        "Error in /home/user/b.rs: compilation failed",
        "Error in /home/user/c.rs: compilation failed",
        "permission denied accessing resource",
        "authentication failed for user",
    ];

    let analysis = analyze_errors(&messages);

    // The three compilation errors should normalize to the same message
    // and be grouped as one recurring error with count 3.
    let recurring_compilation = analysis
        .recurring_errors
        .iter()
        .find(|e| e.occurrence_count == 3);
    assert!(
        recurring_compilation.is_some(),
        "Expected a recurring error with count 3, got errors: {:?}",
        analysis
            .recurring_errors
            .iter()
            .map(|e| (&e.normalized_message, e.occurrence_count))
            .collect::<Vec<_>>()
    );

    // Total distinct error patterns should be 3 (compilation x3, permission x1, auth x1)
    assert_eq!(analysis.recurring_errors.len(), 3);

    // A hypothesis should be generated for the recurring error (count >= 2)
    assert!(!analysis.hypotheses.is_empty());
}
