# Testing Guide

The public Mahalaxmi OSS crates ship with a comprehensive test suite covering
every public API surface used by the bundled examples. Tests are designed as
API regression guards — any change that breaks a stable interface will fail
a test before it can reach users.

---

## Quick Reference

```bash
# All tests (unit + integration)
cargo test --workspace

# Single crate
cargo test -p mahalaxmi-orchestration

# Single test by name
cargo test -p mahalaxmi-providers test_claude_build_command

# Integration tests only
cargo test -p mahalaxmi-integration-tests

# All examples compile
cargo build --examples

# Clippy (zero warnings, mirrors CI)
cargo clippy --workspace --examples -- -D warnings
```

---

## Test Layout

```
mahalaxmi-oss/
├── crates/
│   ├── mahalaxmi-core/tests/example_api_tests.rs          # 21 tests
│   ├── mahalaxmi-detection/tests/example_api_tests.rs     # 20 tests
│   ├── mahalaxmi-indexing/tests/example_api_tests.rs      # 34 tests
│   ├── mahalaxmi-orchestration/tests/example_api_tests.rs # 40 tests
│   ├── mahalaxmi-providers/tests/example_api_tests.rs     # 34 tests
│   └── mahalaxmi-pty/tests/session_api_tests.rs           # 22 tests
├── integration-tests/
│   └── tests/e2e_scenarios.rs                             # 20 tests
└── examples/                                              # 14 runnable examples
    ├── core/
    ├── detection/
    ├── indexing/
    ├── orchestration/
    ├── providers/
    └── pty/
```

All test files use only public APIs — `pub` items accessible to downstream
crates. This means the tests break when the public interface changes, not when
internal implementation details change.

---

## Crate Test Inventory

### mahalaxmi-core — 21 tests
**File:** `crates/mahalaxmi-core/tests/example_api_tests.rs`

Covers the foundational types and services used by every other crate.

| Test | What it verifies |
|------|-----------------|
| `default_config_path_returns_path_buf` | `MahalaxmiConfig::default_path()` returns a valid `PathBuf` |
| `load_config_with_nonexistent_path_returns_defaults` | Config load from missing file returns `Default` without panic |
| `config_log_level_default` | `config.general.log_level` defaults to `Info` |
| `config_locale_default` | `config.general.locale` defaults to `EnUs` |
| `config_max_concurrent_workers` | `config.orchestration.max_concurrent_workers` field accessible |
| `config_providers_section` | `config.providers.default_provider` field accessible |
| `config_indexing_section` | `config.indexing.max_file_size_bytes` and `excluded_dirs` accessible |
| `config_ui_section` | `config.ui.terminal_font_size` field accessible |
| `supported_locales_all_load` | All 10 `SupportedLocale` variants construct `I18nService` without panic |
| `i18n_service_lookup` | `i18n.t(key)` returns a non-empty string for a known key |
| `process_command_fields` | `ProcessCommand` has `program`, `args`, `env`, `working_dir`, `stdin_data` |
| `terminal_id_new` | `TerminalId::new()` produces a unique ID each call |
| `worker_id_new` | `WorkerId::new()` produces a unique ID each call |
| `task_id_new` | `TaskId::new(key)` round-trips through `as_str()` |
| `terminal_config_default` | `TerminalConfig::default()` has `rows=24`, `cols=80` |
| `terminal_config_custom` | Custom rows/cols survive struct construction |
| `process_command_serde` | `ProcessCommand` serializes and deserializes via serde_json |
| `mahalaxmi_config_serde` | `MahalaxmiConfig::default()` serializes without error |
| `logging_init_does_not_panic` | `logging::init_logging(&config.general, &i18n)` returns `Ok` |
| `logging_init_idempotent` | Calling `init_logging` twice does not panic |
| `supported_locale_en_us` | `SupportedLocale::EnUs` is the default |

### mahalaxmi-detection — 20 tests
**File:** `crates/mahalaxmi-detection/tests/example_api_tests.rs`

Covers the rule matching engine: built-in rule sets, custom rules, evaluation
semantics, priority, and cooldown.

| Test group | Tests | Key invariants |
|-----------|------:|---------------|
| Built-in rules | 3 | `BuiltinRuleSets` non-empty; returns known rule set names |
| `RuleMatcher` construction | 2 | `new(rules, &i18n)` succeeds; rejects empty rule list gracefully |
| Evaluation basics | 4 | No match → `None`; match → `Some`; result has `matched_rule_name`, `severity`, `message` |
| Custom rule types | 5 | Contains rule, regex rule, provider-scoped rule, priority ordering |
| Cooldown | 2 | Same rule does not fire twice within cooldown window; `reset_cooldowns` clears state |
| Provider filter | 2 | Rule with `provider_id = "claude"` does not fire for `provider_id = "openai"` |
| Simulation | 2 | Multi-line worker output stream — correct rules fire, correct count |

### mahalaxmi-indexing — 34 tests
**File:** `crates/mahalaxmi-indexing/tests/example_api_tests.rs`

Covers language detection, Tree-sitter symbol extraction, dependency graphs,
and repo map generation.

| Test group | Tests | Key invariants |
|-----------|------:|---------------|
| `SupportedLanguage::from_extension` | 10 | `.rs`, `.ts`, `.py`, `.go`, `.js`, `.tsx`, `.java`, `.cpp` detected; unknown → `None`; leading dot required |
| Language methods | 3 | `as_str()` and `extensions()` are consistent |
| `LanguageRegistry` | 2 | `with_defaults()` includes all 8 languages |
| `ExtractorFactory` | 4 | Creates extractor for each language; `extract_symbols` returns `Vec<Symbol>` |
| Symbol fields | 4 | `kind`, `name`, `visibility`, `line_start` present; `SymbolKind::Function` and `Struct` recognized |
| `FileDependencyGraph` | 6 | `add_dependency`, `dependencies_of`, `dependents_of`, `bfs_distance` |
| BFS semantics | 3 | Distance 0 = same file; multi-hop distance correct; missing file → `None` |
| `RepoMapConfig` | 2 | Default token budget and `GroupBy` accessible |

### mahalaxmi-orchestration — 40 tests
**File:** `crates/mahalaxmi-orchestration/tests/example_api_tests.rs`

Covers the DAG task model, worker queue lifecycle, phase builder, and cycle
plan construction.

| Test group | Tests | Key invariants |
|-----------|------:|---------------|
| `TaskId` / `WorkerId` | 4 | `new`, `as_str`, equality, unique IDs |
| `WorkerTask` | 5 | Field access; `depends_on` populated; serde round-trip |
| `ExecutionPhase` | 3 | `phase_number` field; `tasks` vec |
| `ExecutionPlan` | 6 | `new` / `from_phases`; `all_workers()`; `validate()` rejects duplicate worker IDs |
| `validate_dag` | 4 | Empty plan valid; acyclic plan valid; cyclic plan returns `Err` |
| `detect_cycles` | 3 | Self-loop; two-node cycle; three-node acyclic |
| `build_phases` | 4 | Dependencies respected; independent tasks in same phase; phase count matches |
| `WorkerQueue` lifecycle | 11 | Ready → activate → complete; `ready_worker_ids`; `activate_worker`; `complete_worker`; statistics |

### mahalaxmi-providers — 34 tests
**File:** `crates/mahalaxmi-providers/tests/example_api_tests.rs`

Covers the `AiProvider` trait via `ClaudeCodeProvider` and `CustomCliProvider`,
capability fields, auth modes, and command building.

| Test group | Tests | Key invariants |
|-----------|------:|---------------|
| `ClaudeCodeProvider` construction | 3 | `new()`, `from_mahalaxmi_config(&config)`, both succeed |
| Provider identity | 3 | `name()`, `id()`, `cli_binary()` return non-empty strings |
| Capabilities | 7 | `max_context_tokens`, `supports_streaming`, `supports_local_only`, `cost_tier`, `supports_tool_use`, `supports_structured_output` accessible |
| Command building | 3 | `build_command(path, prompt)` → `ProcessCommand` with non-empty `program` and `args` |
| Output markers | 3 | `completion_marker`, `error_marker`, `prompt_marker` are valid regex strings |
| `CustomCliProvider` | 5 | `from_mahalaxmi_config`, `name`, `cli_binary`, `build_command`, `output_markers` |
| `ProviderMetadata` builder | 3 | `new(hint)`, `with_auth_mode(mode)`, `auth_mode()` round-trip |
| `AuthMode` variants | 4 | `None`, `ApiKey { env_var }`, `CliLogin { login_command, check_command }`, `ServiceAccount { env_var, description }` |
| `ProviderCapabilities::default()` | 3 | Sensible defaults — `supports_streaming = true`, `cost_tier` not `Free` |

### mahalaxmi-pty — 22 tests
**File:** `crates/mahalaxmi-pty/tests/session_api_tests.rs`

Covers the output buffer, VT terminal cleaner, terminal events, and session
manager subscription pattern.

| Test group | Tests | Key invariants |
|-----------|------:|---------------|
| `OutputBuffer` | 10 | `new`, `push_line`, `push_text`, `flush`, `drain`, `len`, `is_empty`, `tail(n)`, `search(pattern)` |
| Raw replay cap | 1 | `DEFAULT_RAW_REPLAY_CAPACITY_BYTES == 512 * 1024` |
| `VtCleaner` | 6 | Plain text pass-through; ANSI colour strip; bold strip; newline preservation; empty input; escape-only input |
| `TerminalEvent` variants | 4 | `TextOutput`, `OutputReceived`, `ProcessExited`, `StateChanged` — all constructible |
| `TerminalSessionManager` | 1 | `new(&config.orchestration, i18n)` + `subscribe()` succeed without panic |

---

## Integration Tests — 20 tests
**File:** `integration-tests/tests/e2e_scenarios.rs`

Cross-crate scenarios that mirror real orchestration workflows. Each scenario
constructs a full pipeline from config through provider selection, detection,
and orchestration, verifying that the crates compose correctly.

| Scenario group | Tests | Pipeline exercised |
|---------------|------:|-------------------|
| Config → Provider | 3 | `MahalaxmiConfig` → `ClaudeCodeProvider::from_mahalaxmi_config` → `build_command` |
| Detection pipeline | 3 | Built-in rules → `RuleMatcher` → evaluate output lines → collect matched rules |
| Orchestration pipeline | 6 | `WorkerTask` → `build_phases` → `ExecutionPlan::from_phases` → `WorkerQueue` lifecycle |
| Indexing pipeline | 4 | `from_extension` → `ExtractorFactory::create` → `extract_symbols` → `bfs_distance` |
| Provider capability routing | 4 | Capability field comparison; `CostTier` ordering; `max_context_tokens` for routing decisions |

---

## Examples

14 runnable examples document the public API in executable form. They compile
in CI and serve as integration smoke tests.

| Example | File | What it demonstrates |
|---------|------|----------------------|
| `core-01-config-loading` | `examples/core/01-config-loading.rs` | Load config, inspect all sections |
| `core-02-logging-setup` | `examples/core/02-logging-setup.rs` | `init_logging`, structured tracing spans |
| `detection-01-basic-detection` | `examples/detection/01-basic-detection.rs` | Built-in rules, `RuleMatcher::evaluate` |
| `detection-02-custom-rules` | `examples/detection/02-custom-rules.rs` | Contains/regex/provider-scoped rules |
| `indexing-01-parse-repository` | `examples/indexing/01-parse-repository.rs` | `CodebaseIndex::build`, repo map |
| `indexing-02-symbol-extraction` | `examples/indexing/02-symbol-extraction.rs` | `ExtractorFactory`, `FileDependencyGraph` |
| `orchestration-01-dag-types` | `examples/orchestration/01-dag-types.rs` | Task/Phase/Plan construction, DAG validation |
| `orchestration-02-worker-queue` | `examples/orchestration/02-worker-queue.rs` | `WorkerQueue` lifecycle |
| `providers-01-implement-provider` | `examples/providers/01-implement-provider.rs` | Custom `AiProvider` implementation |
| `providers-02-claude-code-stub` | `examples/providers/02-claude-code-stub.rs` | `ClaudeCodeProvider` inspection |
| `providers-03-custom-cli-provider` | `examples/providers/03-custom-cli-provider.rs` | `CustomCliProvider` configuration |
| `pty-01-spawn-process` | `examples/pty/01-spawn-process.rs` | `PtySpawner::spawn`, `output_snapshot`, `VtCleaner` |
| `pty-02-stream-output` | `examples/pty/02-stream-output.rs` | `TerminalSessionManager`, event subscription |
| `cli-01-connect-to-instance` | `examples/cli/01-connect-to-instance.rs` | `ServiceClient` REST + SSE connection |

---

## Writing New Tests

```rust
// crates/mahalaxmi-core/tests/example_api_tests.rs  (add to existing file)
use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};

#[test]
fn my_new_api_works() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    // test only pub items — this file has access to nothing private
}
```

Rules:
- Only test public API. If you need `pub(crate)` internals, use an inline
  `#[cfg(test)]` module inside the crate instead.
- `I18nService` is **not Clone** — create a fresh instance per owner.
- Use `expect("descriptive message")` not `unwrap()`.
- If a new public function is added, add at least one positive and one
  negative test here before merging.

---

## Recommended CI Configuration

```yaml
# .github/workflows/ci.yml
on: [push, pull_request]
jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy --workspace --examples -- -D warnings
      - run: cargo build --workspace --examples
      - run: cargo test --workspace
```

---

## Test Count Summary

| Scope | Tests |
|-------|------:|
| `mahalaxmi-core` | 21 |
| `mahalaxmi-detection` | 20 |
| `mahalaxmi-indexing` | 34 |
| `mahalaxmi-orchestration` | 40 |
| `mahalaxmi-providers` | 34 |
| `mahalaxmi-pty` | 22 |
| Integration (E2E) | 20 |
| **Total** | **191** |

Last updated: 2026-03-13.
