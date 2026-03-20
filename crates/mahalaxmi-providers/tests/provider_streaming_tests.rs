//! Tests for streaming_args() and extract_response() across all providers.

use std::path::Path;

use mahalaxmi_providers::tier1;
use mahalaxmi_providers::{
    AiProvider, ClaudeCodeProvider, GeminiProvider, GenericCliProvider, MockProvider, OutputMarkers,
};

fn test_markers() -> OutputMarkers {
    OutputMarkers::new(r"\$\s*$", r"(?i)error:", r">\s*$").unwrap()
}

// ===========================================================================
// Claude streaming_args()
// ===========================================================================

#[test]
fn claude_streaming_args_returns_verbose_and_stream_json() {
    let provider = ClaudeCodeProvider::new();
    let args = provider
        .streaming_args()
        .expect("Claude should return streaming args");
    assert_eq!(args, vec!["--verbose", "--output-format", "stream-json"]);
}

#[test]
fn claude_streaming_args_has_three_elements() {
    let provider = ClaudeCodeProvider::new();
    let args = provider.streaming_args().unwrap();
    assert_eq!(args.len(), 3);
}

// ===========================================================================
// Non-Claude providers return None
// ===========================================================================

#[test]
fn gemini_streaming_args_is_none() {
    let provider = GeminiProvider::new();
    assert!(provider.streaming_args().is_none());
}

#[test]
fn mock_streaming_args_is_none() {
    let provider = MockProvider::new();
    assert!(provider.streaming_args().is_none());
}

#[test]
fn generic_streaming_args_is_none() {
    let provider = GenericCliProvider::new("test", "Test", "test-cli", test_markers());
    assert!(provider.streaming_args().is_none());
}

#[test]
fn kiro_streaming_args_is_none() {
    let provider = tier1::kiro_provider();
    assert!(provider.streaming_args().is_none());
}

#[test]
fn goose_streaming_args_is_none() {
    let provider = tier1::goose_provider();
    assert!(provider.streaming_args().is_none());
}

#[test]
fn deepseek_streaming_args_is_none() {
    let provider = tier1::deepseek_provider();
    assert!(provider.streaming_args().is_none());
}

#[test]
fn qwen_streaming_args_is_none() {
    let provider = tier1::qwen_provider();
    assert!(provider.streaming_args().is_none());
}

#[test]
fn opencode_streaming_args_is_none() {
    let provider = tier1::opencode_provider();
    assert!(provider.streaming_args().is_none());
}

#[test]
fn cody_streaming_args_is_none() {
    let provider = tier1::cody_provider();
    assert!(provider.streaming_args().is_none());
}

// ===========================================================================
// Claude extract_response() — realistic stream-json parsing
// ===========================================================================

#[test]
fn claude_extract_response_parses_result_event() {
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"assistant","message":{"id":"msg_01","type":"message"}}
{"type":"content_block_start","index":0}
{"type":"content_block_delta","index":0,"delta":{"text":"Hello"}}
{"type":"content_block_stop","index":0}
{"type":"result","result":"Here is the implementation:\n\nfn main() {}","subtype":"text","cost_usd":0.01,"duration_ms":5000}"#;
    let result = provider.extract_response(output);
    assert_eq!(result, "Here is the implementation:\n\nfn main() {}");
}

#[test]
fn claude_extract_response_empty_output_returns_empty() {
    let provider = ClaudeCodeProvider::new();
    let result = provider.extract_response("");
    assert_eq!(result, "");
}

#[test]
fn claude_extract_response_no_result_event_returns_raw() {
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"content_block_delta","index":0,"delta":{"text":"partial"}}
{"type":"content_block_stop","index":0}"#;
    let result = provider.extract_response(output);
    assert_eq!(result, output);
}

#[test]
fn claude_extract_response_partial_json_returns_raw() {
    let provider = ClaudeCodeProvider::new();
    let output = "not json at all\njust plain text";
    let result = provider.extract_response(output);
    assert_eq!(result, output);
}

#[test]
fn claude_extract_response_unicode_content() {
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"result","result":"日本語テスト ñ é ü 🦀","subtype":"text"}"#;
    let result = provider.extract_response(output);
    assert_eq!(result, "日本語テスト ñ é ü 🦀");
}

#[test]
fn claude_extract_response_multiline_result() {
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"result","result":"line1\nline2\nline3","subtype":"text"}"#;
    let result = provider.extract_response(output);
    assert_eq!(result, "line1\nline2\nline3");
}

#[test]
fn claude_extract_response_takes_last_result_event() {
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"result","result":"first attempt","subtype":"text"}
{"type":"result","result":"final answer","subtype":"text"}"#;
    let result = provider.extract_response(output);
    // Scanning from the end, takes the last result event
    assert_eq!(result, "final answer");
}

#[test]
fn claude_extract_response_empty_result_field() {
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"result","result":"","subtype":"text"}"#;
    let result = provider.extract_response(output);
    assert_eq!(result, "");
}

#[test]
fn claude_extract_response_result_with_special_chars() {
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"result","result":"code: `fn main() { println!(\"hello\"); }`","subtype":"text"}"#;
    let result = provider.extract_response(output);
    assert!(result.contains("fn main()"));
}

#[test]
fn claude_extract_response_skips_blank_lines() {
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"result","result":"the answer","subtype":"text"}

"#;
    let result = provider.extract_response(output);
    assert_eq!(result, "the answer");
}

// ===========================================================================
// Non-Claude extract_response() returns as-is
// ===========================================================================

#[test]
fn gemini_extract_response_returns_raw() {
    let provider = GeminiProvider::new();
    let output = "Some Gemini output\nwith multiple lines";
    assert_eq!(provider.extract_response(output), output);
}

#[test]
fn mock_extract_response_returns_raw() {
    let provider = MockProvider::new();
    let output = "echo output";
    assert_eq!(provider.extract_response(output), output);
}

#[test]
fn generic_extract_response_returns_raw() {
    let provider = GenericCliProvider::new("test", "Test", "test-cli", test_markers());
    let output = "generic output";
    assert_eq!(provider.extract_response(output), output);
}

// ===========================================================================
// Object safety — both methods work via Box<dyn AiProvider>
// ===========================================================================

#[test]
fn streaming_args_via_trait_object_claude() {
    let provider: Box<dyn AiProvider> = Box::new(ClaudeCodeProvider::new());
    assert!(provider.streaming_args().is_some());
}

#[test]
fn streaming_args_via_trait_object_mock() {
    let provider: Box<dyn AiProvider> = Box::new(MockProvider::new());
    assert!(provider.streaming_args().is_none());
}

#[test]
fn extract_response_via_trait_object() {
    let provider: Box<dyn AiProvider> = Box::new(ClaudeCodeProvider::new());
    let output = r#"{"type":"result","result":"via trait object","subtype":"text"}"#;
    assert_eq!(provider.extract_response(output), "via trait object");
}

// ===========================================================================
// extract_response() length behavior (used by driver extraction_changed logic)
// ===========================================================================

#[test]
fn claude_extract_response_changes_output_length() {
    // Parsing stream-json with a result event produces shorter output than raw.
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"assistant","message":{"id":"msg_01","type":"message"}}
{"type":"content_block_start","index":0}
{"type":"content_block_delta","index":0,"delta":{"text":"Hello"}}
{"type":"content_block_stop","index":0}
{"type":"result","result":"Hello world","subtype":"text","cost_usd":0.01,"duration_ms":5000}"#;
    let clean = provider.extract_response(output);
    assert!(
        clean.len() < output.len(),
        "extraction should produce shorter output (clean={}, raw={})",
        clean.len(),
        output.len()
    );
    assert_eq!(clean, "Hello world");
}

#[test]
fn non_claude_extract_response_preserves_length() {
    // Default trait impl returns identical output — length unchanged.
    let provider = GeminiProvider::new();
    let output = "Some Gemini output\nwith multiple lines\nand more content";
    let clean = provider.extract_response(output);
    assert_eq!(
        clean.len(),
        output.len(),
        "non-Claude extraction should preserve length"
    );
    assert_eq!(clean, output);
}

#[test]
fn claude_extract_response_fallback_preserves_length() {
    // No result event → falls back to raw output, length unchanged.
    let provider = ClaudeCodeProvider::new();
    let output = r#"{"type":"content_block_delta","index":0,"delta":{"text":"partial"}}
{"type":"content_block_stop","index":0}"#;
    let clean = provider.extract_response(output);
    assert_eq!(
        clean.len(),
        output.len(),
        "fallback extraction should preserve length"
    );
    assert_eq!(clean, output);
}

// ===========================================================================
// Integration: build_command() + streaming_args() produce correct full command
// ===========================================================================

#[test]
fn claude_build_command_plus_streaming_args_produces_correct_args() {
    let provider = ClaudeCodeProvider::new();
    let mut cmd = provider
        .build_command(Path::new("/project"), "fix the bug")
        .unwrap();
    if let Some(streaming) = provider.streaming_args() {
        for arg in streaming {
            cmd.args.push(arg);
        }
    }
    // Verify the full arg list
    assert_eq!(cmd.program, "claude");
    assert!(cmd.args.contains(&"--print".to_string()));
    assert!(cmd
        .args
        .contains(&"--dangerously-skip-permissions".to_string()));
    assert!(cmd.args.contains(&"fix the bug".to_string()));
    assert!(cmd.args.contains(&"--verbose".to_string()));
    assert!(cmd.args.contains(&"--output-format".to_string()));
    assert!(cmd.args.contains(&"stream-json".to_string()));
    // streaming args should be at the end: --verbose --output-format stream-json
    let v_idx = cmd.args.iter().position(|a| a == "--verbose").unwrap();
    let of_idx = cmd
        .args
        .iter()
        .position(|a| a == "--output-format")
        .unwrap();
    let sj_idx = cmd.args.iter().position(|a| a == "stream-json").unwrap();
    assert_eq!(of_idx, v_idx + 1);
    assert_eq!(sj_idx, of_idx + 1);
}

#[test]
fn gemini_build_command_plus_streaming_args_unchanged() {
    let provider = GeminiProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "task")
        .unwrap();
    let original_count = cmd.args.len();
    // streaming_args is None so no additional args
    assert!(provider.streaming_args().is_none());
    // Verify args count unchanged
    assert_eq!(cmd.args.len(), original_count);
}

// ===========================================================================
// Claude extract_response() — agentic stream-json format (--verbose)
// ===========================================================================

#[test]
fn claude_extract_response_agentic_text_blocks() {
    let provider = ClaudeCodeProvider::new();
    // Simulate agentic stream with assistant messages containing text content blocks.
    let output = r#"{"type":"system","message":{"content":"You are Claude."}}
{"type":"assistant","message":{"id":"msg_01","type":"message","content":[{"type":"text","text":"I'll analyze the codebase and create an execution plan."}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"tu_01","content":"file contents here"}]}}
{"type":"assistant","message":{"id":"msg_02","type":"message","content":[{"type":"tool_use","id":"tu_02","name":"read_file","input":{"path":"src/main.rs"}},{"type":"text","text":"Here is the execution plan:\n\n```json\n{\"tasks\": [{\"id\": 1}]}\n```"}]}}
{"subtype":"usage","totalDurationMs":83357}"#;

    let result = provider.extract_response(output);
    assert!(result.contains("I'll analyze the codebase"));
    assert!(result.contains("execution plan"));
    assert!(result.contains("tasks"));
    // Two text blocks joined with \n\n
    assert!(result.contains("\n\n"));
}

#[test]
fn claude_extract_response_agentic_no_text_returns_raw() {
    let provider = ClaudeCodeProvider::new();
    // Simulate agentic stream where assistant messages have only tool_use (no text blocks).
    let output = r#"{"type":"system","message":{"content":"You are Claude."}}
{"type":"assistant","message":{"id":"msg_01","type":"message","content":[{"type":"tool_use","id":"tu_01","name":"read_file","input":{"path":"src/main.rs"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"tu_01","content":"fn main() {}"}]}}
{"subtype":"usage","totalDurationMs":5000}"#;

    let result = provider.extract_response(output);
    // No text blocks → Tier 3 fallback returns raw output
    assert_eq!(result, output);
}

#[test]
fn claude_extract_response_agentic_mixed_tool_and_text() {
    let provider = ClaudeCodeProvider::new();
    // First assistant message: tool_use only (no text). Second: has text.
    let output = r#"{"type":"assistant","message":{"id":"msg_01","type":"message","content":[{"type":"tool_use","id":"tu_01","name":"read_file","input":{"path":"src/lib.rs"}}]}}
{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"tu_01","content":"pub fn hello() {}"}]}}
{"type":"assistant","message":{"id":"msg_02","type":"message","content":[{"type":"text","text":"The final answer is 42."}]}}
{"subtype":"usage","totalDurationMs":10000}"#;

    let result = provider.extract_response(output);
    assert_eq!(result, "The final answer is 42.");
}

#[test]
fn claude_extract_response_prefers_result_over_agentic() {
    let provider = ClaudeCodeProvider::new();
    // Stream has both assistant text blocks AND a result event.
    // Tier 1 (result event) should take priority.
    let output = r#"{"type":"assistant","message":{"id":"msg_01","type":"message","content":[{"type":"text","text":"Agentic text should be ignored"}]}}
{"type":"result","result":"Result event wins","subtype":"text","cost_usd":0.01,"duration_ms":5000}"#;

    let result = provider.extract_response(output);
    assert_eq!(result, "Result event wins");
}

// ===========================================================================
// stream_complete_marker()
// ===========================================================================

#[test]
fn claude_stream_complete_marker_returns_total_duration() {
    let provider = ClaudeCodeProvider::new();
    let marker = provider.stream_complete_marker();
    assert_eq!(marker, Some("\"totalDurationMs\":"));
}

#[test]
fn non_claude_stream_complete_marker_returns_none() {
    assert!(GeminiProvider::new().stream_complete_marker().is_none());
    assert!(MockProvider::new().stream_complete_marker().is_none());
    assert!(
        GenericCliProvider::new("test", "Test", "test-cli", test_markers())
            .stream_complete_marker()
            .is_none()
    );
}

#[test]
fn stream_complete_marker_via_trait_object() {
    let claude: Box<dyn AiProvider> = Box::new(ClaudeCodeProvider::new());
    assert!(claude.stream_complete_marker().is_some());
    let mock: Box<dyn AiProvider> = Box::new(MockProvider::new());
    assert!(mock.stream_complete_marker().is_none());
}

// ===========================================================================
// stream_init_marker()
// ===========================================================================

#[test]
fn claude_stream_init_marker_returns_subtype_init() {
    let provider = ClaudeCodeProvider::new();
    let marker = provider.stream_init_marker();
    assert_eq!(marker, Some("\"subtype\":\"init\""));
}

#[test]
fn non_claude_stream_init_marker_returns_none() {
    assert!(GeminiProvider::new().stream_init_marker().is_none());
    assert!(MockProvider::new().stream_init_marker().is_none());
    assert!(
        GenericCliProvider::new("test", "Test", "test-cli", test_markers())
            .stream_init_marker()
            .is_none()
    );
    assert!(tier1::kiro_provider().stream_init_marker().is_none());
    assert!(tier1::goose_provider().stream_init_marker().is_none());
    assert!(tier1::deepseek_provider().stream_init_marker().is_none());
}

#[test]
fn stream_init_marker_via_trait_object() {
    let claude: Box<dyn AiProvider> = Box::new(ClaudeCodeProvider::new());
    assert_eq!(claude.stream_init_marker(), Some("\"subtype\":\"init\""));
    let mock: Box<dyn AiProvider> = Box::new(MockProvider::new());
    assert!(mock.stream_init_marker().is_none());
}

// ===========================================================================
// validate_stream_completion()
// ===========================================================================

#[test]
fn claude_validate_stream_completion_accepts_result_event() {
    let provider = ClaudeCodeProvider::new();
    // A line with type=result AND totalDurationMs is a true completion signal.
    let line = r#"{"type":"result","result":"done","subtype":"text","totalDurationMs":83357,"totalTokens":50000}"#;
    assert!(provider.validate_stream_completion(line));
}

#[test]
fn claude_validate_stream_completion_rejects_user_event() {
    let provider = ClaudeCodeProvider::new();
    // A line with type=user that happens to contain totalDurationMs is NOT completion.
    let line =
        r#"{"type":"user","message":{"content":[{"type":"tool_result"}]},"totalDurationMs":12345}"#;
    assert!(!provider.validate_stream_completion(line));
}

#[test]
fn non_claude_validate_stream_completion_always_true() {
    // Default trait impl returns true regardless of line content.
    let gemini = GeminiProvider::new();
    assert!(gemini.validate_stream_completion("anything"));

    let mock = MockProvider::new();
    assert!(mock.validate_stream_completion(""));

    let generic = GenericCliProvider::new("test", "Test", "test-cli", test_markers());
    assert!(generic.validate_stream_completion("some marker line"));
}
