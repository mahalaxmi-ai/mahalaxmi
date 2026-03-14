# mahalaxmi-core

Shared domain types, error handling, configuration, internationalization, and logging infrastructure for the Mahalaxmi AI Orchestration platform.

## Overview

`mahalaxmi-core` provides the foundation that every other Mahalaxmi crate builds on. It defines the canonical domain types (`WorkerId`, `TaskId`, `LicenseTier`, `ProcessCommand`), the unified error type (`MahalaxmiError`), and the configuration system that reads `~/.mahalaxmi/config.toml` and exposes typed access to every setting.

Internationalization is a first-class concern. All user-visible strings are loaded from Fluent (`.ftl`) locale files bundled with the crate. Ten locales ship out of the box: English (US), German, Spanish, French, Hindi, Arabic, Japanese, Korean, Brazilian Portuguese, and Simplified Chinese. Error messages constructed from `MahalaxmiError` are always localized through `I18nService`.

Most developers using Mahalaxmi will not depend on `mahalaxmi-core` directly — they consume it re-exported through the higher-level crates (`mahalaxmi-providers`, `mahalaxmi-orchestration`, etc.). Crate authors building new Mahalaxmi extensions should depend on it directly.

## Key Types

| Type | Kind | Description |
|------|------|-------------|
| `MahalaxmiConfig` | Struct | Runtime configuration (decrypted, typed); loaded from `~/.mahalaxmi/config.toml` |
| `MahalaxmiError` | Enum | Unified error type; always carries an i18n-localized message |
| `MahalaxmiResult<T>` | Type alias | `Result<T, MahalaxmiError>` |
| `I18nService` | Struct | Localized string lookup; wraps `rust-i18n` with typed message keys |
| `SupportedLocale` | Enum | The 10 supported UI locales |
| `WorkerId` | Struct | Newtype wrapping `u32`; identifies a worker agent slot |
| `TaskId` | Struct | Newtype wrapping `String`; identifies a task within an execution plan |
| `ProcessCommand` | Struct | Specification for launching a subprocess (program, args, env, working dir) |
| `TerminalConfig` | Struct | PTY terminal dimensions and settings |
| `TerminalId` | Struct | Unique handle for a managed PTY terminal |
| `LicenseTier` | Enum | `Trial`, `Basic`, `Pro`, `AllAccess`, `Enterprise` |
| `Developer` | Struct | Represents a team member operating the platform |
| `EncryptedString` | Struct | AES-GCM encrypted string for storing credentials at rest |

## Key Functions / Methods

| Function | Description |
|----------|-------------|
| `config::loader::load_config(path, i18n)` | Load and validate config from a TOML file; falls back to defaults if file absent |
| `config::loader::default_config_path()` | Returns `~/.mahalaxmi/config.toml` |
| `I18nService::new(locale)` | Create a localized string service for the given locale |
| `I18nService::translate(key, args)` | Look up a localized message by key with interpolated arguments |
| `MahalaxmiError::orchestration(i18n, key, args)` | Construct a localized orchestration error |
| `MahalaxmiError::pty(i18n, key, args)` | Construct a localized PTY error |
| `derive_key_from_passphrase(passphrase, salt)` | Derive an AES-256 key from a passphrase via PBKDF2 |
| `logging::init_tracing(config)` | Initialize the global `tracing` subscriber |

## Feature Flags

No feature flags.

## Dependencies

| Dependency | Why |
|-----------|-----|
| `serde` + `serde_json` + `toml` | Config serialization/deserialization |
| `thiserror` | Ergonomic error type derivation |
| `tracing` | Structured logging primitives |
| `rust-i18n` | Fluent-based internationalization |
| `uuid` | Unique IDs for sessions and events |
| `chrono` | Timestamps throughout the domain model |
| `aes-gcm` | AES-256-GCM encryption for `EncryptedString` |
| `hmac` + `sha2` + `base64` | HMAC signing used by the license subsystem |

## Stability

**Unstable** — API may change in minor versions during the pre-1.0 period.

## License

MIT — Copyright 2026 ThriveTech Services LLC
