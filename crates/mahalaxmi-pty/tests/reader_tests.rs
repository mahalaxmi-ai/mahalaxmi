//! Tests for the PTY reader task and bidirectional message routing.
//!
//! Covers the full data path from raw PTY bytes → broadcast events → output buffer,
//! which is the critical pipeline for routing AI provider output to the frontend.

use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

use mahalaxmi_pty::{OutputBuffer, TerminalEvent, TerminalId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_reader(data: &[u8]) -> Box<dyn std::io::Read + Send> {
    Box::new(Cursor::new(data.to_vec()))
}

fn make_buffer(capacity: usize) -> Arc<Mutex<OutputBuffer>> {
    Arc::new(Mutex::new(OutputBuffer::new(capacity, 512 * 1024)))
}

fn make_channel() -> (
    broadcast::Sender<TerminalEvent>,
    broadcast::Receiver<TerminalEvent>,
) {
    broadcast::channel(64)
}

// Drain all currently available events from the receiver without blocking.
fn collect_events(rx: &mut broadcast::Receiver<TerminalEvent>) -> Vec<TerminalEvent> {
    let mut events = Vec::new();
    loop {
        match rx.try_recv() {
            Ok(e) => events.push(e),
            Err(_) => break,
        }
    }
    events
}

// ---------------------------------------------------------------------------
// Reader task event emission tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reader_task_emits_output_received_event() {
    let id = TerminalId::new();
    let data = b"hello from PTY\r\n";
    let (tx, mut rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let _ = handle.await;
    // Allow task to flush
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = collect_events(&mut rx);
    let raw_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TerminalEvent::OutputReceived { .. }))
        .collect();
    assert!(
        !raw_events.is_empty(),
        "Expected at least one OutputReceived event"
    );
}

#[tokio::test]
async fn reader_task_emits_text_output_event() {
    let id = TerminalId::new();
    let data = b"hello world\n";
    let (tx, mut rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = collect_events(&mut rx);
    let text_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TerminalEvent::TextOutput { .. }))
        .collect();
    assert!(
        !text_events.is_empty(),
        "Expected at least one TextOutput event"
    );
}

#[tokio::test]
async fn reader_task_emits_both_event_types_for_text_chunk() {
    let id = TerminalId::new();
    // Printable ASCII text produces both raw and clean-text events
    let data = b"line one\nline two\n";
    let (tx, mut rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = collect_events(&mut rx);
    let has_raw = events
        .iter()
        .any(|e| matches!(e, TerminalEvent::OutputReceived { .. }));
    let has_text = events
        .iter()
        .any(|e| matches!(e, TerminalEvent::TextOutput { .. }));
    assert!(has_raw, "Expected OutputReceived event");
    assert!(has_text, "Expected TextOutput event");
}

#[tokio::test]
async fn reader_task_output_received_contains_correct_terminal_id() {
    let id = TerminalId::new();
    let data = b"data\n";
    let (tx, mut rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = collect_events(&mut rx);
    for event in &events {
        match event {
            TerminalEvent::OutputReceived { terminal_id, .. } => {
                assert_eq!(*terminal_id, id);
            }
            TerminalEvent::TextOutput { terminal_id, .. } => {
                assert_eq!(*terminal_id, id);
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn reader_task_output_received_contains_raw_bytes() {
    let id = TerminalId::new();
    let data = b"hello\n";
    let (tx, mut rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = collect_events(&mut rx);
    let raw: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let TerminalEvent::OutputReceived { data, .. } = e {
                Some(data.clone())
            } else {
                None
            }
        })
        .collect();
    assert!(!raw.is_empty());
    let combined: Vec<u8> = raw.into_iter().flat_map(|b| b.to_vec()).collect();
    assert!(combined.windows(5).any(|w| w == b"hello"));
}

#[tokio::test]
async fn reader_task_ansi_output_emits_raw_with_sequences() {
    let id = TerminalId::new();
    // ANSI escape sequences that AI providers like Claude Code emit
    let data = b"\x1b[32mWelcome\x1b[0m\r\n\x1b[1mClaude Code\x1b[0m\r\n";
    let (tx, mut rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = collect_events(&mut rx);
    // OutputReceived should have the raw bytes including ANSI sequences
    let raw_bytes: Vec<u8> = events
        .iter()
        .filter_map(|e| {
            if let TerminalEvent::OutputReceived { data, .. } = e {
                Some(data.to_vec())
            } else {
                None
            }
        })
        .flat_map(|v| v)
        .collect();
    assert!(
        raw_bytes.contains(&0x1b),
        "OutputReceived should preserve ESC bytes for ANSI sequences"
    );
}

#[tokio::test]
async fn reader_task_ansi_output_text_output_strips_sequences() {
    let id = TerminalId::new();
    let data = b"\x1b[32mWelcome\x1b[0m\n";
    let (tx, mut rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = collect_events(&mut rx);
    for event in &events {
        if let TerminalEvent::TextOutput { text, .. } = event {
            // Clean text should not contain ESC sequences
            assert!(
                !text.contains('\x1b'),
                "TextOutput should strip ANSI sequences, got: {:?}",
                text
            );
        }
    }
}

#[tokio::test]
async fn reader_task_empty_reader_emits_no_events() {
    let id = TerminalId::new();
    let (tx, mut rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(b""), tx, buf);
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events = collect_events(&mut rx);
    assert!(
        events.is_empty(),
        "Expected no events for empty reader, got: {}",
        events.len()
    );
}

// ---------------------------------------------------------------------------
// Raw buffer population tests (verifies the replay fix)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reader_task_populates_raw_replay_buffer() {
    let id = TerminalId::new();
    let data = b"raw replay data\r\n";
    let (tx, _rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf.clone());
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let locked = buf.lock().unwrap();
    assert!(
        locked.raw_len() > 0,
        "Raw replay buffer should contain the PTY bytes"
    );
}

#[tokio::test]
async fn reader_task_raw_replay_contains_original_bytes() {
    let id = TerminalId::new();
    let data = b"marker_content\n";
    let (tx, _rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf.clone());
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let locked = buf.lock().unwrap();
    let replay = locked.raw_replay();
    assert!(
        replay.windows(14).any(|w| w == b"marker_content"),
        "Raw replay should contain original bytes"
    );
}

#[tokio::test]
async fn reader_task_raw_replay_preserves_ansi_for_xterm() {
    let id = TerminalId::new();
    // Simulate Claude Code startup output with rich ANSI formatting
    let startup = b"\x1b[2J\x1b[H\x1b[1;36mClaude Code\x1b[0m v1.0\r\n\x1b[32m>\x1b[0m ";
    let (tx, _rx) = make_channel();
    let buf = make_buffer(100);

    let handle =
        mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(startup), tx, buf.clone());
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let locked = buf.lock().unwrap();
    let replay = locked.raw_replay();
    // The raw replay must contain ESC bytes so xterm.js can render colors
    assert!(
        replay.contains(&0x1b),
        "Raw replay must preserve ANSI escape bytes for xterm.js"
    );
    // And the text content
    assert!(
        replay.windows(10).any(|w| w == b"Claude Cod"),
        "Raw replay should contain visible text"
    );
}

#[tokio::test]
async fn reader_task_multiple_chunks_all_in_raw_replay() {
    let id = TerminalId::new();
    // Simulate multiple PTY read chunks being accumulated
    let chunk1 = b"first chunk\r\n";
    let chunk2 = b"second chunk\r\n";
    let mut combined = chunk1.to_vec();
    combined.extend_from_slice(chunk2);
    let (tx, _rx) = make_channel();
    let buf = make_buffer(100);

    let handle =
        mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(&combined), tx, buf.clone());
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let locked = buf.lock().unwrap();
    let replay = locked.raw_replay();
    assert!(replay.windows(11).any(|w| w == b"first chunk"));
    assert!(replay.windows(12).any(|w| w == b"second chunk"));
}

// ---------------------------------------------------------------------------
// Clean text buffer population tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reader_task_populates_clean_text_buffer() {
    let id = TerminalId::new();
    let data = b"visible text\n";
    let (tx, _rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf.clone());
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let locked = buf.lock().unwrap();
    assert!(locked.len() > 0, "Clean text buffer should have lines");
    let lines: Vec<String> = locked.lines().iter().cloned().collect();
    assert!(
        lines.iter().any(|l| l.contains("visible text")),
        "Clean text buffer should contain stripped text"
    );
}

#[tokio::test]
async fn reader_task_ansi_stripped_in_clean_buffer() {
    let id = TerminalId::new();
    let data = b"\x1b[31mred text\x1b[0m\n";
    let (tx, _rx) = make_channel();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf.clone());
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let locked = buf.lock().unwrap();
    let lines: Vec<String> = locked.lines().iter().cloned().collect();
    for line in &lines {
        assert!(
            !line.contains('\x1b'),
            "Clean buffer must not contain ANSI escape sequences, got: {:?}",
            line
        );
    }
}

// ---------------------------------------------------------------------------
// Event channel capacity / no-subscriber tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reader_task_completes_even_with_no_subscribers() {
    let id = TerminalId::new();
    let data = b"some output\n";
    let (tx, rx) = make_channel();
    // Drop the receiver — no subscriber
    drop(rx);
    let buf = make_buffer(100);

    // Should complete without panicking
    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let result = handle.await;
    assert!(
        result.is_ok(),
        "Reader task should not panic with no subscribers"
    );
}

#[tokio::test]
async fn reader_task_multiple_subscribers_all_receive_events() {
    let id = TerminalId::new();
    let data = b"broadcast test\n";
    let (tx, mut rx1) = make_channel();
    let mut rx2 = tx.subscribe();
    let buf = make_buffer(100);

    let handle = mahalaxmi_pty::reader::spawn_reader_task(id, make_reader(data), tx, buf);
    let _ = handle.await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let events1 = collect_events(&mut rx1);
    let events2 = collect_events(&mut rx2);

    assert!(
        !events1.is_empty(),
        "First subscriber should receive events"
    );
    assert!(
        !events2.is_empty(),
        "Second subscriber should receive events"
    );
    assert_eq!(
        events1.len(),
        events2.len(),
        "All subscribers should receive the same number of events"
    );
}

// ---------------------------------------------------------------------------
// ManagedTerminal raw_replay_snapshot integration test
// ---------------------------------------------------------------------------

#[test]
fn managed_terminal_raw_replay_snapshot_after_read() {
    use mahalaxmi_pty::{I18nService, ProcessCommand, PtySpawner, SupportedLocale, TerminalConfig};

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let cmd = ProcessCommand::new("echo").arg("replay test content");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let mut terminal = PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    std::thread::sleep(Duration::from_millis(500));
    let _ = terminal.read_output(&i18n);

    // raw_replay_snapshot is populated by the reader task (spawn_reader_task),
    // not by read_output (which only updates clean text buffer).
    // This verifies the method exists and returns a Vec<u8>.
    let snapshot = terminal.raw_replay_snapshot();
    let _ = snapshot; // Type is Vec<u8>

    let _ = terminal.kill(&i18n);
}

#[test]
fn managed_terminal_raw_replay_snapshot_returns_vec_u8() {
    use mahalaxmi_pty::{I18nService, ProcessCommand, PtySpawner, SupportedLocale, TerminalConfig};

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let cmd = ProcessCommand::new("sleep").arg("10");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let mut terminal = PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();
    let snapshot: Vec<u8> = terminal.raw_replay_snapshot();
    // No output yet — should be empty
    assert!(snapshot.is_empty());
    let _ = terminal.kill(&i18n);
}
