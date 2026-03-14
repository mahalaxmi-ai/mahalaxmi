# mahalaxmi-providers

AI provider abstraction layer: the `AiProvider` trait, built-in implementations for major AI CLI tools, credential management, routing, and resilience.

## Overview

`mahalaxmi-providers` defines the `AiProvider` trait — the single interface through which Mahalaxmi's orchestration engine interacts with every AI coding tool. Whether the underlying tool is Claude Code, OpenAI Foundry, AWS Bedrock, Google Gemini, or any custom CLI, the orchestration engine only ever calls `build_command()`, `validate_credentials()`, and `capabilities()`. New providers are added by implementing the trait; no engine changes are needed.

The crate ships built-in providers for Claude Code, OpenAI (ChatGPT/Foundry), AWS Bedrock, Google Gemini, GitHub Copilot, Grok, Ollama, and Aider, plus a `CustomCliProvider` that wraps any arbitrary AI CLI tool with a single configuration struct. Tier-1 community providers (Kiro, Goose, DeepSeek, Qwen Coder, OpenCode, Cody) are included in the `tier1` module. The `ProviderRegistry` loads all built-in providers and discovers custom ones from config.

This is the **primary community contribution lane**. If you want to add support for a new AI tool, implement `AiProvider` and submit a pull request. The `examples/providers/` directory contains a complete worked example (`EchoProvider`) and walkthroughs of the Claude Code and CustomCLI patterns.

## Key Types

| Type | Kind | Description |
|------|------|-------------|
| `AiProvider` | Trait | Core abstraction: every AI CLI tool implements this |
| `ProviderCapabilities` | Struct | What a provider can do: task types, cost tier, context window, streaming |
| `ProviderMetadata` | Struct | Install hints, test commands, supported auth modes, model list |
| `CredentialSpec` | Struct | Describes one credential the provider needs (env var name, auth method) |
| `AuthMethod` | Enum | `ApiKey`, `OAuth`, `AwsIam`, `EnvironmentVariable`, `SystemKeyring`, `ServiceAccount` |
| `OutputMarkers` | Struct | Regex patterns for completion, error, and prompt detection |
| `ProviderRegistry` | Struct | Discovers and holds all registered providers |
| `TaskRouter` | Struct | Routes tasks to the best provider using `RoutingStrategy` |
| `RoutingStrategy` | Enum | `QualityFirst`, `CostFirst`, `SpeedFirst`, `Balanced` |
| `CircuitBreaker` | Struct | Per-provider circuit breaker for failure isolation |
| `ChainedCredentialStore` | Struct | Env → keyring → file credential lookup chain |
| `TokenUsage` | Struct | Input/output token counts; exact or estimated from bytes |
| `TaskType` | Enum | `CodeGeneration`, `CodeReview`, `Debugging`, `Refactoring`, `Testing`, `Documentation`, `Planning` |
| `MockProvider` | Struct | Test double implementing `AiProvider`; configurable responses |

## Key Functions / Methods

| Function | Description |
|----------|-------------|
| `ProviderRegistry::new(config, i18n)` | Build registry from config; registers all built-in providers |
| `ProviderRegistry::get(id)` | Look up a provider by its `ProviderId` |
| `TaskRouter::route(task_type, constraints)` | Select best provider for a task type given constraints |
| `OutputMarkers::new(completion, error, prompt)` | Construct markers from regex strings |
| `resolve_provider_credentials(provider, store, i18n)` | Resolve credentials from the chained store |
| `probe_keyring()` | Check if the OS keyring is available at runtime |
| `credential_key(provider, var)` | Build a namespaced keyring key (e.g., `"claude-code/ANTHROPIC_API_KEY"`) |
| `find_binary(name)` | Search PATH for a CLI binary; returns its resolved path |
| `CircuitBreaker::call(f)` | Execute `f`; open circuit if consecutive failures exceed threshold |
| `TokenUsage::estimate_from_bytes(bytes)` | Heuristic token count from raw byte length |

## Feature Flags

No feature flags.

## Dependencies

| Dependency | Why |
|-----------|-----|
| `async-trait` | Object-safe async methods on `AiProvider` |
| `tokio` | Async runtime for credential validation |
| `reqwest` | HTTP calls for remote credential validation |
| `keyring` | OS keyring access (macOS Keychain, Windows Credential Manager, libsecret) |
| `regex` | Pattern matching for `OutputMarkers` |
| `serde` + `serde_json` | Provider config and capability serialization |
| `mahalaxmi-core` | Shared types, config, errors, i18n |

## Stability

**Unstable** — API may change in minor versions during the pre-1.0 period.

## License

MIT — Copyright 2026 ThriveTech Services LLC
