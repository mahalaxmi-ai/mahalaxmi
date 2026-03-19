# Contributing to Mahalaxmi

Thank you for your interest in contributing to Mahalaxmi.
Please read this document before submitting your first pull request.

## Contributor License Agreement

Before your first pull request can be merged, you must accept our
[Contributor License Agreement](CLA.md).

When you open your first PR, a bot will post a comment explaining this
and asking you to reply with:

```
I have read and agree to the CLA.
```

That's it — no email, no DocuSign. Your acceptance is recorded by GitHub
username in this repository's `.github/cla-signatures.json`.

For corporate CLAs (contributing on behalf of a company), contact
legal@mahalaxmi.ai before submitting.

## Contribution Scope

Community contributions are accepted in these areas only:

- **Provider plugins** — new AiProvider trait implementations
- **Detection rules** — new state detection patterns
- **Language parsers** — new Tree-sitter grammar integrations
- **CLI commands** — new commands or improvements to mahalaxmi-cli
- **Bug fixes** — corrections to existing behavior in any public crate
- **Documentation** — README, docs site, inline documentation

Contributions that touch licensing systems, cloud infrastructure,
or commercial features will not be accepted.

## How to Contribute

1. Fork the repository
2. Create a branch: `git checkout -b feature/your-feature-name`
3. Make your changes
4. Run `cargo test --workspace` — all tests must pass
5. Run `cargo clippy --workspace -- -D warnings` — must be clean
6. Open a pull request against `main`

## Code Style

- Follow standard Rust formatting: `cargo fmt --all`
- All public items must have doc comments
- No `unwrap()` in library code — use proper error handling

## Questions

Open a GitHub Discussion or email support@mahalaxmi.ai

---

*Mahalaxmi™ is a trademark of ThriveTech Services LLC*
