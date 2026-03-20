use bytes::Bytes;
use mahalaxmi_core::types::TerminalId;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::debug;

use crate::buffer::OutputBuffer;
use crate::events::TerminalEvent;
use crate::vt_cleaner::VtCleaner;

/// Spawn an async task that continuously reads from a PTY reader and sends events.
///
/// **Chunk reassembly**: PTY output arrives in arbitrary byte chunks that may split:
/// - Mid-line — reassembled by `OutputBuffer.push_text()`
/// - Mid-UTF-8 character — reassembled by `VtCleaner.clean()`
/// - Mid-escape sequence — reassembled by the VTE state machine parser
///
/// On EOF, both are flushed to commit any pending partial data.
pub fn spawn_reader_task(
    terminal_id: TerminalId,
    mut reader: Box<dyn Read + Send>,
    event_tx: broadcast::Sender<TerminalEvent>,
    output_buffer: Arc<Mutex<OutputBuffer>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        tracing::info!(
            terminal_id = %terminal_id,
            receivers = event_tx.receiver_count(),
            "PTY reader task started — waiting for output"
        );

        let mut vt_cleaner = VtCleaner::new();
        let mut buf = vec![0u8; 4096];
        let mut total_bytes: usize = 0;
        let mut chunk_count: usize = 0;

        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    tracing::info!(
                        terminal_id = %terminal_id,
                        total_bytes = total_bytes,
                        chunk_count = chunk_count,
                        "PTY reader: EOF"
                    );

                    let trailing = vt_cleaner.flush();
                    if !trailing.is_empty() {
                        if let Ok(mut ob) = output_buffer.lock() {
                            ob.push_text(&trailing);
                        }
                    }

                    if let Ok(mut ob) = output_buffer.lock() {
                        ob.flush();
                    }

                    break;
                }
                Ok(n) => {
                    chunk_count += 1;
                    total_bytes += n;

                    if chunk_count <= 3 {
                        let preview = String::from_utf8_lossy(&buf[..n.min(200)]);
                        tracing::info!(
                            terminal_id = %terminal_id,
                            bytes = n,
                            chunk = chunk_count,
                            preview = %preview,
                            "PTY reader: chunk #{} received",
                            chunk_count,
                        );
                    }

                    debug!(
                        terminal_id = %terminal_id,
                        bytes = n,
                        total_bytes = total_bytes,
                        chunk = chunk_count,
                        "PTY reader: chunk"
                    );

                    let raw_data = Bytes::copy_from_slice(&buf[..n]);

                    // Store raw bytes for replay before emitting event, so
                    // get_terminal_output can return ANSI-intact bytes for xterm.js.
                    if let Ok(mut ob) = output_buffer.lock() {
                        ob.push_raw(&buf[..n]);
                    }

                    let _ = event_tx.send(TerminalEvent::OutputReceived {
                        terminal_id,
                        data: raw_data,
                    });

                    let clean_text = vt_cleaner.clean(&buf[..n]);
                    if !clean_text.is_empty() {
                        if let Ok(mut ob) = output_buffer.lock() {
                            ob.push_text(&clean_text);
                        }

                        let _ = event_tx.send(TerminalEvent::TextOutput {
                            terminal_id,
                            text: clean_text,
                        });
                    }
                }
                Err(e) => {
                    tracing::info!(
                        terminal_id = %terminal_id,
                        error = %e,
                        total_bytes = total_bytes,
                        chunk_count = chunk_count,
                        "PTY reader: error (process may have exited)"
                    );

                    let trailing = vt_cleaner.flush();
                    if !trailing.is_empty() {
                        if let Ok(mut ob) = output_buffer.lock() {
                            ob.push_text(&trailing);
                        }
                    }
                    if let Ok(mut ob) = output_buffer.lock() {
                        ob.flush();
                    }

                    break;
                }
            }
        }
    })
}
