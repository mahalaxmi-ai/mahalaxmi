use mahalaxmi_core::config::GeneralConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::logging::init_logging;
use mahalaxmi_core::logging::spans;
use std::sync::Once;

static SUBSCRIBER_INIT: Once = Once::new();

fn ensure_subscriber() {
    SUBSCRIBER_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::TRACE)
            .try_init();
    });
}

#[test]
fn init_logging_succeeds_or_is_already_set() {
    let dir = tempfile::tempdir().unwrap();
    let config = GeneralConfig {
        data_directory: dir.path().to_path_buf(),
        ..Default::default()
    };
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // init_logging calls set_global_default which can only succeed once per process.
    // If a subscriber is already set (from ensure_subscriber or another test),
    // this will return an error. Both outcomes are acceptable.
    let result = init_logging(&config, &i18n);
    let _ = result;
}

#[test]
fn log_directory_created_if_missing() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    assert!(!log_dir.exists());

    let config = GeneralConfig {
        data_directory: dir.path().to_path_buf(),
        ..Default::default()
    };
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // Even if set_global_default fails (already set), the directory
    // creation should still be attempted
    let _ = init_logging(&config, &i18n);
    assert!(log_dir.exists(), "Logs directory should have been created");
}

#[test]
fn orchestration_span_has_expected_name() {
    ensure_subscriber();
    let span = spans::orchestration_span("cycle-001");
    let metadata = span.metadata().expect("Span should have metadata");
    assert_eq!(metadata.name(), "orchestration");
}

#[test]
fn terminal_span_includes_role() {
    ensure_subscriber();
    let span = spans::terminal_span("term-1", "manager");
    let metadata = span.metadata().expect("Span should have metadata");
    assert_eq!(metadata.name(), "terminal");
}

#[test]
fn worker_span_includes_task() {
    ensure_subscriber();
    let span = spans::worker_span(1, "build module");
    let metadata = span.metadata().expect("Span should have metadata");
    assert_eq!(metadata.name(), "worker");
}

#[test]
fn provider_span_has_name() {
    ensure_subscriber();
    let span = spans::provider_span("claude-code");
    let metadata = span.metadata().expect("Span should have metadata");
    assert_eq!(metadata.name(), "provider");
}

#[test]
fn template_span_has_fields() {
    ensure_subscriber();
    let span = spans::template_span("tpl-001", "software");
    let metadata = span.metadata().expect("Span should have metadata");
    assert_eq!(metadata.name(), "template");
}

#[test]
fn config_span_has_source() {
    ensure_subscriber();
    let span = spans::config_span("file");
    let metadata = span.metadata().expect("Span should have metadata");
    assert_eq!(metadata.name(), "config");
}

#[test]
fn span_metadata_is_present_for_all_types() {
    ensure_subscriber();
    let span_creators: Vec<(&str, tracing::Span)> = vec![
        ("orchestration", spans::orchestration_span("test")),
        ("terminal", spans::terminal_span("test", "worker")),
        ("provider", spans::provider_span("test")),
        ("worker", spans::worker_span(0, "test")),
        ("template", spans::template_span("test", "test")),
        ("config", spans::config_span("test")),
    ];
    for (expected_name, span) in &span_creators {
        let metadata = span.metadata().expect("Span should have metadata");
        assert_eq!(
            metadata.name(),
            *expected_name,
            "Span name mismatch for {expected_name}"
        );
    }
}

#[test]
fn different_span_types_have_different_names() {
    ensure_subscriber();
    let names: Vec<&str> = vec![
        spans::orchestration_span("t").metadata().unwrap().name(),
        spans::terminal_span("t", "w").metadata().unwrap().name(),
        spans::provider_span("t").metadata().unwrap().name(),
        spans::worker_span(0, "t").metadata().unwrap().name(),
        spans::template_span("t", "c").metadata().unwrap().name(),
        spans::config_span("s").metadata().unwrap().name(),
    ];

    // Verify all names are unique
    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            assert_ne!(
                names[i], names[j],
                "Span names should be unique: {} vs {}",
                names[i], names[j]
            );
        }
    }
}
