// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Connecting to a mahalaxmi-service instance
//
// Demonstrates the HTTP client pattern used by mahalaxmi-cli to interact
// with a running mahalaxmi-service instance. Shows health checking,
// cycle start, status polling, and event streaming.
//
// This is a library-style example showing the REST client pattern directly.
// The actual CLI binary (mahalaxmi-cli) wraps these calls with clap subcommands.
//
// Prerequisites: mahalaxmi-service must be running on localhost:17421.
// If it is not running, all requests will fail with a connection refused error
// and the example will exit gracefully.

use serde_json::Value;

const BASE_URL: &str = "http://127.0.0.1:17421";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    // --- Step 1: Health check ---
    println!("=== Health check ===");
    match client.get(format!("{BASE_URL}/v1/health")).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: Value = resp.json().await?;
            println!("status:      {}", body["status"].as_str().unwrap_or("?"));
            println!("version:     {}", body["version"].as_str().unwrap_or("?"));
            println!("uptime_secs: {}", body["uptime_secs"].as_u64().unwrap_or(0));
        }
        Ok(resp) => {
            println!("Health check failed: {}", resp.status());
            return Ok(());
        }
        Err(e) if e.is_connect() => {
            println!("Cannot connect to {BASE_URL}.");
            println!("Start mahalaxmi-service first, then re-run this example.");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    }

    // --- Step 2: Start a cycle ---
    println!("\n=== Start cycle ===");
    let start_req = serde_json::json!({
        "project_root": "/tmp/example-project",
        "requirements": "Add structured logging to all HTTP handlers.",
        "worker_count": 0      // 0 = auto-scale
    });

    let resp = client
        .post(format!("{BASE_URL}/v1/cycles"))
        .json(&start_req)
        .send()
        .await?;

    if resp.status() != reqwest::StatusCode::CREATED {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        println!("Cycle start failed: {status} — {body}");
        return Ok(());
    }

    let body: Value = resp.json().await?;
    let cycle_id = body["cycle_id"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    println!("Cycle started: {cycle_id}");

    // --- Step 3: Poll status ---
    println!("\n=== Poll status ===");
    let resp = client
        .get(format!("{BASE_URL}/v1/cycles/{cycle_id}"))
        .send()
        .await?;

    if resp.status().is_success() {
        let body: Value = resp.json().await?;
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("Status poll failed: {}", resp.status());
    }

    // --- Step 4: Stop the cycle ---
    println!("\n=== Stop cycle ===");
    let resp = client
        .delete(format!("{BASE_URL}/v1/cycles/{cycle_id}"))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("Cycle {cycle_id} stopped.");
    } else {
        println!("Stop failed: {}", resp.status());
    }

    Ok(())
}
