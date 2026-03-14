// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Spawning a PTY process
//
// Demonstrates how to use PtySpawner to launch a command in a pseudo-terminal,
// read its output, and inspect the output buffer. Here we spawn `echo` as a
// simple stand-in for an AI CLI tool.
//
// In production, the `ProcessCommand` is built by `AiProvider::build_command()`.

use mahalaxmi_core::{
    i18n::{locale::SupportedLocale, I18nService},
    types::{ProcessCommand, TerminalConfig, TerminalId},
};
use mahalaxmi_pty::{PtySpawner, VtCleaner};
use std::collections::HashMap;

fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // Build a ProcessCommand for a simple echo invocation.
    // In practice this comes from AiProvider::build_command().
    let command = ProcessCommand {
        program: "echo".to_string(),
        args: vec!["Hello from Mahalaxmi PTY!".to_string()],
        env: HashMap::new(),
        working_dir: None,
        stdin_data: None,
    };

    // TerminalConfig sets the PTY dimensions.
    let config = TerminalConfig {
        rows: 24,
        cols: 220,
        ..Default::default()
    };

    let terminal_id = TerminalId::new();

    println!("Spawning PTY process: {} {:?}", command.program, command.args);

    // PtySpawner::spawn opens the PTY pair and starts the child process.
    match PtySpawner::spawn(&command, &config, terminal_id, &i18n) {
        Ok(mut terminal) => {
            println!("Process spawned successfully (id: {:?})", terminal_id);

            // Give the child a moment to write its output.
            std::thread::sleep(std::time::Duration::from_millis(200));

            // Read available output into the internal buffer.
            let _ = terminal.read_output(&i18n);

            // Retrieve clean text lines from the output snapshot.
            let lines = terminal.output_snapshot();
            let output = lines.join("\n");
            let output = output.trim();

            // VtCleaner can also be used standalone on arbitrary byte slices.
            let raw = b"\x1b[32mGreen text\x1b[0m plain";
            let mut cleaner = VtCleaner::new();
            let stripped = cleaner.clean(raw);

            println!("PTY output (clean): {:?}", output);
            println!("VtCleaner standalone: {:?}", stripped);
        }
        Err(err) => {
            // `echo` may not be available on all platforms in test environments.
            eprintln!("Spawn failed (expected in restricted environments): {err}");
        }
    }
}
