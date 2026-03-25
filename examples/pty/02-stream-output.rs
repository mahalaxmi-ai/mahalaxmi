// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Streaming terminal events
//
// Demonstrates the TerminalSessionManager event broadcast pattern.
// A subscriber receives TerminalEvents asynchronously as the session
// produces output. This is how the Mahalaxmi dashboard subscribes to
// worker terminal output in real time.

use mahalaxmi_core::{
    config::MahalaxmiConfig,
    i18n::{locale::SupportedLocale, I18nService},
};
use mahalaxmi_pty::{TerminalEvent, TerminalSessionManager};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = MahalaxmiConfig::default();

    // Create a session manager from the orchestration config.
    // It owns the broadcast channel for TerminalEvents.
    // I18nService is passed by value and stored inside the manager.
    let manager = TerminalSessionManager::new(&config.orchestration, i18n);

    // Subscribe before spawning so we don't miss early events.
    // Any number of subscribers can call subscribe() independently.
    let mut event_rx = manager.subscribe();

    // In a real scenario you would call manager.spawn_terminal() here.
    // For this example we demonstrate the subscriber pattern with a timeout.
    println!("Subscribed to terminal events.");
    println!("(In production, spawn a terminal and events flow through this channel.)");

    // Show how events would be handled in an async loop.
    tokio::select! {
        event = event_rx.recv() => {
            match event {
                Ok(TerminalEvent::TextOutput { terminal_id, text }) => {
                    println!("[{:?}] text output: {}", terminal_id, text.trim());
                }
                Ok(TerminalEvent::OutputReceived { terminal_id, data }) => {
                    println!("[{:?}] raw bytes: {} bytes", terminal_id, data.len());
                }
                Ok(TerminalEvent::ProcessExited { terminal_id, exit_code }) => {
                    println!("[{:?}] process exited (code: {})", terminal_id, exit_code);
                }
                Ok(TerminalEvent::StateChanged { terminal_id, new_state, .. }) => {
                    println!("[{:?}] state changed to {:?}", terminal_id, new_state);
                }
                Err(_) => {
                    println!("Channel closed — no active sessions.");
                }
            }
        }
        _ = sleep(Duration::from_millis(100)) => {
            println!("No events within 100ms — channel is live but idle.");
        }
    }
}
