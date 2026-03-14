# mahalaxmi-pty

Pseudo-terminal spawning, stream I/O, VT sequence parsing, and multi-session management for Mahalaxmi AI worker processes.

## Overview

`mahalaxmi-pty` replaces the fragile OCR-based screen capture used in the original Ganesha platform with direct PTY stream interception. Each AI worker agent runs inside a managed pseudo-terminal; its output is captured byte-for-byte, VT escape sequences are stripped for clean text analysis, and the raw bytes are preserved for UI replay.

The crate manages the full lifecycle of PTY sessions: spawning child processes with isolated environments (each worker gets only its own provider's credentials in `env`), reading output through a non-blocking reader, buffering clean text and raw bytes separately, and broadcasting `TerminalEvent`s to any number of subscribers. `TerminalSessionManager` enforces separate concurrency limits for orchestration terminals (managers and workers) and utility terminals (install, login, test commands).

Orchestration engine developers will use `PtySpawner` and `TerminalSessionManager` to launch and monitor AI agents. UI developers subscribe to the event stream and replay raw bytes through `xterm.js`. Most end users never interact with this crate directly.

## Key Types

| Type | Kind | Description |
|------|------|-------------|
| `PtySpawner` | Struct | Spawns PTY processes from `ProcessCommand`; returns a `ManagedTerminal` |
| `ManagedTerminal` | Struct | Wraps a live PTY pair; provides read/write and output buffering |
| `OutputBuffer` | Struct | Thread-safe buffer of clean text (for detection) + raw bytes (for UI replay) |
| `TerminalSessionManager` | Struct | Manages multiple concurrent PTY sessions; enforces capacity limits |
| `TerminalEvent` | Enum | Events broadcast by sessions: `OutputReceived`, `SessionEnded`, `ErrorOccurred` |
| `VtCleaner` | Struct | Strips VT/ANSI escape sequences from raw terminal output |
| `TerminalId` | Struct | Unique handle for a managed PTY session |
| `TerminalState` | Enum | `Idle`, `Running`, `Paused`, `Terminated` |
| `TerminalPurpose` | Enum | `Orchestration` (manager/worker) or `Utility` (install/login/test) |

## Key Functions / Methods

| Function | Description |
|----------|-------------|
| `PtySpawner::spawn(command, config, id, i18n)` | Spawn a new PTY process; returns `ManagedTerminal` |
| `TerminalSessionManager::new(config, i18n)` | Create a session manager with limits from `OrchestrationConfig` |
| `TerminalSessionManager::spawn_terminal(cmd, cfg, purpose, i18n)` | Spawn and register a terminal; enforces capacity |
| `TerminalSessionManager::subscribe()` | Subscribe to the broadcast event channel |
| `ManagedTerminal::read_output(i18n)` | Read available output into the internal buffer |
| `ManagedTerminal::write_input(text, i18n)` | Write text to the terminal's stdin |
| `OutputBuffer::clean_text_snapshot()` | Get a copy of the clean (VT-stripped) text seen so far |
| `OutputBuffer::raw_replay_snapshot()` | Get the raw bytes for UI replay (capped at 512 KB) |
| `VtCleaner::clean(raw)` | Strip ANSI/VT sequences from a byte slice |
| `compute_channel_capacity(max_workers)` | Compute broadcast buffer size from worker count |

## Feature Flags

No feature flags.

## Dependencies

| Dependency | Why |
|-----------|-----|
| `portable-pty` | Cross-platform PTY creation (Windows ConPTY, Unix openpty) |
| `vte` | VT/ANSI sequence parser for clean text extraction |
| `tokio` | Async runtime for non-blocking I/O |
| `bytes` | Efficient byte buffer management |
| `mahalaxmi-core` | Shared types, config, errors, i18n |

## Stability

**Unstable** — API may change in minor versions during the pre-1.0 period.

## License

MIT — Copyright 2026 ThriveTech Services LLC
