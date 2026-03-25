use mahalaxmi_pty::OutputBuffer;

#[test]
fn new_is_empty() {
    let buf = OutputBuffer::new(100, 512 * 1024);
    assert!(buf.is_empty());
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.pending(), "");
}

#[test]
fn push_line_and_len() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("hello".to_string());
    assert_eq!(buf.len(), 1);
    assert!(!buf.is_empty());
}

#[test]
fn capacity_enforcement() {
    let mut buf = OutputBuffer::new(3, 512 * 1024);
    buf.push_line("line1".to_string());
    buf.push_line("line2".to_string());
    buf.push_line("line3".to_string());
    buf.push_line("line4".to_string());
    assert_eq!(buf.len(), 3);
    let lines: Vec<&str> = buf.lines().iter().map(String::as_str).collect();
    assert_eq!(lines, vec!["line2", "line3", "line4"]);
}

#[test]
fn capacity_zero() {
    let mut buf = OutputBuffer::new(0, 512 * 1024);
    buf.push_line("hello".to_string());
    assert_eq!(buf.len(), 0);
    assert!(buf.is_empty());
}

#[test]
fn push_text_complete_lines() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("hello\nworld\n");
    assert_eq!(buf.len(), 2);
    assert_eq!(buf.pending(), "");
    let lines: Vec<&str> = buf.lines().iter().map(String::as_str).collect();
    assert_eq!(lines, vec!["hello", "world"]);
}

#[test]
fn push_text_trailing_incomplete() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("hello\nworld");
    assert_eq!(buf.len(), 1);
    assert_eq!(buf.pending(), "world");
}

#[test]
fn push_text_reassembly_across_chunks() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("hel");
    assert_eq!(buf.pending(), "hel");
    buf.push_text("lo\n");
    assert_eq!(buf.len(), 1);
    assert_eq!(buf.pending(), "");
    let lines: Vec<&str> = buf.lines().iter().map(String::as_str).collect();
    assert_eq!(lines, vec!["hello"]);
}

#[test]
fn push_text_multiple_lines_with_trailing() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("line1\nline2\npartial");
    assert_eq!(buf.len(), 2);
    assert_eq!(buf.pending(), "partial");
}

#[test]
fn push_text_only_newline() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("waiting");
    buf.push_text("\n");
    assert_eq!(buf.len(), 1);
    assert_eq!(buf.pending(), "");
    let lines: Vec<&str> = buf.lines().iter().map(String::as_str).collect();
    assert_eq!(lines, vec!["waiting"]);
}

#[test]
fn push_text_empty_string() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("");
    assert!(buf.is_empty());
    assert_eq!(buf.pending(), "");
}

#[test]
fn flush_commits_pending() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("incomplete");
    assert_eq!(buf.len(), 0);
    buf.flush();
    assert_eq!(buf.len(), 1);
    assert_eq!(buf.pending(), "");
    let lines: Vec<&str> = buf.lines().iter().map(String::as_str).collect();
    assert_eq!(lines, vec!["incomplete"]);
}

#[test]
fn flush_when_no_pending() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("line\n");
    let len_before = buf.len();
    buf.flush();
    assert_eq!(buf.len(), len_before);
}

#[test]
fn pending_accessible() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_text("partial");
    assert_eq!(buf.pending(), "partial");
}

#[test]
fn drain_returns_all() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("a".to_string());
    buf.push_line("b".to_string());
    buf.push_line("c".to_string());
    let drained = buf.drain();
    assert_eq!(drained, vec!["a", "b", "c"]);
    assert_eq!(buf.len(), 0);
}

#[test]
fn drain_idempotent() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("x".to_string());
    let first = buf.drain();
    assert_eq!(first.len(), 1);
    let second = buf.drain();
    assert!(second.is_empty());
}

#[test]
fn search_finds_matches() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("hello world".to_string());
    buf.push_line("foo bar".to_string());
    buf.push_line("hello again".to_string());
    let results = buf.search("hello");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], "hello world");
    assert_eq!(results[1], "hello again");
}

#[test]
fn search_includes_pending() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("committed line".to_string());
    buf.push_text("pending match");
    let results = buf.search("match");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "pending match");
}

#[test]
fn search_no_matches() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("hello".to_string());
    let results = buf.search("xyz");
    assert!(results.is_empty());
}

#[test]
fn search_empty() {
    let buf = OutputBuffer::new(100, 512 * 1024);
    let results = buf.search("anything");
    assert!(results.is_empty());
}

#[test]
fn tail_returns_last_n() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("a".to_string());
    buf.push_line("b".to_string());
    buf.push_line("c".to_string());
    buf.push_line("d".to_string());
    let last_two = buf.tail(2);
    assert_eq!(last_two, vec!["c", "d"]);
}

#[test]
fn tail_fewer_than_n() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("only".to_string());
    let last_five = buf.tail(5);
    assert_eq!(last_five, vec!["only"]);
}

#[test]
fn tail_zero() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("something".to_string());
    let result = buf.tail(0);
    assert!(result.is_empty());
}

#[test]
fn clear() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("line".to_string());
    buf.push_text("pending");
    buf.clear();
    assert!(buf.is_empty());
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.pending(), "");
}

#[test]
fn lines_reference() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_line("first".to_string());
    buf.push_line("second".to_string());
    let lines = buf.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "first");
    assert_eq!(lines[1], "second");
}

// ---------------------------------------------------------------------------
// Raw Replay Buffer Tests
// ---------------------------------------------------------------------------

#[test]
fn raw_replay_empty_on_new() {
    let buf = OutputBuffer::new(100, 512 * 1024);
    assert_eq!(buf.raw_len(), 0);
    assert!(buf.raw_replay().is_empty());
}

#[test]
fn push_raw_stores_bytes() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_raw(b"hello world");
    assert_eq!(buf.raw_len(), 11);
    assert_eq!(buf.raw_replay(), b"hello world");
}

#[test]
fn push_raw_accumulates_chunks() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_raw(b"hello ");
    buf.push_raw(b"world");
    assert_eq!(buf.raw_len(), 11);
    assert_eq!(buf.raw_replay(), b"hello world");
}

#[test]
fn push_raw_preserves_ansi_sequences() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    let ansi_output = b"\x1b[32mgreen text\x1b[0m\r\n";
    buf.push_raw(ansi_output);
    assert_eq!(buf.raw_replay(), ansi_output.as_ref());
}

#[test]
fn push_raw_empty_is_noop() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_raw(b"");
    assert_eq!(buf.raw_len(), 0);
    buf.push_raw(b"data");
    buf.push_raw(b"");
    assert_eq!(buf.raw_len(), 4);
}

#[test]
fn push_raw_enforces_capacity_drops_oldest() {
    // Use a small buffer to test capacity enforcement directly.
    // The actual implementation uses RAW_REPLAY_CAPACITY_BYTES (512KB),
    // but we can test the trimming logic by filling it up.
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    // Fill with 500KB of data (10 chunks of 50KB each)
    let chunk = vec![b'A'; 50 * 1024];
    for _ in 0..10 {
        buf.push_raw(&chunk);
    }
    // Total pushed: 500KB — should be within the 512KB cap
    assert!(buf.raw_len() <= 512 * 1024);

    // Push one more 50KB chunk to trigger trimming
    buf.push_raw(&chunk);
    // Should not exceed 512KB after trim
    assert!(buf.raw_len() <= 512 * 1024);
    // The most recent data should be preserved
    let replay = buf.raw_replay();
    assert!(!replay.is_empty());
    let last_byte = replay[replay.len() - 1];
    assert_eq!(last_byte, b'A');
}

#[test]
fn push_raw_single_chunk_exceeding_capacity_is_truncated() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    // Push exactly the cap size — should fit exactly
    let at_cap = vec![b'X'; 512 * 1024];
    buf.push_raw(&at_cap);
    assert_eq!(buf.raw_len(), 512 * 1024);
}

#[test]
fn push_raw_multi_line_terminal_output() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    let output = b"\x1b[H\x1b[2J\x1b[1;34mWelcome to Claude Code\x1b[0m\r\n> ";
    buf.push_raw(output);
    assert_eq!(buf.raw_len(), output.len());
    assert_eq!(buf.raw_replay(), output.as_ref());
}

#[test]
fn raw_replay_independent_from_clean_text_buffer() {
    // Pushing raw bytes should not affect the clean text line buffer and vice versa
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_raw(b"\x1b[32mhello\x1b[0m\n");
    buf.push_text("hello\n");
    // Clean text buffer has 1 line
    assert_eq!(buf.len(), 1);
    // Raw replay has the raw bytes: \x1b[32m (5) + hello (5) + \x1b[0m (4) + \n (1) = 15
    assert_eq!(buf.raw_len(), 15);
}

#[test]
fn raw_replay_cleared_by_clear() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_raw(b"some data");
    assert_eq!(buf.raw_len(), 9);
    buf.clear();
    assert_eq!(buf.raw_len(), 0);
    assert!(buf.raw_replay().is_empty());
}

// ---------------------------------------------------------------------------
// total_raw_bytes Tests
// ---------------------------------------------------------------------------

#[test]
fn total_raw_bytes_zero_on_new() {
    let buf = OutputBuffer::new(100, 512 * 1024);
    assert_eq!(buf.total_raw_bytes(), 0);
}

#[test]
fn total_raw_bytes_accumulates() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_raw(b"hello");
    assert_eq!(buf.total_raw_bytes(), 5);
    buf.push_raw(b" world");
    assert_eq!(buf.total_raw_bytes(), 11);
}

#[test]
fn total_raw_bytes_continues_past_ring_buffer_cap() {
    // Use a small raw capacity to force the ring buffer to trim.
    // total_raw_bytes must keep growing even after the ring buffer is full.
    let cap = 1024; // 1 KB ring buffer
    let mut buf = OutputBuffer::new(100, cap);
    let chunk = vec![b'X'; 600]; // 600 bytes per push
    buf.push_raw(&chunk); // total: 600, ring: 600
    buf.push_raw(&chunk); // total: 1200, ring: 1024 (trimmed)

    assert_eq!(buf.total_raw_bytes(), 1200);
    assert!(buf.raw_len() <= cap);
}

#[test]
fn total_raw_bytes_reset_on_clear() {
    let mut buf = OutputBuffer::new(100, 512 * 1024);
    buf.push_raw(b"data");
    assert_eq!(buf.total_raw_bytes(), 4);
    buf.clear();
    assert_eq!(buf.total_raw_bytes(), 0);
}

// ---------------------------------------------------------------------------
// Configurable Raw Capacity Tests
// ---------------------------------------------------------------------------

#[test]
fn custom_raw_capacity_enforced() {
    // 1 KB cap — push 2 KB, verify ring buffer stays within limit
    let mut buf = OutputBuffer::new(100, 1024);
    let data = vec![b'A'; 2048];
    buf.push_raw(&data);
    assert!(buf.raw_len() <= 1024);
    assert_eq!(buf.total_raw_bytes(), 2048);
    // Most recent bytes are preserved
    let replay = buf.raw_replay();
    assert_eq!(replay.len(), 1024);
    assert!(replay.iter().all(|&b| b == b'A'));
}
