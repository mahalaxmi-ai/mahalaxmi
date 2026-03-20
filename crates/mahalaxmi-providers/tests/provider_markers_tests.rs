//! Marker regex validation against realistic terminal output.

use mahalaxmi_providers::tier1;
use mahalaxmi_providers::{
    AiProvider, ClaudeCodeProvider, GeminiProvider, MockProvider, ProviderRegistry,
};

// ===========================================================================
// Claude Code markers
// ===========================================================================

#[test]
fn claude_completion_marker_matches_shell_prompt() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers.completion_marker.is_match("user@host:~/project$ "));
}

#[test]
fn claude_completion_marker_matches_bare_dollar() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers.completion_marker.is_match("$ "));
}

#[test]
fn claude_error_marker_matches_error_keyword() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers.error_marker.is_match("Error: something went wrong"));
}

#[test]
fn claude_error_marker_matches_fatal() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers.error_marker.is_match("fatal: not a git repository"));
}

#[test]
fn claude_error_marker_matches_failed() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers.error_marker.is_match("Build failed with 3 errors"));
}

#[test]
fn claude_error_marker_case_insensitive() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers.error_marker.is_match("ERROR: compilation aborted"));
    assert!(markers.error_marker.is_match("FATAL: out of memory"));
    assert!(markers.error_marker.is_match("FAILED: cannot find module"));
}

#[test]
fn claude_prompt_marker_matches_angle_bracket() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers.prompt_marker.is_match("> "));
}

#[test]
fn claude_prompt_marker_matches_waiting_for_input() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers.prompt_marker.is_match("waiting for input"));
}

// ===========================================================================
// False positive checks — normal output should NOT trigger completion
// ===========================================================================

#[test]
fn claude_completion_dollar_in_code_no_false_positive() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    // A dollar sign in the middle of code output should not match (no trailing $)
    assert!(!markers.completion_marker.is_match("let cost = $50;"));
}

#[test]
fn claude_completion_dollar_in_variable_no_false_positive() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(!markers.completion_marker.is_match("echo $HOME"));
}

#[test]
fn claude_error_marker_no_false_positive_on_normal_text() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(!markers.error_marker.is_match("Everything is fine"));
    assert!(!markers
        .error_marker
        .is_match("The function returns a value"));
}

// ===========================================================================
// Gemini markers
// ===========================================================================

#[test]
fn gemini_completion_marker_matches_signal() {
    let provider = GeminiProvider::new();
    let markers = provider.output_markers();
    assert!(markers.completion_marker.is_match("GEMINI_COMPLETE"));
}

#[test]
fn gemini_error_marker_matches_error() {
    let provider = GeminiProvider::new();
    let markers = provider.output_markers();
    assert!(markers.error_marker.is_match("Error: invalid API key"));
}

#[test]
fn gemini_error_marker_matches_fatal() {
    let provider = GeminiProvider::new();
    let markers = provider.output_markers();
    assert!(markers.error_marker.is_match("fatal: permission denied"));
}

#[test]
fn gemini_prompt_marker_matches() {
    let provider = GeminiProvider::new();
    let markers = provider.output_markers();
    assert!(markers.prompt_marker.is_match("> "));
}

// ===========================================================================
// Mock provider markers
// ===========================================================================

#[test]
fn mock_completion_marker_matches_dollar() {
    let provider = MockProvider::new();
    let markers = provider.output_markers();
    assert!(markers.completion_marker.is_match("$ "));
}

#[test]
fn mock_error_marker_matches_error_colon() {
    let provider = MockProvider::new();
    let markers = provider.output_markers();
    assert!(markers.error_marker.is_match("Error: test failure"));
}

#[test]
fn mock_prompt_marker_matches_angle() {
    let provider = MockProvider::new();
    let markers = provider.output_markers();
    assert!(markers.prompt_marker.is_match("> "));
}

// ===========================================================================
// Tier 1 default markers (shared across all tier1 providers)
// ===========================================================================

fn assert_tier1_markers_valid(provider: &dyn AiProvider) {
    let markers = provider.output_markers();

    // Completion: shell prompt with trailing $
    assert!(
        markers.completion_marker.is_match("user@host:~$ "),
        "{}: completion marker should match shell prompt",
        provider.name()
    );

    // Error: standard error keywords
    assert!(
        markers.error_marker.is_match("error: something went wrong"),
        "{}: error marker should match 'error'",
        provider.name()
    );
    assert!(
        markers.error_marker.is_match("fatal: out of memory"),
        "{}: error marker should match 'fatal'",
        provider.name()
    );
    assert!(
        markers.error_marker.is_match("Build failed"),
        "{}: error marker should match 'failed'",
        provider.name()
    );

    // Prompt: waiting for input
    assert!(
        markers.prompt_marker.is_match("> "),
        "{}: prompt marker should match '> '",
        provider.name()
    );
}

#[test]
fn kiro_markers_valid() {
    assert_tier1_markers_valid(&tier1::kiro_provider());
}

#[test]
fn goose_markers_valid() {
    assert_tier1_markers_valid(&tier1::goose_provider());
}

#[test]
fn deepseek_markers_valid() {
    assert_tier1_markers_valid(&tier1::deepseek_provider());
}

#[test]
fn qwen_markers_valid() {
    assert_tier1_markers_valid(&tier1::qwen_provider());
}

#[test]
fn opencode_markers_valid() {
    assert_tier1_markers_valid(&tier1::opencode_provider());
}

#[test]
fn cody_markers_valid() {
    assert_tier1_markers_valid(&tier1::cody_provider());
}

// ===========================================================================
// Error markers against realistic stack traces
// ===========================================================================

#[test]
fn error_marker_matches_rust_panic() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    let rust_panic = "thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/main.rs:42\nfatal runtime error: stack overflow";
    assert!(markers.error_marker.is_match(rust_panic));
}

#[test]
fn error_marker_matches_npm_error() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    assert!(markers
        .error_marker
        .is_match("npm ERR! Failed to install package"));
}

#[test]
fn error_marker_matches_python_traceback_with_error() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    let traceback = "Traceback (most recent call last):\n  File \"app.py\", line 10\nRuntimeError: division by zero";
    assert!(markers.error_marker.is_match(traceback));
}

// ===========================================================================
// Multi-line output matching
// ===========================================================================

#[test]
fn completion_marker_in_multiline_output() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    let output = "output line 1\noutput line 2\nuser@host:~/project$ ";
    // The regex matches per-line (when used by the detection engine)
    assert!(markers.completion_marker.is_match(output));
}

#[test]
fn error_marker_in_multiline_output() {
    let provider = ClaudeCodeProvider::new();
    let markers = provider.output_markers();
    let output =
        "Building...\nCompiling...\nerror[E0308]: mismatched types\n  --> src/main.rs:5:20";
    assert!(markers.error_marker.is_match(output));
}

// ===========================================================================
// All registered providers have valid markers
// ===========================================================================

fn full_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(ClaudeCodeProvider::new()));
    registry.register(Box::new(GeminiProvider::new()));
    registry.register(Box::new(MockProvider::new()));
    tier1::register_tier1_providers(&mut registry);
    registry
}

#[test]
fn all_providers_have_non_empty_completion_marker() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let markers = provider.output_markers();
        assert!(
            !markers.completion_marker.as_str().is_empty(),
            "Provider '{}' has empty completion marker",
            provider.name()
        );
    }
}

#[test]
fn all_providers_have_non_empty_error_marker() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let markers = provider.output_markers();
        assert!(
            !markers.error_marker.as_str().is_empty(),
            "Provider '{}' has empty error marker",
            provider.name()
        );
    }
}

#[test]
fn all_providers_have_non_empty_prompt_marker() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let markers = provider.output_markers();
        assert!(
            !markers.prompt_marker.as_str().is_empty(),
            "Provider '{}' has empty prompt marker",
            provider.name()
        );
    }
}
