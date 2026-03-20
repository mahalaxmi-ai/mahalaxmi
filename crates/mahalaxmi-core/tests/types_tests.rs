use std::collections::HashMap;

use mahalaxmi_core::types::{
    ExecutionMode, ProcessCommand, ProviderId, SessionId, TerminalConfig, TerminalId, TerminalRole,
    TerminalState,
};

// ---------------------------------------------------------------------------
// TerminalId Tests
// ---------------------------------------------------------------------------

#[test]
fn terminal_id_generates_unique_ids() {
    let id1 = TerminalId::new();
    let id2 = TerminalId::new();
    assert_ne!(id1, id2);
}

#[test]
fn terminal_id_default_generates_unique_ids() {
    let id1 = TerminalId::default();
    let id2 = TerminalId::default();
    assert_ne!(id1, id2);
}

#[test]
fn terminal_id_from_uuid_roundtrip() {
    let uuid = uuid::Uuid::new_v4();
    let id = TerminalId::from_uuid(uuid);
    assert_eq!(*id.as_uuid(), uuid);
}

#[test]
fn terminal_id_display_format() {
    let id = TerminalId::new();
    let s = id.to_string();
    assert_eq!(s.len(), 36, "UUID display should be 36 chars: {}", s);
    assert_eq!(s.matches('-').count(), 4, "UUID should have 4 hyphens");
}

#[test]
fn terminal_id_serde_roundtrip() {
    let id = TerminalId::new();
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: TerminalId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

#[test]
fn terminal_id_hash_consistency() {
    use std::collections::HashSet;
    let id = TerminalId::new();
    let mut set = HashSet::new();
    set.insert(id);
    assert!(set.contains(&id));

    let mut map = HashMap::new();
    map.insert(id, "value");
    assert_eq!(map.get(&id), Some(&"value"));
}

// ---------------------------------------------------------------------------
// ProviderId Tests
// ---------------------------------------------------------------------------

#[test]
fn provider_id_from_string() {
    let id = ProviderId::new("claude");
    assert_eq!(id.as_str(), "claude");
}

#[test]
fn provider_id_serde_roundtrip() {
    let id = ProviderId::new("openai-gpt4");
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: ProviderId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

#[test]
fn provider_id_equality() {
    let id1 = ProviderId::new("claude");
    let id2 = ProviderId::new("claude");
    let id3 = ProviderId::new("openai");
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn provider_id_hash_as_map_key() {
    let id = ProviderId::new("bedrock");
    let mut map = HashMap::new();
    map.insert(id.clone(), 42);
    assert_eq!(map.get(&id), Some(&42));
}

// ---------------------------------------------------------------------------
// SessionId Tests
// ---------------------------------------------------------------------------

#[test]
fn session_id_generates_unique_ids() {
    let id1 = SessionId::new();
    let id2 = SessionId::new();
    assert_ne!(id1, id2);
}

#[test]
fn session_id_default_generates_unique_ids() {
    let id1 = SessionId::default();
    let id2 = SessionId::default();
    assert_ne!(id1, id2);
}

#[test]
fn session_id_from_uuid_roundtrip() {
    let uuid = uuid::Uuid::new_v4();
    let id = SessionId::from_uuid(uuid);
    assert_eq!(*id.as_uuid(), uuid);
}

#[test]
fn session_id_serde_roundtrip() {
    let id = SessionId::new();
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: SessionId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

// ---------------------------------------------------------------------------
// TerminalRole Tests
// ---------------------------------------------------------------------------

#[test]
fn terminal_role_serde_roundtrip() {
    for role in [TerminalRole::Manager, TerminalRole::Worker] {
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: TerminalRole = serde_json::from_str(&json).unwrap();
        assert_eq!(role, deserialized);
    }
}

// ---------------------------------------------------------------------------
// TerminalState Tests
// ---------------------------------------------------------------------------

#[test]
fn terminal_state_is_active_for_active_states() {
    assert!(TerminalState::Created.is_active());
    assert!(TerminalState::Starting.is_active());
    assert!(TerminalState::Running.is_active());
    assert!(TerminalState::Paused.is_active());
}

#[test]
fn terminal_state_is_active_false_for_terminal_states() {
    assert!(!TerminalState::Stopping.is_active());
    assert!(!TerminalState::Stopped.is_active());
    assert!(!TerminalState::Failed.is_active());
}

#[test]
fn terminal_state_is_terminal_for_final_states() {
    assert!(TerminalState::Stopped.is_terminal());
    assert!(TerminalState::Failed.is_terminal());
}

#[test]
fn terminal_state_is_terminal_false_for_active_states() {
    assert!(!TerminalState::Created.is_terminal());
    assert!(!TerminalState::Starting.is_terminal());
    assert!(!TerminalState::Running.is_terminal());
    assert!(!TerminalState::Paused.is_terminal());
    assert!(!TerminalState::Stopping.is_terminal());
}

#[test]
fn terminal_state_serde_roundtrip_all_variants() {
    let states = [
        TerminalState::Created,
        TerminalState::Starting,
        TerminalState::Running,
        TerminalState::Paused,
        TerminalState::Stopping,
        TerminalState::Stopped,
        TerminalState::Failed,
    ];
    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: TerminalState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized, "Failed roundtrip for {:?}", state);
    }
}

// ---------------------------------------------------------------------------
// ExecutionMode Tests
// ---------------------------------------------------------------------------

#[test]
fn execution_mode_default_is_interactive() {
    assert_eq!(ExecutionMode::default(), ExecutionMode::Interactive);
}

#[test]
fn execution_mode_serde_roundtrip() {
    for mode in [ExecutionMode::Interactive, ExecutionMode::Headless] {
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: ExecutionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, deserialized);
    }
}

// ---------------------------------------------------------------------------
// ProcessCommand Tests
// ---------------------------------------------------------------------------

#[test]
fn process_command_new_sets_program() {
    let cmd = ProcessCommand::new("claude");
    assert_eq!(cmd.program, "claude");
}

#[test]
fn process_command_new_has_empty_defaults() {
    let cmd = ProcessCommand::new("claude");
    assert!(cmd.args.is_empty());
    assert!(cmd.env.is_empty());
    assert!(cmd.working_dir.is_none());
}

#[test]
fn process_command_single_arg() {
    let cmd = ProcessCommand::new("echo").arg("hello");
    assert_eq!(cmd.args, vec!["hello"]);
}

#[test]
fn process_command_chained_args() {
    let cmd = ProcessCommand::new("claude")
        .arg("--print")
        .arg("--dangerously-skip-permissions")
        .arg("build the project");
    assert_eq!(cmd.args.len(), 3);
    assert_eq!(cmd.args[0], "--print");
    assert_eq!(cmd.args[1], "--dangerously-skip-permissions");
    assert_eq!(cmd.args[2], "build the project");
}

#[test]
fn process_command_env_vars() {
    let cmd = ProcessCommand::new("claude")
        .env_var("ANTHROPIC_API_KEY", "sk-test-123")
        .env_var("DEBUG", "true");
    assert_eq!(
        cmd.env.get("ANTHROPIC_API_KEY"),
        Some(&"sk-test-123".to_string())
    );
    assert_eq!(cmd.env.get("DEBUG"), Some(&"true".to_string()));
}

#[test]
fn process_command_working_dir() {
    let cmd = ProcessCommand::new("claude").working_dir("/home/user/project");
    assert_eq!(
        cmd.working_dir,
        Some(std::path::PathBuf::from("/home/user/project"))
    );
}

#[test]
fn process_command_serde_roundtrip() {
    let cmd = ProcessCommand::new("claude")
        .arg("--print")
        .arg("hello world")
        .env_var("KEY", "value")
        .working_dir("/tmp");
    let json = serde_json::to_string(&cmd).unwrap();
    let deserialized: ProcessCommand = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.program, "claude");
    assert_eq!(deserialized.args, vec!["--print", "hello world"]);
    assert_eq!(deserialized.env.get("KEY"), Some(&"value".to_string()));
    assert_eq!(
        deserialized.working_dir,
        Some(std::path::PathBuf::from("/tmp"))
    );
}

// ---------------------------------------------------------------------------
// TerminalConfig Tests
// ---------------------------------------------------------------------------

#[test]
fn terminal_config_default_values() {
    let config = TerminalConfig::default();
    assert_eq!(config.rows, 24);
    assert_eq!(config.cols, 80);
    assert_eq!(config.scrollback_lines, 10_000);
    assert!(!config.enable_logging);
}

#[test]
fn terminal_config_custom_values() {
    let config = TerminalConfig {
        rows: 50,
        cols: 120,
        scrollback_lines: 5_000,
        enable_logging: true,
        verbose_logging: false,
        raw_replay_capacity_bytes: 512 * 1024,
    };
    assert_eq!(config.rows, 50);
    assert_eq!(config.cols, 120);
    assert_eq!(config.scrollback_lines, 5_000);
    assert!(config.enable_logging);
}

#[test]
fn terminal_config_serde_roundtrip() {
    let config = TerminalConfig {
        rows: 40,
        cols: 132,
        scrollback_lines: 20_000,
        enable_logging: true,
        verbose_logging: true,
        raw_replay_capacity_bytes: 512 * 1024,
    };
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: TerminalConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.rows, config.rows);
    assert_eq!(deserialized.cols, config.cols);
    assert_eq!(deserialized.scrollback_lines, config.scrollback_lines);
    assert_eq!(deserialized.enable_logging, config.enable_logging);
}
