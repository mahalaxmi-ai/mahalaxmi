//! PTY terminal management for Mahalaxmi.
//!
//! Handles pseudo-terminal spawning, stream I/O, and VT parsing.
//! Replaces Ganesha's OCR-based screen capture with direct PTY stream interception.

pub mod buffer;
pub mod events;
pub mod reader;
pub mod session;
pub mod spawner;
pub mod terminal;
pub mod vt_cleaner;

pub use buffer::{OutputBuffer, RAW_REPLAY_BUFFER_BYTES};
#[allow(deprecated)]
pub use buffer::DEFAULT_RAW_REPLAY_CAPACITY_BYTES;
pub use events::TerminalEvent;
pub use session::TerminalSessionManager;
pub use spawner::PtySpawner;
pub use terminal::ManagedTerminal;
pub use vt_cleaner::VtCleaner;

pub use mahalaxmi_core::config::MahalaxmiConfig;
pub use mahalaxmi_core::error::MahalaxmiError;
pub use mahalaxmi_core::i18n::locale::SupportedLocale;
pub use mahalaxmi_core::i18n::I18nService;
pub use mahalaxmi_core::types::{
    ProcessCommand, TerminalConfig, TerminalId, TerminalPurpose, TerminalState,
};
pub use mahalaxmi_core::MahalaxmiResult;
