use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TerminalConfig, TerminalId, TerminalState};
use mahalaxmi_core::MahalaxmiResult;
use portable_pty::{Child, PtyPair};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

use crate::buffer::OutputBuffer;
use crate::vt_cleaner::VtCleaner;

/// A managed PTY terminal with state tracking and output buffering.
pub struct ManagedTerminal {
    id: TerminalId,
    state: TerminalState,
    master: Box<dyn portable_pty::MasterPty + Send>,
    /// Writer handle taken once from the master PTY. `take_writer()` can only
    /// be called once on a `portable_pty::MasterPty`, so we store the result
    /// here and reuse it for every `write_input` call.
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    output_buffer: Arc<Mutex<OutputBuffer>>,
    vt_cleaner: VtCleaner,
    config: TerminalConfig,
}

impl ManagedTerminal {
    /// Create a new managed terminal from a PTY pair and child process.
    pub(crate) fn new(
        id: TerminalId,
        pair: PtyPair,
        child: Box<dyn Child + Send + Sync>,
        config: TerminalConfig,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Self> {
        let output_buffer = Arc::new(Mutex::new(OutputBuffer::new(
            config.scrollback_lines as usize,
            config.raw_replay_capacity_bytes as usize,
        )));

        // Take the writer handle once — portable_pty only allows a single
        // take_writer() call per MasterPty, so we store it for reuse.
        let writer = pair.master.take_writer().map_err(|e| {
            MahalaxmiError::pty(
                i18n,
                keys::pty::WRITE_FAILED,
                &[("terminal_id", &id.to_string()), ("reason", &e.to_string())],
            )
        })?;

        info!(
            terminal_id = %id,
            rows = config.rows,
            cols = config.cols,
            scrollback = config.scrollback_lines,
            verbose_logging = config.verbose_logging,
            "Terminal created"
        );

        Ok(Self {
            id,
            state: TerminalState::Running,
            master: pair.master,
            writer,
            child,
            output_buffer,
            vt_cleaner: VtCleaner::new(),
            config,
        })
    }

    /// Get the terminal ID.
    pub fn id(&self) -> TerminalId {
        self.id
    }

    /// Get the current terminal state.
    pub fn state(&self) -> TerminalState {
        self.state
    }

    /// Write input bytes to the terminal (simulate user typing).
    pub fn write_input(&mut self, data: &[u8], i18n: &I18nService) -> MahalaxmiResult<()> {
        if self.config.verbose_logging {
            debug!(
                terminal_id = %self.id,
                bytes = data.len(),
                "Writing input to terminal"
            );
        }

        self.writer.write_all(data).map_err(|e| {
            MahalaxmiError::pty(
                i18n,
                keys::pty::WRITE_FAILED,
                &[
                    ("terminal_id", &self.id.to_string()),
                    ("reason", &e.to_string()),
                ],
            )
        })?;
        Ok(())
    }

    /// Read available output from the terminal, strip VT codes, buffer the result.
    pub fn read_output(&mut self, i18n: &I18nService) -> MahalaxmiResult<String> {
        let mut reader = self.master.try_clone_reader().map_err(|e| {
            MahalaxmiError::pty(
                i18n,
                keys::pty::READ_FAILED,
                &[
                    ("terminal_id", &self.id.to_string()),
                    ("reason", &e.to_string()),
                ],
            )
        })?;

        let mut raw = vec![0u8; 4096];
        let n = reader.read(&mut raw).unwrap_or(0);
        if n == 0 {
            return Ok(String::new());
        }

        let clean = self.vt_cleaner.clean(&raw[..n]);
        if let Ok(mut buf) = self.output_buffer.lock() {
            buf.push_text(&clean);
        }

        if self.config.verbose_logging {
            debug!(
                terminal_id = %self.id,
                raw_bytes = n,
                clean_len = clean.len(),
                "Read output from terminal"
            );
        }

        Ok(clean)
    }

    /// Get a snapshot of the output buffer.
    pub fn output_snapshot(&self) -> Vec<String> {
        let snapshot = self
            .output_buffer
            .lock()
            .map(|buf| buf.lines().iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        if self.config.verbose_logging {
            debug!(
                terminal_id = %self.id,
                lines = snapshot.len(),
                "Output snapshot taken"
            );
        }

        snapshot
    }

    /// Get the last N lines from the output buffer (efficiently, without
    /// copying the entire buffer). Used by the driver for stream completion
    /// marker detection — checking only the tail avoids scanning all output.
    pub fn output_tail(&self, n: usize) -> Vec<String> {
        self.output_buffer
            .lock()
            .map(|buf| buf.tail(n).into_iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Resize the terminal.
    pub fn resize(&self, rows: u16, cols: u16, i18n: &I18nService) -> MahalaxmiResult<()> {
        if self.config.verbose_logging {
            debug!(
                terminal_id = %self.id,
                rows,
                cols,
                "Resizing terminal"
            );
        }

        self.master
            .resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                MahalaxmiError::pty(
                    i18n,
                    keys::pty::RESIZE_FAILED,
                    &[
                        ("terminal_id", &self.id.to_string()),
                        ("rows", &rows.to_string()),
                        ("cols", &cols.to_string()),
                        ("reason", &e.to_string()),
                    ],
                )
            })
    }

    /// Check if the child process has exited. Returns the exit code if it has.
    pub fn try_wait(&mut self, i18n: &I18nService) -> MahalaxmiResult<Option<u32>> {
        let status = self.child.try_wait().map_err(|e| {
            MahalaxmiError::pty(
                i18n,
                keys::pty::WAIT_FAILED,
                &[
                    ("terminal_id", &self.id.to_string()),
                    ("reason", &e.to_string()),
                ],
            )
        })?;
        if let Some(exit) = &status {
            let success = exit.success();
            self.state = if success {
                TerminalState::Stopped
            } else {
                TerminalState::Failed
            };
            let exit_code: u32 = if success { 0 } else { 1 };
            info!(
                terminal_id = %self.id,
                exit_code,
                new_state = ?self.state,
                "Terminal process exited"
            );
        }
        Ok(status.map(|s| if s.success() { 0 } else { 1 }))
    }

    /// Return the OS process ID of the PTY child process, if available.
    ///
    /// Used by the build infrastructure stall detection to distinguish a
    /// legitimately busy compiler (active compiler child processes) from a
    /// genuinely stuck agent process.
    pub fn child_pid(&mut self) -> Option<u32> {
        self.child.process_id()
    }

    /// Kill the child process.
    pub fn kill(&mut self, i18n: &I18nService) -> MahalaxmiResult<()> {
        info!(terminal_id = %self.id, "Killing terminal process");
        self.state = TerminalState::Stopping;
        self.child.kill().map_err(|e| {
            MahalaxmiError::pty(
                i18n,
                keys::pty::KILL_FAILED,
                &[
                    ("terminal_id", &self.id.to_string()),
                    ("reason", &e.to_string()),
                ],
            )
        })?;
        self.state = TerminalState::Stopped;
        info!(terminal_id = %self.id, "Terminal process killed");
        Ok(())
    }

    /// Clone the PTY reader handle for use by the async reader task.
    ///
    /// Each call returns a new independent reader. Only one reader task should
    /// be spawned per terminal — multiple concurrent readers would split output
    /// unpredictably between them.
    pub fn try_clone_reader(&self, i18n: &I18nService) -> MahalaxmiResult<Box<dyn Read + Send>> {
        self.master.try_clone_reader().map_err(|e| {
            MahalaxmiError::pty(
                i18n,
                keys::pty::READ_FAILED,
                &[
                    ("terminal_id", &self.id.to_string()),
                    ("reason", &e.to_string()),
                ],
            )
        })
    }

    /// Return the cumulative count of all raw bytes ever pushed to this
    /// terminal's output buffer.
    ///
    /// Unlike the ring-buffer size (which plateaus at capacity), this counter
    /// grows monotonically. The orchestration idle detector uses it to
    /// distinguish "buffer full but still receiving" from "truly idle".
    pub fn total_raw_bytes(&self) -> usize {
        self.output_buffer
            .lock()
            .map(|buf| buf.total_raw_bytes())
            .unwrap_or(0)
    }

    /// Get a snapshot of the raw byte replay buffer.
    ///
    /// Returns all raw PTY bytes accumulated since the terminal started (up to
    /// the ring-buffer capacity limit), with ANSI escape sequences intact.
    /// Feed these directly to xterm.js `write()` to restore the terminal state
    /// for panels that mount after output has already arrived.
    pub fn raw_replay_snapshot(&self) -> Vec<u8> {
        self.output_buffer
            .lock()
            .map(|buf| buf.raw_replay().to_vec())
            .unwrap_or_default()
    }

    /// Get the terminal configuration.
    pub fn config(&self) -> &TerminalConfig {
        &self.config
    }

    /// Get a reference to the output buffer.
    pub fn output_buffer(&self) -> &Arc<Mutex<OutputBuffer>> {
        &self.output_buffer
    }
}
