# Changelog

All notable changes to the Mahalaxmi public foundation crates are documented here.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [Unreleased]

### Added
- Self-hosted Hetzner CI runner with full test suite on every PR
- Branch protection: PRs require CLA + passing CI before merge
- `dependabot.yml` for automated dependency security updates

---

## [100.0.0] — 2026-03-18

### Added
- Initial public release of the Mahalaxmi OSS foundation layer
- `mahalaxmi-core` — domain types, config, i18n, logging
- `mahalaxmi-pty` — PTY spawning and terminal I/O
- `mahalaxmi-orchestration` — consensus engine, DAG planner, state machine
- `mahalaxmi-detection` — state detection rule engine
- `mahalaxmi-providers` — `AiProvider` trait and reference implementations
  (Claude Code, GitHub Copilot, OpenAI Codex, Ollama, Gemini)
- `mahalaxmi-indexing` — AST parsing and repo maps via Tree-sitter
- `mahalaxmi-memory` — agent memory store with SQLite persistence
- `mahalaxmi-cli` — command-line interface (`mahalaxmi` binary)
- 14 runnable examples across all crates
- Comprehensive test suite (unit + integration, 200+ tests)
- VS Code extension scaffold
- CLA bot (contributor-assistant) with PR-comment acceptance flow
- SOPS + age encrypted secrets management

### Notes
- Version `100.0.0` reflects internal versioning alignment with the private
  product monorepo. Public crate versioning will follow SemVer from this
  baseline for all breaking changes.

---

[Unreleased]: https://github.com/mahalaxmi-ai/mahalaxmi/compare/v100.0.0...HEAD
[100.0.0]: https://github.com/mahalaxmi-ai/mahalaxmi/releases/tag/v100.0.0
