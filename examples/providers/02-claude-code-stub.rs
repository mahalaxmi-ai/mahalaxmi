// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Claude Code provider walkthrough
//
// This example walks through the ClaudeCodeProvider — the reference
// implementation of AiProvider for Anthropic's Claude Code CLI tool.
// It demonstrates:
//   1. How authentication is handled (subscription login vs API key)
//   2. How capabilities are declared
//   3. How build_command() constructs the launch command
//   4. How output markers enable completion and error detection
//   5. How the provider integrates with the credential chain
//
// This is intended as a reading guide. The actual implementation is in
// crates/mahalaxmi-providers/src/claude.rs.

use mahalaxmi_core::{
    config::MahalaxmiConfig,
    i18n::{locale::SupportedLocale, I18nService},
};
use mahalaxmi_providers::{
    ClaudeCodeProvider,
    credentials::AuthMethod,
    traits::AiProvider,
};

#[tokio::main]
async fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = MahalaxmiConfig::default();

    // Build the provider from config. ClaudeCodeProvider reads claude.* settings:
    //   config.claude.selected_model — e.g. "claude-sonnet-4-6"
    //   config.claude.max_tokens     — context window override
    let provider = ClaudeCodeProvider::from_mahalaxmi_config(&config);

    // --- Authentication ---
    // Claude Code supports two auth modes:
    //
    // 1. Subscription login (recommended for desktop use):
    //    The user runs `claude auth login` once. Claude Code stores a session
    //    token in its own config directory. No API key is needed.
    //
    // 2. API key (ANTHROPIC_API_KEY env var):
    //    Set ANTHROPIC_API_KEY before starting Mahalaxmi. ClaudeCodeProvider
    //    injects this into the PTY environment via build_command().
    //
    // credential_requirements() lists both modes so the UI can show setup hints.
    let creds = provider.credential_requirements();
    println!("Credential requirements ({}):", creds.len());
    for spec in &creds {
        let method = match spec.method {
            AuthMethod::ApiKey => "API key",
            AuthMethod::EnvironmentVariable => "env var",
            AuthMethod::SystemKeyring => "keyring",
            _ => "other",
        };
        println!(
            "  [{method}] {:?}  required={}",
            spec.env_var_name,
            spec.required,
        );
    }

    // validate_credentials() checks LOCAL availability only (no network call).
    // It checks: ANTHROPIC_API_KEY env var, keyring, and claude session state.
    match provider.validate_credentials(&i18n).await {
        Ok(()) => println!("\nCredentials: valid (claude CLI or ANTHROPIC_API_KEY found)"),
        Err(e) => println!("\nCredentials: not configured ({e})"),
    }

    // --- Capabilities ---
    // The capabilities struct drives TaskRouter decisions — which provider
    // gets assigned which task type.
    let caps = provider.capabilities();
    println!("\nCapabilities:");
    println!("  max_context_tokens: {} tokens", caps.max_context_tokens);
    println!("  cost_tier:          {:?}", caps.cost_tier);
    println!("  streaming:          {}", caps.supports_streaming);
    println!("  local_only:         {}", caps.supports_local_only);
    println!("  tool_use:           {}", caps.supports_tool_use);

    // --- Command construction ---
    // build_command() produces the ProcessCommand passed to PtySpawner.
    // It adds: --dangerously-skip-permissions (non-interactive mode),
    //          -p <prompt> (the task instruction),
    //          --model <model> if a model is configured,
    //          ANTHROPIC_API_KEY in env if set.
    let cmd = provider.build_command(
        std::path::Path::new("/tmp/my-project"),
        "Refactor auth.rs to use the new token refresh flow.",
    );
    match cmd {
        Ok(cmd) => {
            println!("\nbuild_command output:");
            println!("  program: {}", cmd.program);
            println!("  args:    {:?}", cmd.args);
            let has_key = cmd.env.contains_key("ANTHROPIC_API_KEY");
            println!("  ANTHROPIC_API_KEY in env: {has_key}");
        }
        Err(e) => println!("\nbuild_command error: {e}"),
    }

    // --- Output markers ---
    // These regex patterns are matched against every line of PTY output.
    // The detection engine calls RuleMatcher with these plus the built-in
    // claude_code() rule set (see examples/detection/).
    let markers = provider.output_markers();
    println!("\nOutput markers:");
    println!("  completion: {}", markers.completion_marker.as_str());
    println!("  error:      {}", markers.error_marker.as_str());
    println!("  prompt:     {}", markers.prompt_marker.as_str());

    // --- Stream completion marker ---
    // When Claude Code emits streaming JSON, the driver can skip the idle
    // timeout by watching for this marker in the raw output stream.
    if let Some(marker) = provider.stream_complete_marker() {
        println!("  stream_complete: {}", marker);
    }
}
