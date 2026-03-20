//! Cross-provider build_command() validation tests.

use std::path::Path;

use mahalaxmi_providers::tier1;
use mahalaxmi_providers::{
    AiProvider, ClaudeCodeProvider, GeminiProvider, GenericCliProvider, MockProvider,
    OutputMarkers, ProviderRegistry,
};

fn test_markers() -> OutputMarkers {
    OutputMarkers::new(r"\$\s*$", r"(?i)error:", r">\s*$").unwrap()
}

// ===========================================================================
// Claude Code build_command() validation
// ===========================================================================

#[test]
fn claude_command_has_print_flag() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert!(cmd.args.contains(&"--print".to_string()));
}

#[test]
fn claude_command_has_skip_permissions() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert!(cmd
        .args
        .contains(&"--dangerously-skip-permissions".to_string()));
}

#[test]
fn claude_command_prompt_is_last_positional() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "my prompt text")
        .unwrap();
    // Prompt should be the last arg from build_command (before streaming args are appended)
    assert_eq!(cmd.args.last().unwrap(), "my prompt text");
}

#[test]
fn claude_command_no_stdin_data() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert!(cmd.stdin_data.is_none());
}

#[test]
fn claude_command_program_is_claude() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert_eq!(cmd.program, "claude");
}

#[test]
fn claude_command_working_dir_set() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/my/project"), "prompt")
        .unwrap();
    assert_eq!(
        cmd.working_dir,
        Some(std::path::PathBuf::from("/my/project"))
    );
}

// ===========================================================================
// Gemini build_command() validation
// ===========================================================================

#[test]
fn gemini_command_has_model_flag() {
    let provider = GeminiProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert!(cmd.args.contains(&"-m".to_string()));
}

#[test]
fn gemini_command_has_prompt_flag() {
    let provider = GeminiProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert!(cmd.args.contains(&"-p".to_string()));
}

#[test]
fn gemini_command_has_raw_output() {
    let provider = GeminiProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert!(cmd.args.contains(&"--raw-output".to_string()));
}

#[test]
fn gemini_command_has_approval_mode_yolo() {
    let provider = GeminiProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert!(cmd.args.contains(&"--approval-mode".to_string()));
    assert!(cmd.args.contains(&"yolo".to_string()));
}

#[test]
fn gemini_command_working_dir_set() {
    let provider = GeminiProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "prompt")
        .unwrap();
    assert_eq!(cmd.working_dir, Some(std::path::PathBuf::from("/project")));
}

// ===========================================================================
// Generic build_command() validation
// ===========================================================================

#[test]
fn generic_base_args_before_prompt() {
    let provider = GenericCliProvider::new("test", "Test", "test-cli", test_markers())
        .with_arg("--flag1")
        .with_arg("--flag2");
    let cmd = provider
        .build_command(Path::new("/tmp"), "do work")
        .unwrap();
    let flag_idx = cmd.args.iter().position(|a| a == "--flag1").unwrap();
    let prompt_idx = cmd.args.iter().position(|a| a == "do work").unwrap();
    assert!(flag_idx < prompt_idx, "base args should come before prompt");
}

#[test]
fn generic_env_vars_set() {
    let provider = GenericCliProvider::new("test", "Test", "test-cli", test_markers())
        .with_env("MY_VAR", "my_value");
    let cmd = provider.build_command(Path::new("/tmp"), "task").unwrap();
    assert_eq!(cmd.env.get("MY_VAR"), Some(&"my_value".to_string()));
}

#[test]
fn generic_working_dir_set() {
    let provider = GenericCliProvider::new("test", "Test", "test-cli", test_markers());
    let cmd = provider.build_command(Path::new("/work"), "task").unwrap();
    assert_eq!(cmd.working_dir, Some(std::path::PathBuf::from("/work")));
}

// ===========================================================================
// Mock build_command() validation
// ===========================================================================

#[test]
fn mock_uses_echo() {
    let provider = MockProvider::new();
    let cmd = provider.build_command(Path::new("/tmp"), "hello").unwrap();
    assert_eq!(cmd.program, "echo");
}

#[test]
fn mock_prompt_as_arg() {
    let provider = MockProvider::new();
    let cmd = provider
        .build_command(Path::new("/tmp"), "hello world")
        .unwrap();
    assert!(cmd.args.contains(&"hello world".to_string()));
}

// ===========================================================================
// Tier 1 providers build_command() succeeds
// ===========================================================================

fn assert_tier1_cmd_valid(provider: &dyn AiProvider, expected_program: &str) {
    let cmd = provider
        .build_command(Path::new("/project"), "do task")
        .unwrap();
    assert_eq!(cmd.program, expected_program);
    assert!(
        cmd.args.contains(&"do task".to_string()),
        "{} should include prompt in args",
        provider.name()
    );
    assert_eq!(cmd.working_dir, Some(std::path::PathBuf::from("/project")));
    assert!(
        !cmd.program.is_empty(),
        "{} should have non-empty program",
        provider.name()
    );
}

#[test]
fn kiro_build_command_succeeds() {
    let provider = tier1::kiro_provider();
    assert_tier1_cmd_valid(&provider, "kiro");
}

#[test]
fn goose_build_command_succeeds() {
    let provider = tier1::goose_provider();
    assert_tier1_cmd_valid(&provider, "goose");
}

#[test]
fn deepseek_build_command_succeeds() {
    let provider = tier1::deepseek_provider();
    assert_tier1_cmd_valid(&provider, "deepseek");
}

#[test]
fn qwen_build_command_succeeds() {
    let provider = tier1::qwen_provider();
    assert_tier1_cmd_valid(&provider, "qwen");
}

#[test]
fn opencode_build_command_succeeds() {
    let provider = tier1::opencode_provider();
    assert_tier1_cmd_valid(&provider, "opencode");
}

#[test]
fn cody_build_command_succeeds() {
    let provider = tier1::cody_provider();
    assert_tier1_cmd_valid(&provider, "cody");
}

// ===========================================================================
// Cross-provider: all registered providers build_command succeeds
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
fn all_registered_providers_build_command_succeeds() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let result = provider.build_command(Path::new("/tmp"), "test prompt");
        assert!(
            result.is_ok(),
            "Provider '{}' build_command failed: {:?}",
            provider.name(),
            result.err()
        );
    }
}

#[test]
fn no_provider_uses_stdin_data() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let cmd = provider.build_command(Path::new("/tmp"), "prompt").unwrap();
        assert!(
            cmd.stdin_data.is_none(),
            "Provider '{}' unexpectedly sets stdin_data",
            provider.name()
        );
    }
}
