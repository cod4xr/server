// SPDX-License-Identifier: AGPL-3.0-or-later
//! Developer-friendly defaults, explicit JSON, and terminal color controls.
#![allow(
    clippy::expect_used,
    reason = "Console test processes have bounded run durations."
)]

use std::process::Command;

use serde_json::Value;

fn run(extra: &[&str]) -> String {
    let result = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--query-only", "--bind", "127.0.0.1:0", "--run-for-ms", "1"])
        .args(extra)
        .output()
        .expect("Bounded development run");
    assert!(result.status.success());
    String::from_utf8(result.stderr).expect("Console uses UTF-8")
}

#[test]
fn default_output_is_pretty_in_development_and_json_in_release() {
    let logs = run(&[]);
    if cfg!(debug_assertions) {
        assert!(
            logs.starts_with('+'),
            "Development timestamps show compact elapsed time"
        );
        assert!(logs.contains("INFO"));
        assert!(logs.contains("development UDP runtime started"));
        assert!(!logs.contains("\"timestamp\":"));
    } else {
        for line in logs.lines() {
            let _: Value = serde_json::from_str(line).expect("Release defaults to JSON");
        }
    }
    assert!(
        !logs.contains('\u{1b}'),
        "Redirected auto-color output must be escape-free"
    );
}

#[test]
fn explicit_pretty_output_supports_forced_and_disabled_colors() {
    let colored = run(&["--human-logs", "--color", "always"]);
    assert!(colored.contains("\u{1b}["));
    assert!(colored.contains("development UDP runtime started"));
    let plain = run(&["--human-logs", "--color", "never"]);
    assert!(plain.starts_with('+'));
    assert!(!plain.contains('\u{1b}'));
}

#[test]
fn explicit_json_stays_parseable_even_when_color_is_forced() {
    let logs = run(&["--json-logs", "--color", "always"]);
    assert!(!logs.contains('\u{1b}'));
    for line in logs.lines() {
        let record: Value = serde_json::from_str(line).expect("JSON must remain machine-readable");
        assert!(record["fields"]["event"].is_string());
    }
}
