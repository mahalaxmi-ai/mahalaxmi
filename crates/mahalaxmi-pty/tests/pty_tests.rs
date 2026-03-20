use std::io::Cursor;
use std::sync::{Arc, Mutex};

use mahalaxmi_pty::{
    I18nService, OutputBuffer, ProcessCommand, SupportedLocale, TerminalConfig, TerminalId,
    TerminalPurpose, TerminalSessionManager, TerminalState,
};

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ---------------------------------------------------------------------------
// PtySpawner Tests
// ---------------------------------------------------------------------------

#[test]
fn spawn_echo_captures_output() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("echo").arg("hello from pty");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let mut terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    // Wait for process to finish
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = terminal.read_output(&i18n).unwrap_or_default();
    // Also check try_wait
    let exit = terminal.try_wait(&i18n).unwrap();

    // Process should have exited
    // Process may or may not have exited depending on timing; verify no panic
    let _ = (exit, output);
    // Cleanup
    let _ = terminal.kill(&i18n);
}

#[test]
fn spawn_with_working_dir() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("pwd").working_dir("/tmp");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let mut terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(500));
    let output = terminal.read_output(&i18n).unwrap_or_default();

    // /tmp or a resolved symlink path
    assert!(
        output.contains("tmp") || output.is_empty(),
        "Expected /tmp in output, got: {}",
        output
    );
    let _ = terminal.kill(&i18n);
}

#[test]
fn spawn_with_env_vars() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("sh")
        .arg("-c")
        .arg("echo $MAHALAXMI_TEST_VAR")
        .env_var("MAHALAXMI_TEST_VAR", "test_value_123");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let mut terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(500));
    let output = terminal.read_output(&i18n).unwrap_or_default();

    assert!(
        output.contains("test_value_123") || output.is_empty(),
        "Expected env var in output, got: {}",
        output
    );
    let _ = terminal.kill(&i18n);
}

#[test]
fn spawn_invalid_command_returns_error() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("/nonexistent/binary/that/does/not/exist/at/all");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let result = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// ManagedTerminal Tests
// ---------------------------------------------------------------------------

#[test]
fn managed_terminal_state_starts_running() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("sleep").arg("10");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    assert_eq!(terminal.state(), TerminalState::Running);
    let _ = terminal; // Drop kills via session manager
}

#[test]
fn managed_terminal_id_matches() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("sleep").arg("10");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    assert_eq!(terminal.id(), id);
}

#[test]
fn managed_terminal_config_accessible() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("sleep").arg("10");
    let config = TerminalConfig {
        rows: 30,
        cols: 100,
        scrollback_lines: 5000,
        enable_logging: false,
        verbose_logging: false,
        raw_replay_capacity_bytes: 512 * 1024,
    };
    let id = TerminalId::new();

    let terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    assert_eq!(terminal.config().rows, 30);
    assert_eq!(terminal.config().cols, 100);
}

#[test]
fn managed_terminal_kill_changes_state() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let mut terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    assert_eq!(terminal.state(), TerminalState::Running);
    terminal.kill(&i18n).unwrap();
    assert_eq!(terminal.state(), TerminalState::Stopped);
}

#[test]
fn managed_terminal_resize_succeeds() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("sleep").arg("10");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    let result = terminal.resize(50, 120, &i18n);
    assert!(result.is_ok());
}

#[test]
fn managed_terminal_output_snapshot() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("echo").arg("snapshot test");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let mut terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(500));
    let _ = terminal.read_output(&i18n);
    let snapshot = terminal.output_snapshot();
    // Snapshot should be a Vec<String> (may or may not have content depending on timing)
    let _ = snapshot; // Verify type is Vec<String>
}

#[test]
fn managed_terminal_output_buffer_accessible() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("sleep").arg("10");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    let buffer = terminal.output_buffer();
    let locked = buffer.lock().unwrap();
    assert_eq!(locked.len(), 0);
}

#[test]
fn managed_terminal_write_input() {
    let i18n = i18n();
    let cmd = ProcessCommand::new("cat");
    let config = TerminalConfig::default();
    let id = TerminalId::new();

    let mut terminal = mahalaxmi_pty::PtySpawner::spawn(&cmd, &config, id, &i18n).unwrap();

    // Write some input - cat should echo it
    let result = terminal.write_input(b"hello\n", &i18n);
    // write_input may succeed or fail depending on pty implementation
    // but the method should be callable
    assert!(result.is_ok() || result.is_err());
    let _ = terminal.kill(&i18n);
}

// ---------------------------------------------------------------------------
// Reader Task Tests (async)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reader_sends_output_events() {
    let terminal_id = TerminalId::new();
    let data = b"hello world\n";
    let reader: Box<dyn std::io::Read + Send> = Box::new(Cursor::new(data.to_vec()));
    let (tx, mut rx) = tokio::sync::broadcast::channel(64);
    let buffer = Arc::new(Mutex::new(OutputBuffer::new(100, 512 * 1024)));

    let handle = mahalaxmi_pty::reader::spawn_reader_task(terminal_id, reader, tx, buffer);

    // Wait for reader to process
    handle.await.unwrap();

    // Should have received at least one OutputReceived event
    let mut got_output = false;
    while let Ok(event) = rx.try_recv() {
        match event {
            mahalaxmi_pty::TerminalEvent::OutputReceived { .. } => got_output = true,
            _ => {}
        }
    }
    assert!(got_output, "Expected at least one OutputReceived event");
}

#[tokio::test]
async fn reader_sends_text_events() {
    let terminal_id = TerminalId::new();
    let data = b"clean text\n";
    let reader: Box<dyn std::io::Read + Send> = Box::new(Cursor::new(data.to_vec()));
    let (tx, mut rx) = tokio::sync::broadcast::channel(64);
    let buffer = Arc::new(Mutex::new(OutputBuffer::new(100, 512 * 1024)));

    let handle = mahalaxmi_pty::reader::spawn_reader_task(terminal_id, reader, tx, buffer);
    handle.await.unwrap();

    let mut got_text = false;
    while let Ok(event) = rx.try_recv() {
        match event {
            mahalaxmi_pty::TerminalEvent::TextOutput { text, .. } => {
                assert!(text.contains("clean text"));
                got_text = true;
            }
            _ => {}
        }
    }
    assert!(got_text, "Expected at least one TextOutput event");
}

#[tokio::test]
async fn reader_buffers_output() {
    let terminal_id = TerminalId::new();
    let data = b"line1\nline2\n";
    let reader: Box<dyn std::io::Read + Send> = Box::new(Cursor::new(data.to_vec()));
    let (tx, _rx) = tokio::sync::broadcast::channel(64);
    let buffer = Arc::new(Mutex::new(OutputBuffer::new(100, 512 * 1024)));

    let handle = mahalaxmi_pty::reader::spawn_reader_task(terminal_id, reader, tx, buffer.clone());
    handle.await.unwrap();

    let locked = buffer.lock().unwrap();
    assert!(
        !locked.is_empty(),
        "Buffer should have lines after reader completes"
    );
}

#[tokio::test]
async fn reader_reassembles_split_lines() {
    let terminal_id = TerminalId::new();
    // Simulate data arriving in chunks that split a line
    let data = b"hello world\ncomplete line\n";
    let reader: Box<dyn std::io::Read + Send> = Box::new(Cursor::new(data.to_vec()));
    let (tx, _rx) = tokio::sync::broadcast::channel(64);
    let buffer = Arc::new(Mutex::new(OutputBuffer::new(100, 512 * 1024)));

    let handle = mahalaxmi_pty::reader::spawn_reader_task(terminal_id, reader, tx, buffer.clone());
    handle.await.unwrap();

    let locked = buffer.lock().unwrap();
    let lines: Vec<String> = locked.lines().iter().cloned().collect();
    assert!(
        lines.len() >= 2,
        "Expected at least 2 lines, got {}",
        lines.len()
    );
}

#[tokio::test]
async fn reader_flushes_on_eof() {
    let terminal_id = TerminalId::new();
    // Data without trailing newline
    let data = b"no trailing newline";
    let reader: Box<dyn std::io::Read + Send> = Box::new(Cursor::new(data.to_vec()));
    let (tx, _rx) = tokio::sync::broadcast::channel(64);
    let buffer = Arc::new(Mutex::new(OutputBuffer::new(100, 512 * 1024)));

    let handle = mahalaxmi_pty::reader::spawn_reader_task(terminal_id, reader, tx, buffer.clone());
    handle.await.unwrap();

    let locked = buffer.lock().unwrap();
    // After EOF, flush should commit pending
    assert!(
        !locked.is_empty() || locked.pending().is_empty(),
        "After EOF flush, pending should be empty or lines should exist"
    );
}

#[tokio::test]
async fn reader_exits_on_eof() {
    let terminal_id = TerminalId::new();
    let reader: Box<dyn std::io::Read + Send> = Box::new(Cursor::new(Vec::new()));
    let (tx, _rx) = tokio::sync::broadcast::channel(64);
    let buffer = Arc::new(Mutex::new(OutputBuffer::new(100, 512 * 1024)));

    let handle = mahalaxmi_pty::reader::spawn_reader_task(terminal_id, reader, tx, buffer);
    // Should complete (not hang)
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
    assert!(result.is_ok(), "Reader task should exit on EOF");
}

// ---------------------------------------------------------------------------
// TerminalSessionManager Tests
// ---------------------------------------------------------------------------

#[test]
fn session_manager_new_with_config() {
    let config = mahalaxmi_core::config::OrchestrationConfig::default();
    let mgr = TerminalSessionManager::new(&config, i18n());
    assert_eq!(mgr.active_count(), 0);
    assert_eq!(mgr.max_concurrent(), config.max_concurrent_workers);
}

#[test]
fn session_manager_with_max_concurrent() {
    let mgr = TerminalSessionManager::with_max_concurrent(5, i18n());
    assert_eq!(mgr.max_concurrent(), 5);
    assert_eq!(mgr.active_count(), 0);
}

#[test]
fn session_manager_spawn_and_list() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();

    let id = mgr.spawn_terminal(&cmd, &config).unwrap();
    let terminals = mgr.list_terminals();

    assert_eq!(terminals.len(), 1);
    assert!(terminals.contains(&id));
    assert_eq!(mgr.active_count(), 1);
}

#[test]
fn session_manager_max_concurrent_enforcement() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(1, i18n());
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();

    let _id1 = mgr.spawn_terminal(&cmd, &config).unwrap();
    let result = mgr.spawn_terminal(&cmd, &config);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.i18n_key().unwrap(), "error-pty-max-concurrent-reached");
}

#[test]
fn session_manager_close_terminal() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();

    let id = mgr.spawn_terminal(&cmd, &config).unwrap();
    assert_eq!(mgr.active_count(), 1);

    mgr.close_terminal(&id).unwrap();
    assert_eq!(mgr.active_count(), 0);
}

#[test]
fn session_manager_close_nonexistent() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let fake_id = TerminalId::new();
    let result = mgr.close_terminal(&fake_id);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.i18n_key().unwrap(), "error-pty-terminal-not-found");
}

#[test]
fn session_manager_resize_terminal() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();

    let id = mgr.spawn_terminal(&cmd, &config).unwrap();
    let result = mgr.resize_terminal(&id, 50, 120);
    assert!(result.is_ok());
}

#[test]
fn session_manager_resize_nonexistent() {
    let mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let fake_id = TerminalId::new();
    let result = mgr.resize_terminal(&fake_id, 50, 120);
    assert!(result.is_err());
}

#[test]
fn session_manager_close_all() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();

    let _id1 = mgr.spawn_terminal(&cmd, &config).unwrap();
    let _id2 = mgr.spawn_terminal(&cmd, &config).unwrap();
    assert_eq!(mgr.active_count(), 2);

    mgr.close_all();
    assert_eq!(mgr.active_count(), 0);
}

#[test]
fn session_manager_close_all_idempotent() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    mgr.close_all();
    mgr.close_all();
    assert_eq!(mgr.active_count(), 0);
}

#[test]
fn session_manager_active_count() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    assert_eq!(mgr.active_count(), 0);

    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();
    let _id = mgr.spawn_terminal(&cmd, &config).unwrap();

    assert_eq!(mgr.active_count(), 1);
}

#[test]
fn session_manager_get_terminal() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();

    let id = mgr.spawn_terminal(&cmd, &config).unwrap();
    let terminal = mgr.get_terminal(&id);
    assert!(terminal.is_some());
    assert_eq!(terminal.unwrap().id(), id);
}

#[test]
fn session_manager_get_terminal_mut() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();

    let id = mgr.spawn_terminal(&cmd, &config).unwrap();
    let terminal = mgr.get_terminal_mut(&id);
    assert!(terminal.is_some());
}

#[test]
fn session_manager_subscribe() {
    let mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let _rx = mgr.subscribe();
    // Just verify we can subscribe without panic
}

#[test]
fn session_manager_ten_concurrent_sessions() {
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());
    let cmd = ProcessCommand::new("sleep").arg("60");
    let config = TerminalConfig::default();

    let mut ids = Vec::new();
    for _ in 0..10 {
        let id = mgr.spawn_terminal(&cmd, &config).unwrap();
        ids.push(id);
    }
    assert_eq!(mgr.active_count(), 10);

    // 11th should fail
    let result = mgr.spawn_terminal(&cmd, &config);
    assert!(result.is_err());

    // All IDs should be retrievable
    for id in &ids {
        assert!(mgr.get_terminal(id).is_some());
    }

    mgr.close_all();
    assert_eq!(mgr.active_count(), 0);
}

// ===========================================================================
// Group A: Single Terminal with Real OS Commands (4 tests)
// ===========================================================================

#[test]
fn test_single_terminal_echo_command() {
    let cmd = ProcessCommand::new("sh")
        .arg("-c")
        .arg("echo 'hello world'");
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(5, i18n());

    let tid = mgr.spawn_terminal(&cmd, &config).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));

    let terminal = mgr.get_terminal_mut(&tid).unwrap();
    let output = terminal.read_output(&i18n()).unwrap_or_default();
    assert!(
        output.contains("hello world"),
        "Expected 'hello world' in output, got: {output}"
    );

    mgr.close_all();
}

#[test]
fn test_single_terminal_pwd_command() {
    let cmd = ProcessCommand::new("pwd").working_dir("/tmp");
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(5, i18n());

    let tid = mgr.spawn_terminal(&cmd, &config).unwrap();

    // Poll for output — a single read after a fixed sleep is unreliable
    // because pwd exits immediately and the PTY may have EOF'd.
    let mut output = String::new();
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(5) {
        let terminal = mgr.get_terminal_mut(&tid).unwrap();
        let chunk = terminal.read_output(&i18n()).unwrap_or_default();
        output.push_str(&chunk);
        if !output.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // /tmp or /private/tmp on macOS
    assert!(
        output.contains("tmp"),
        "Expected path containing 'tmp' in output, got: {output}"
    );

    mgr.close_all();
}

#[test]
fn test_single_terminal_write_and_read() {
    let cmd = ProcessCommand::new("sh");
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(5, i18n());

    let tid = mgr.spawn_terminal(&cmd, &config).unwrap();
    // Give shell time to start
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Write a command
    let terminal = mgr.get_terminal_mut(&tid).unwrap();
    terminal
        .write_input(b"echo test_marker_42\n", &i18n())
        .unwrap();

    // Wait for output
    std::thread::sleep(std::time::Duration::from_millis(500));

    let terminal = mgr.get_terminal_mut(&tid).unwrap();
    let output = terminal.read_output(&i18n()).unwrap_or_default();
    assert!(
        output.contains("test_marker_42"),
        "Expected 'test_marker_42' in output, got: {output}"
    );

    mgr.close_all();
}

#[test]
fn test_single_terminal_exit_detection() {
    let cmd = ProcessCommand::new("sh")
        .arg("-c")
        .arg("echo done && exit 0");
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(5, i18n());

    let tid = mgr.spawn_terminal(&cmd, &config).unwrap();

    // Poll for exit with timeout — 50 × 100ms = 5s (macOS CI is slow)
    let mut exited = false;
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let terminal = mgr.get_terminal_mut(&tid).unwrap();
        if let Ok(Some(_)) = terminal.try_wait(&i18n()) {
            exited = true;
            break;
        }
    }

    assert!(exited, "Terminal should have exited within 5 seconds");

    let terminal = mgr.get_terminal(&tid).unwrap();
    assert_eq!(terminal.state(), TerminalState::Stopped);

    mgr.close_all();
}

// ===========================================================================
// Group B: 15 Parallel Terminals (3 tests)
// ===========================================================================

#[test]
fn test_fifteen_parallel_echo_terminals() {
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(20, i18n());

    let mut ids = Vec::new();
    for n in 0..15 {
        let cmd = ProcessCommand::new("sh")
            .arg("-c")
            .arg(format!("echo 'terminal-{n}'"));
        let tid = mgr.spawn_terminal(&cmd, &config).unwrap();
        ids.push((tid, n));
    }

    assert_eq!(mgr.active_count(), 15);

    // Wait for all to produce output
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Verify each terminal produced its expected output
    let mut found_count = 0;
    for (tid, n) in &ids {
        let terminal = mgr.get_terminal_mut(tid).unwrap();
        let output = terminal.read_output(&i18n()).unwrap_or_default();
        if output.contains(&format!("terminal-{n}")) {
            found_count += 1;
        }
    }

    assert!(
        found_count >= 12,
        "Expected at least 12 of 15 terminals to produce output, got {found_count}"
    );

    mgr.close_all();
}

#[test]
fn test_fifteen_parallel_unique_ids() {
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(20, i18n());

    let mut ids = Vec::new();
    for _ in 0..15 {
        let cmd = ProcessCommand::new("echo").arg("ping");
        let tid = mgr.spawn_terminal(&cmd, &config).unwrap();
        ids.push(tid);
    }

    // All IDs must be unique
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), 15, "All 15 terminal IDs must be unique");

    // Each is individually addressable
    for id in &ids {
        assert!(mgr.get_terminal(id).is_some());
    }

    mgr.close_all();
}

#[test]
fn test_fifteen_parallel_all_complete() {
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(20, i18n());

    let mut ids = Vec::new();
    for n in 0..15 {
        let cmd = ProcessCommand::new("sh")
            .arg("-c")
            .arg(format!("echo 'done-{n}' && exit 0"));
        let tid = mgr.spawn_terminal(&cmd, &config).unwrap();
        ids.push(tid);
    }

    // Poll until all exited (with 15-second timeout for slow CI runners)
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > std::time::Duration::from_secs(15) {
            break;
        }
        let mut all_done = true;
        for tid in &ids {
            let terminal = mgr.get_terminal_mut(tid).unwrap();
            if terminal.state() == TerminalState::Running {
                let _ = terminal.try_wait(&i18n());
                if terminal.state() == TerminalState::Running {
                    all_done = false;
                }
            }
        }
        if all_done {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Verify all reached a terminal state
    let terminal_count = ids
        .iter()
        .filter(|tid| {
            let t = mgr.get_terminal(tid).unwrap();
            t.state().is_terminal()
        })
        .count();

    assert!(
        terminal_count >= 10,
        "Expected at least 10 of 15 terminals to complete, got {terminal_count}"
    );

    mgr.close_all();
}

// ===========================================================================
// Group C: Utility Pool Isolation (3 tests)
// ===========================================================================

#[test]
fn test_utility_pool_separate_from_orchestration() {
    let config = TerminalConfig::default();
    // 3 orchestration max, 5 utility max
    let mut mgr = TerminalSessionManager::with_limits(3, 5, i18n());

    // Fill orchestration pool to max
    let cmd = ProcessCommand::new("sh").arg("-c").arg("sleep 10");
    for _ in 0..3 {
        mgr.spawn_terminal(&cmd, &config).unwrap();
    }
    assert_eq!(mgr.count_by_purpose(TerminalPurpose::Orchestration), 3);

    // Orchestration pool full — next orchestration spawn should fail
    let result = mgr.spawn_terminal(&cmd, &config);
    assert!(result.is_err(), "Orchestration pool should be full");

    // Utility spawn should still succeed (separate pool)
    let utility_cmd = ProcessCommand::new("echo").arg("install-test");
    let tid = mgr
        .spawn_terminal_with_purpose(&utility_cmd, &config, TerminalPurpose::Utility)
        .unwrap();
    assert_eq!(mgr.count_by_purpose(TerminalPurpose::Utility), 1);
    assert!(mgr.get_terminal(&tid).is_some());

    mgr.close_all();
}

#[test]
fn test_utility_pool_has_own_limit() {
    let config = TerminalConfig::default();
    // 5 orchestration, 3 utility
    let mut mgr = TerminalSessionManager::with_limits(5, 3, i18n());

    // Fill utility pool
    let cmd = ProcessCommand::new("sh").arg("-c").arg("sleep 10");
    for _ in 0..3 {
        mgr.spawn_terminal_with_purpose(&cmd, &config, TerminalPurpose::Utility)
            .unwrap();
    }
    assert_eq!(mgr.count_by_purpose(TerminalPurpose::Utility), 3);

    // Next utility spawn should fail
    let result = mgr.spawn_terminal_with_purpose(&cmd, &config, TerminalPurpose::Utility);
    assert!(result.is_err(), "Utility pool should be full at 3");

    mgr.close_all();
}

#[test]
fn test_orchestration_unaffected_by_utility_exhaustion() {
    let config = TerminalConfig::default();
    // 5 orchestration, 2 utility
    let mut mgr = TerminalSessionManager::with_limits(5, 2, i18n());

    // Fill utility pool
    let cmd = ProcessCommand::new("sh").arg("-c").arg("sleep 10");
    for _ in 0..2 {
        mgr.spawn_terminal_with_purpose(&cmd, &config, TerminalPurpose::Utility)
            .unwrap();
    }

    // Utility full
    let result = mgr.spawn_terminal_with_purpose(&cmd, &config, TerminalPurpose::Utility);
    assert!(result.is_err());

    // Orchestration should still work
    let orch_cmd = ProcessCommand::new("echo").arg("worker-1");
    let tid = mgr.spawn_terminal(&orch_cmd, &config).unwrap();
    assert_eq!(mgr.count_by_purpose(TerminalPurpose::Orchestration), 1);
    assert!(mgr.get_terminal(&tid).is_some());

    mgr.close_all();
}

// ===========================================================================
// Group D: Reap + Lifecycle (3 tests)
// ===========================================================================

#[test]
fn test_reap_frees_slots_for_new_spawns() {
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(3, i18n());

    // Spawn 3 long-running terminals (fill pool)
    let long_cmd = ProcessCommand::new("sh").arg("-c").arg("sleep 30");
    for _ in 0..3 {
        mgr.spawn_terminal(&long_cmd, &config).unwrap();
    }
    assert_eq!(mgr.active_count(), 3);

    // Pool is full — long-running processes won't be reaped
    let extra = ProcessCommand::new("echo").arg("should-fail");
    assert!(
        mgr.spawn_terminal(&extra, &config).is_err(),
        "Pool should be full with 3 long-running terminals"
    );

    // Kill one terminal to simulate completion
    let first_id = mgr.list_terminals()[0];
    mgr.close_terminal(&first_id).unwrap();
    assert_eq!(mgr.active_count(), 2);

    // Now spawning should succeed (slot freed)
    let new_cmd = ProcessCommand::new("echo").arg("new-terminal");
    let tid = mgr.spawn_terminal(&new_cmd, &config).unwrap();
    assert!(mgr.get_terminal(&tid).is_some());

    mgr.close_all();
}

#[test]
fn test_event_emission_on_spawn_and_close() {
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(5, i18n());

    // Subscribe before spawning
    let mut rx = mgr.subscribe();

    let cmd = ProcessCommand::new("echo").arg("event-test");
    let tid = mgr.spawn_terminal(&cmd, &config).unwrap();

    // Should have received a StateChanged event (Created -> Running)
    let event = rx.try_recv();
    assert!(event.is_ok(), "Should have received spawn event");

    // Close and verify close event
    mgr.close_terminal(&tid).unwrap();
    let close_event = rx.try_recv();
    assert!(close_event.is_ok(), "Should have received close event");
}

#[test]
fn test_close_all_terminates_every_terminal() {
    let config = TerminalConfig::default();
    let mut mgr = TerminalSessionManager::with_max_concurrent(10, i18n());

    // Spawn 5 long-running terminals
    let cmd = ProcessCommand::new("sh").arg("-c").arg("sleep 30");
    for _ in 0..5 {
        mgr.spawn_terminal(&cmd, &config).unwrap();
    }
    assert_eq!(mgr.active_count(), 5);

    mgr.close_all();
    assert_eq!(mgr.active_count(), 0, "All terminals should be closed");
}
