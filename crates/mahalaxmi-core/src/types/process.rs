use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Command specification for spawning a process in a PTY or pipe.
///
/// This is the bridge between the AI provider layer (which knows what command to run)
/// and the execution engine (which knows how to spawn and manage the process).
/// Providers build a `ProcessCommand`; the spawn logic consumes it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCommand {
    /// The program to execute (e.g., "claude", "openai", "bash").
    pub program: String,
    /// Command-line arguments.
    pub args: Vec<String>,
    /// Additional environment variables to set for the process.
    pub env: HashMap<String, String>,
    /// Working directory for the process. If None, inherits from parent.
    pub working_dir: Option<PathBuf>,
    /// Data to pipe to the process's stdin, then close the handle.
    ///
    /// Used by providers that accept prompts via stdin (e.g., `claude --print`
    /// reads from stdin when it's not a TTY). The spawn logic writes this data
    /// to stdin and then closes the handle to signal EOF.
    ///
    /// If None, stdin is set to null (`/dev/null`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdin_data: Option<String>,
}

impl ProcessCommand {
    /// Create a new process command with the given program.
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            stdin_data: None,
        }
    }

    /// Add a single argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments.
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Set an environment variable.
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the working directory.
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set data to pipe to the process's stdin (then close to signal EOF).
    pub fn stdin_data(mut self, data: impl Into<String>) -> Self {
        self.stdin_data = Some(data.into());
        self
    }
}
