# Mahalaxmi

**Parallel AI coding agents with consensus — run Claude Code, Copilot, and Ollama side-by-side,
then let the consensus engine reconcile their work into a single unified diff.**

Mahalaxmi is an open-source Rust orchestration engine that spawns multiple AI coding agents
in isolated PTY sessions, coordinates them via a Manager-Worker architecture,
and uses a consensus protocol to detect conflicts and produce a clean, reviewable result.
No chaos. No merge hell. Just parallel speed with a human-in-the-loop safety net.

[![License: MIT](https://img.shields.io/badge/License-MIT-teal.svg)](LICENSE)
[![Build](https://github.com/mahalaxmi-ai/mahalaxmi/actions/workflows/ci.yml/badge.svg)](https://github.com/mahalaxmi-ai/mahalaxmi/actions)
[![Discord](https://img.shields.io/discord/bSkzhTPK?label=Discord&color=5865F2)](https://discord.gg/bSkzhTPK)

![Mahalaxmi Dashboard](docs/assets/demo.gif)
<!-- TODO: replace with actual demo GIF before launch -->

> **Keywords:** AI coding agents · multi-agent orchestration · LLM parallelism · PTY terminal automation ·
> consensus engine · Claude Code · GitHub Copilot · Ollama · multi-provider routing · Rust · agentic coding

## Quick Start

```bash
# 1. Install
cargo install mahalaxmi-cli

# 2. Configure your first provider
mahalaxmi provider add claude-code

# 3. Run your first orchestration cycle
mahalaxmi run
```

## How It Works

Mahalaxmi uses a **Manager-Worker consensus architecture**:

1. The **Manager** analyzes your codebase via AST-based repo maps,
   decomposes the task into a dependency-ordered DAG, and assigns subtasks to workers
2. **Workers** execute in parallel inside isolated PTY sessions —
   each worker drives a real AI CLI tool (Claude Code, Copilot, Codex, Ollama, or any custom CLI)
   over its actual terminal interface, with no SDK wrapping
3. The **Consensus Engine** collects all worker outputs, runs conflict detection
   across file-level and semantic boundaries, and produces a single unified diff
4. **You review and approve** — one clean patch, not five conflicting ones

This means you get genuine multi-provider parallelism (not just one model with retries),
real terminal I/O (not API calls that bypass your auth or tool config),
and a deterministic reconciliation step before anything touches your repo.

## Why PTY-Based Routing

Most multi-agent systems call LLM APIs directly. Mahalaxmi does not.

Instead, each worker spawns your actual AI CLI tool in a pseudo-terminal — the same way
you'd use it interactively. This has meaningful consequences:

- **No API key juggling** — workers authenticate exactly as you do (OAuth, keychain, enterprise SSO)
- **Full tool fidelity** — workers use each tool's native file editing, diff, and search capabilities
- **Provider isolation** — a bug in one provider's output can't corrupt another worker's context
- **Bring any CLI** — if it runs in a terminal and produces diffs, you can plug it in

## Supported Providers

| Provider | Status |
|----------|--------|
| Claude Code (Anthropic) | ✅ Built-in |
| GitHub Copilot | ✅ Built-in |
| OpenAI Codex | ✅ Built-in |
| Ollama (local models) | ✅ Built-in |
| Custom CLI | ✅ Built-in — bring any AI CLI tool |
| Community plugins | [Contribute one →](CONTRIBUTING.md) |

## Repository Structure

| Crate | Description |
|-------|-------------|
| [`mahalaxmi-core`](crates/mahalaxmi-core) | Domain types, config, logging |
| [`mahalaxmi-pty`](crates/mahalaxmi-pty) | PTY spawning and terminal I/O |
| [`mahalaxmi-orchestration`](crates/mahalaxmi-orchestration) | Consensus engine and DAG types |
| [`mahalaxmi-detection`](crates/mahalaxmi-detection) | State detection rule engine |
| [`mahalaxmi-providers`](crates/mahalaxmi-providers) | Provider trait and reference implementations |
| [`mahalaxmi-indexing`](crates/mahalaxmi-indexing) | AST parsing and repo maps |
| [`mahalaxmi-cli`](crates/mahalaxmi-cli) | Command-line interface |

## Build a Provider Plugin

The highest-value contribution you can make is adding a new AI provider.
If your favorite AI CLI tool isn't in the table above, you can add it in one Rust file.

Implement the `AiProvider` trait from `mahalaxmi-providers`.
This skeleton covers every required method — paste it, fill in `build_command`
and `validate_credentials`, and it will compile:

```rust
// In Cargo.toml:
// mahalaxmi-providers = { git = "https://github.com/mahalaxmi-ai/mahalaxmi" }

use async_trait::async_trait;
use mahalaxmi_providers::{
    AiProvider, CredentialSpec, MahalaxmiConfig, MahalaxmiResult,
    OutputMarkers, ProviderCapabilities, ProviderMetadata, ProviderId,
    I18nService, ProcessCommand,
};
use std::path::Path;

#[derive(Clone)]
pub struct MyToolProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    metadata: ProviderMetadata,
}

impl MyToolProvider {
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("mytool"),
            capabilities: ProviderCapabilities::default(),
            markers: OutputMarkers::new(
                r"DONE",                      // pattern that signals completion
                r"(?i)(error|fatal|failed)",  // pattern that signals an error
                r">\s*$",                     // pattern that signals waiting for input
            ).expect("markers are valid regex"),
            metadata: ProviderMetadata::new("pip install mytool"), // install hint
        }
    }
}

#[async_trait]
impl AiProvider for MyToolProvider {
    fn name(&self) -> &str { "My Tool" }
    fn id(&self) -> &ProviderId { &self.id }
    fn cli_binary(&self) -> &str { "mytool" }
    fn metadata(&self) -> &ProviderMetadata { &self.metadata }
    fn capabilities(&self) -> &ProviderCapabilities { &self.capabilities }
    fn output_markers(&self) -> &OutputMarkers { &self.markers }
    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        vec![] // return CredentialSpec entries for any API keys / env vars needed
    }

    fn build_command(&self, dir: &Path, prompt: &str) -> MahalaxmiResult<ProcessCommand> {
        // Build the shell command that launches your CLI with the prompt.
        // The PTY engine will spawn this command and manage its terminal I/O.
        Ok(ProcessCommand::new(self.cli_binary())
            .arg("--some-flag")
            .arg(prompt)
            .working_dir(dir))
    }

    async fn validate_credentials(&self, _i18n: &I18nService) -> MahalaxmiResult<()> {
        // Check env vars, files, or keyrings. No network calls.
        // Return Err(MahalaxmiError::ProviderNotConfigured { .. }) if credentials are missing.
        Ok(())
    }

    fn configure(&self, _config: &MahalaxmiConfig) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }

    fn clone_box(&self) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }
}
```

The trait has optional overrides for streaming markers, token extraction, and model
selection — leave them at their defaults until you need them.
See [`crates/mahalaxmi-providers/src/ollama.rs`](crates/mahalaxmi-providers/src/ollama.rs)
for a complete real-world implementation.

**To submit a provider:**
1. Open a [Provider Plugin issue](.github/ISSUE_TEMPLATE/provider_plugin.md) first to claim the slot
2. Fork, implement, add tests, open a PR against `main`
3. Accept the [CLA](CLA.md) via PR comment (`I have read and agree to the CLA.`)

Other contributions (detection rules, language parsers, bug fixes, docs) are also welcome.
See [CONTRIBUTING.md](CONTRIBUTING.md) for the full scope.

## Community

- 💬 [Discord](https://discord.gg/bSkzhTPK) — questions, showcase, provider plugin help
- 🐦 [Twitter](https://x.com/MahalaxmiDev) — releases and updates
- 🐛 [Issues](https://github.com/mahalaxmi-ai/mahalaxmi/issues) — bug reports
- 💡 [Discussions](https://github.com/mahalaxmi-ai/mahalaxmi/discussions) — feature ideas

## License

The foundation crates in this repository are MIT licensed.
See [LICENSE](LICENSE) for details.

The full Mahalaxmi product — including the orchestration driver,
GraphRAG engine, cloud service, and desktop app — is proprietary
software available at [mahalaxmi.ai](https://mahalaxmi.ai).

Mahalaxmi™ is a trademark of ThriveTech Services LLC.
