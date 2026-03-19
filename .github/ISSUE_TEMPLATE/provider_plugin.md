---
name: Add a Provider Plugin
about: Claim a slot and implement support for a new AI CLI tool. This is the primary contribution path.
labels: provider-plugin
assignees: ''
---

## Provider Name

Name of the AI CLI tool (e.g., "Aider", "Cursor", "Continue").

## Provider CLI Binary

The binary name this tool installs (e.g., `aider`, `cursor`, `continue`).

## Provider Website / Repository

Link to the provider's official site or repo.

## Why This Provider

Why would this provider be valuable to Mahalaxmi users?
(e.g., open-source alternative, unique model access, privacy-friendly, enterprise SSO)

## Implementation

- [ ] I will implement this and submit a PR
- [ ] I am requesting someone else implement it

If implementing: paste a rough sketch of `build_command` and `validate_credentials`
to confirm you understand the interface before you invest time in the full PR.
See [`ollama.rs`](../../crates/mahalaxmi-providers/src/ollama.rs) for a reference implementation.

## Additional Context

Auth method, install instructions, any quirks with the CLI's output format
that would affect `output_markers` or `extract_response`.
