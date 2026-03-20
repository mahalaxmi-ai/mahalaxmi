use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ProcessCommand, TerminalConfig, TerminalId};
use mahalaxmi_core::MahalaxmiResult;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tracing::info;

use crate::terminal::ManagedTerminal;

/// Spawns PTY processes from `ProcessCommand` specifications.
pub struct PtySpawner;

impl PtySpawner {
    /// Spawn a new PTY terminal running the given command.
    ///
    /// Returns a `ManagedTerminal` that wraps the PTY pair and provides
    /// read/write access to the terminal streams.
    pub fn spawn(
        command: &ProcessCommand,
        config: &TerminalConfig,
        terminal_id: TerminalId,
        i18n: &I18nService,
    ) -> MahalaxmiResult<ManagedTerminal> {
        let pty_system = native_pty_system();

        let pty_size = PtySize {
            rows: config.rows,
            cols: config.cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(pty_size).map_err(|e| {
            MahalaxmiError::pty(i18n, keys::pty::OPEN_FAILED, &[("reason", &e.to_string())])
        })?;

        let mut cmd = CommandBuilder::new(&command.program);
        for arg in &command.args {
            cmd.arg(arg);
        }
        // Isolate worker environment: clear all inherited env vars to prevent
        // concurrent workers from reading each other's API keys, then inject
        // only the credentials belonging to this specific worker's provider.
        cmd.env_clear();
        for (key, value) in &command.env {
            cmd.env(key, value);
        }
        // Restore minimal host variables required for CLIs to locate binaries
        // and their own config files.
        if let Ok(path) = std::env::var("PATH") {
            cmd.env("PATH", path);
        }
        if let Ok(home) = std::env::var("HOME") {
            cmd.env("HOME", home);
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(v) = std::env::var("USERPROFILE") {
                cmd.env("USERPROFILE", v);
            }
            if let Ok(v) = std::env::var("APPDATA") {
                cmd.env("APPDATA", v);
            }
            if let Ok(v) = std::env::var("LOCALAPPDATA") {
                cmd.env("LOCALAPPDATA", v);
            }
        }
        if let Some(dir) = &command.working_dir {
            cmd.cwd(dir);
        }

        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            MahalaxmiError::pty(
                i18n,
                keys::pty::SPAWN_FAILED,
                &[("program", &command.program), ("reason", &e.to_string())],
            )
        })?;

        info!(
            terminal_id = %terminal_id,
            program = %command.program,
            "PTY process spawned"
        );

        ManagedTerminal::new(terminal_id, pair, child, config.clone(), i18n)
    }
}
