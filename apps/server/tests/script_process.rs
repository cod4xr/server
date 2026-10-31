// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline script diagnostics report locations/counts without source or host paths.
#![allow(
    clippy::expect_used,
    reason = "Tests own synthetic source files and child processes."
)]

mod content_support;

use std::fs::OpenOptions;
use std::process::Command;

use serde_json::Value;

use content_support::FixtureFile;

#[test]
fn offline_structure_reports_counts_as_json_and_readable_development_logs() {
    let bytes = b"// private-secret\nmain() { return private_function(); }\n";
    let file = FixtureFile::from_bytes(bytes);
    for format in ["--json-logs", "--human-logs"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args([format, "--color", "never", "--inspect-gsc"])
            .arg(&file.0)
            .output()
            .expect("Inspector");
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.to_lowercase().contains("secret"));
        assert!(!logs.contains("private_function"));
        assert!(!logs.contains("server.started"));
        assert!(logs.contains("script.syntax_inspected"));
        assert!(logs.contains("compilation and execution remain unavailable"));
        if format == "--json-logs" {
            let records: Vec<Value> = logs
                .lines()
                .map(|row| serde_json::from_str(row).expect("JSON"))
                .collect();
            assert_eq!(records.len(), 2);
            let fields = &records[1]["fields"];
            assert_eq!(fields["profile"], "legacy");
            assert_eq!(fields["source_bytes"], bytes.len());
            assert_eq!(fields["lines"], 3);
            assert_eq!(fields["source_tokens"], 10);
            assert_eq!(fields["functions"], 1);
            assert_eq!(fields["parameters"], 0);
            assert_eq!(fields["includes"], 0);
            assert!(fields["peak_stack_entries"].as_u64().expect("Stack") > 0);
            assert!(fields["peak_metadata_bytes"].as_u64().expect("Metadata") > 0);
        } else {
            assert!(logs.contains("INFO"));
            assert!(!logs.contains('\u{1b}'));
            assert!(!logs.starts_with('{'));
        }
    }
}

#[test]
fn syntax_failure_reports_original_line_and_byte_column_with_no_partial_success() {
    let bytes = b"main() {\n  private_secret = ;\n}";
    let file = FixtureFile::from_bytes(bytes);
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--inspect-gsc"])
        .arg(&file.0)
        .output()
        .expect("Inspector");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).expect("Logs");
    assert!(!logs.to_lowercase().contains("secret"));
    assert!(!logs.contains("script.syntax_inspected"));
    assert!(!logs.contains("server.started"));
    let records: Vec<Value> = logs
        .lines()
        .filter(|row| row.starts_with('{'))
        .map(|row| serde_json::from_str(row).expect("JSON"))
        .collect();
    assert_eq!(records.len(), 2);
    let fields = &records[1]["fields"];
    assert_eq!(fields["event"], "script.failed");
    assert_eq!(fields["phase"], "parse");
    assert_eq!(fields["kind"], "UnexpectedToken");
    assert_eq!(fields["line"], 2);
    let offset = bytes
        .iter()
        .position(|byte| *byte == b';')
        .expect("Offending byte");
    assert_eq!(fields["offset"], offset);
    assert_eq!(
        fields["byte_column"],
        offset
            - bytes
                .iter()
                .position(|byte| *byte == b'\n')
                .expect("Line start")
    );
}

#[test]
fn source_budget_rejects_large_snapshot_before_parsing_or_allocating_its_contents() {
    let file = FixtureFile::from_bytes(b"");
    OpenOptions::new()
        .write(true)
        .open(&file.0)
        .expect("Test-owned file")
        .set_len(64 * 1024 * 1024 + 1)
        .expect("Oversize metadata");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--inspect-gsc"])
        .arg(&file.0)
        .output()
        .expect("Inspector");
    assert!(!output.status.success());
    let logs = String::from_utf8(output.stderr).expect("Logs");
    assert!(logs.contains("file_budget"));
    assert!(!logs.contains("syntax_inspected"));
    assert!(!logs.to_lowercase().contains("secret"));
}

#[test]
fn inspection_requires_a_path_and_rejects_other_modes_or_network_options() {
    for arguments in [
        vec!["--inspect-gsc"],
        vec!["--inspect-gsc", "secret-missing"],
        vec!["--inspect-gsc", "secret-missing", "--query-only"],
        vec![
            "--inspect-gsc",
            "secret-missing",
            "--inspect-rawfiles",
            "other",
        ],
        vec!["--inspect-gsc", "secret-missing", "--bind", "127.0.0.1:0"],
        vec!["--inspect-gsc", "secret-missing", "--run-for-ms", "1"],
        vec!["--inspect-gsc", "secret-missing", "--max-datagrams", "1"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(arguments)
            .output()
            .expect("Invalid mode");
        assert!(!output.status.success());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.contains("secret-missing"));
        assert!(!logs.contains("server.started"));
        assert!(!logs.contains("syntax_inspected"));
    }
}
