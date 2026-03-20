use serde::{Deserialize, Serialize};

/// Configuration for a PTY terminal instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Number of rows (height). Default: 24.
    pub rows: u16,
    /// Number of columns (width). Default: 80.
    pub cols: u16,
    /// Maximum number of scrollback lines to retain. Default: 10000.
    pub scrollback_lines: u32,
    /// Whether to log terminal output to disk. Default: false.
    pub enable_logging: bool,
    /// Whether to emit verbose tracing logs for terminal I/O operations.
    /// When true, debug-level logs are emitted for write_input, read_output,
    /// resize, and output_snapshot. Lifecycle logs (spawn, exit, kill) are
    /// always emitted regardless of this setting. Default: false.
    pub verbose_logging: bool,
    /// Maximum size in bytes for the raw PTY replay ring buffer.
    /// Default: 2 MB. The 2 MB cap accommodates large AI provider stream-json
    /// responses while still bounding memory per terminal. A `tracing::warn!`
    /// is emitted when the buffer crosses 75% capacity.
    pub raw_replay_capacity_bytes: u32,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            rows: 24,
            cols: 80,
            scrollback_lines: 10_000,
            enable_logging: false,
            verbose_logging: false,
            raw_replay_capacity_bytes: 2 * 1024 * 1024,
        }
    }
}
