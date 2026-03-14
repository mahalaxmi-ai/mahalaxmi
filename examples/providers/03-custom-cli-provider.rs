// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Custom CLI provider pattern
//
// Shows how to wrap any arbitrary AI CLI tool using CustomCliProvider.
// This is the fastest path for adding provider support when you don't
// need custom logic — configure the binary path, arguments, env vars,
// and you're done.
//
// Use this pattern for:
//   - Proprietary internal AI tools
//   - Self-hosted open-source models wrapped in a custom CLI
//   - Rapid prototyping before writing a full AiProvider implementation
//
// For production-grade providers with custom streaming logic, token
// counting, or complex auth flows, implement AiProvider directly
// (see examples/providers/01-implement-provider.rs).

use mahalaxmi_core::{
    config::{CustomCliConfig, MahalaxmiConfig},
    i18n::{locale::SupportedLocale, I18nService},
};
use mahalaxmi_providers::{
    CustomCliProvider,
    traits::AiProvider,
};

#[tokio::main]
async fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // CustomCliConfig is the config-file counterpart to building a provider
    // in code. The user sets these fields in ~/.mahalaxmi/config.toml under
    // the [custom_cli] section.
    //
    // Here we build one in code to show the fields:
    let custom_config = CustomCliConfig {
        // The CLI binary path. When None, CustomCliProvider falls back to
        // a default binary name. Must be on PATH or an absolute path.
        binary_path: Some("my-ai-cli".to_string()),
        // Arguments passed before the prompt. The prompt is appended last.
        //   Final command: my-ai-cli --no-color --task <PROMPT>
        args: Some(vec!["--no-color".to_string(), "--task".to_string()]),
        // Optional env vars to inject (name → value). If None, no extra
        // env vars are set beyond what the system provides.
        env_vars: None,
        // Optional API key for the provider.
        api_key: None,
        // Selected model ID (empty = auto-select from models list).
        selected_model: None,
        // Available models (empty = accept any model name).
        models: vec![],
        // Auto-selection configuration.
        auto_select: Default::default(),
    };

    // Build a MahalaxmiConfig that includes our custom CLI config.
    let config = MahalaxmiConfig {
        custom_cli: custom_config,
        ..Default::default()
    };

    // Construct the provider. Internally this builds OutputMarkers from the
    // regex patterns and ProviderCapabilities with sensible defaults.
    let provider = CustomCliProvider::from_mahalaxmi_config(&config);

    // --- Inspect the provider ---
    println!("Provider name: {}", provider.name());
    println!("CLI binary:    {}", provider.cli_binary());

    let caps = provider.capabilities();
    println!("Capabilities:");
    println!("  cost tier:   {:?}", caps.cost_tier);
    println!("  streaming:   {}", caps.supports_streaming);
    println!("  local only:  {}", caps.supports_local_only);

    // --- Validate credentials ---
    // CustomCliProvider checks that any configured API key env var is set.
    match provider.validate_credentials(&i18n).await {
        Ok(()) => println!("\nCredentials: ok"),
        Err(e) => println!("\nCredentials: not configured — {e}"),
    }

    // --- Build the command ---
    // Args are: [binary] [configured_args...] [prompt]
    let cmd = provider.build_command(
        std::path::Path::new("/tmp/project"),
        "Add OpenTelemetry spans to the HTTP handler.",
    );
    match cmd {
        Ok(cmd) => {
            println!("\nbuild_command:");
            println!("  program: {}", cmd.program);
            println!("  args:    {:?}", cmd.args);
            println!("  env keys: {:?}", cmd.env.keys().collect::<Vec<_>>());
        }
        Err(e) => println!("\nbuild_command error: {e}"),
    }

    // --- Output markers ---
    let markers = provider.output_markers();
    println!("\nOutput markers:");
    println!("  completion: {}", markers.completion_marker.as_str());
    println!("  error:      {}", markers.error_marker.as_str());
    println!("  prompt:     {}", markers.prompt_marker.as_str());

    println!("\nCustomCliProvider wired successfully.");
    println!("To use in production: set [custom_cli] in ~/.mahalaxmi/config.toml");
}
