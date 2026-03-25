use bytes::Bytes;
use mahalaxmi_core::types::{TerminalId, TerminalState};

/// Events emitted by a managed terminal.
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Raw bytes received from the PTY output stream.
    OutputReceived {
        terminal_id: TerminalId,
        data: Bytes,
    },
    /// Clean text output after VT escape sequence stripping.
    TextOutput {
        terminal_id: TerminalId,
        text: String,
    },
    /// Terminal state has changed.
    StateChanged {
        terminal_id: TerminalId,
        old_state: TerminalState,
        new_state: TerminalState,
    },
    /// The process inside the PTY has exited.
    ProcessExited {
        terminal_id: TerminalId,
        exit_code: i32,
    },
}
