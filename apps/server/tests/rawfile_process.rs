// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline typed content diagnostics never expose filenames or execute scripts.
#![allow(
    clippy::expect_used,
    reason = "Tests own synthetic files and child processes."
)]

mod content_support;

use std::process::Command;

use serde_json::Value;

use content_support::FixtureFile;

#[test]
fn rawfile_inspection_reports_accounting_without_names_buffers_or_paths() {
    let file = FixtureFile::rawfile("aliases");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--inspect-rawfiles"])
        .arg(&file.0)
        .output()
        .expect("Inspector");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).expect("Logs");
    assert!(!logs.to_lowercase().contains("secret"));
    assert!(!logs.contains("synthetic"));
    let records: Vec<Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON"))
        .collect();
    assert_eq!(records.len(), 2);
    let fields = &records[1]["fields"];
    assert_eq!(fields["event"], "content.rawfiles_inspected");
    assert_eq!(fields["root_entries"], 5);
    assert_eq!(fields["present_entries"], 4);
    assert_eq!(fields["inline_buffer_bytes"], 19);
    assert_eq!(fields["temporary_bytes"], 12);
    assert_eq!(fields["virtual_bytes"], 85);
    assert!(!logs.contains("server.started"));
}

#[test]
fn unsupported_and_corrupt_raw_collections_fail_without_partial_success_or_secrets() {
    for label in [
        "unsupported",
        "unknown_alias",
        "bad_terminator",
        "negative_length",
    ] {
        let file = FixtureFile::rawfile(label);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--inspect-rawfiles"])
            .arg(&file.0)
            .output()
            .expect("Inspector");
        assert!(!output.status.success());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.to_lowercase().contains("secret"));
        assert!(!logs.contains("synthetic"));
        assert!(logs.contains("content.failed"));
        assert!(!logs.contains("rawfiles_inspected"));
        assert!(!logs.contains("server.started"));
    }
}

#[test]
fn rawfile_mode_rejects_missing_paths_and_other_modes_or_network_settings() {
    for arguments in [
        vec!["--inspect-rawfiles"],
        vec!["--inspect-rawfiles", "secret-missing"],
        vec!["--inspect-rawfiles", "secret-missing", "--query-only"],
        vec![
            "--inspect-rawfiles",
            "secret-missing",
            "--inspect-zone-prefix",
            "other",
        ],
        vec![
            "--inspect-rawfiles",
            "secret-missing",
            "--bind",
            "127.0.0.1:0",
        ],
        vec!["--inspect-rawfiles", "secret-missing", "--run-for-ms", "1"],
        vec![
            "--inspect-rawfiles",
            "secret-missing",
            "--max-datagrams",
            "1",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(arguments)
            .output()
            .expect("Invalid mode");
        assert!(!output.status.success());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.contains("secret-missing"));
        assert!(!logs.contains("server.started"));
    }
}
