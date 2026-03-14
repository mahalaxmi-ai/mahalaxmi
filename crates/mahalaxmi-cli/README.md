# mahalaxmi-cli

Command-line interface for controlling a running `mahalaxmi-service` instance via its REST API.

## Overview

`mahalaxmi-cli` is a thin HTTP client binary for headless and CI/CD use of Mahalaxmi. It communicates with a running `mahalaxmi-service` instance (the background orchestration daemon) over its REST API on port 17421. All commands — health checks, cycle management, event streaming — map directly to REST endpoints, making the CLI composable with shell scripts and CI pipelines.

The CLI is designed for three audiences: developers running Mahalaxmi in headless server mode without the desktop app, CI/CD pipelines that need to trigger and monitor orchestration cycles programmatically, and power users who prefer a terminal interface over the desktop GUI. The `mahalaxmi-service` daemon must be running separately; the CLI does not start it.

The service URL defaults to `http://127.0.0.1:17421` and can be overridden with `--service` or `MAHALAXMI_SERVICE_URL` for remote deployments.

## Key Types

This crate is a binary (`mahalaxmi-cli`), not a library. Its types are internal. The public interface is its command-line API:

| Command | Description |
|---------|-------------|
| `health` | Check service health, version, and uptime |
| `cycle start` | Start a new orchestration cycle; prints the cycle ID |
| `cycle status --id <ID>` | Show current status and worker states for a cycle |
| `cycle stop --id <ID>` | Stop a running cycle |
| `cycle approve --id <ID>` | Approve a plan awaiting human review |
| `events` | Stream all live SSE events from the service |
| `events --cycle <ID>` | Stream events for a specific cycle only |

## Key Functions / Methods

Not applicable — this is a binary crate.

## Feature Flags

No feature flags.

## Dependencies

| Dependency | Why |
|-----------|-----|
| `clap` | CLI argument parsing with subcommands |
| `reqwest` | HTTP client for REST API calls |
| `tokio` | Async runtime |
| `serde_json` | JSON request/response serialization |
| `futures-util` | SSE byte stream processing |

## Usage

```bash
# Check service health
mahalaxmi-cli health

# Start a cycle (prints cycle ID)
CYCLE_ID=$(mahalaxmi-cli cycle start \
  --project-root /path/to/project \
  --requirements "Add OpenTelemetry tracing to all HTTP handlers")

# Monitor status
mahalaxmi-cli cycle status --id "$CYCLE_ID"

# Stream live events
mahalaxmi-cli events --cycle "$CYCLE_ID" --pretty

# Connect to a remote service
mahalaxmi-cli --service http://build-server:17421 health
```

## Stability

**Unstable** — CLI interface may change in minor versions during the pre-1.0 period.

## License

MIT — Copyright 2026 ThriveTech Services LLC
