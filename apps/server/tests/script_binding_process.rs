// SPDX-License-Identifier: AGPL-3.0-or-later
//! Local symbol diagnostics, explicit developer mode, and complete source privacy.
#![allow(
    clippy::expect_used,
    reason = "Tests own synthetic script files and child processes."
)]

mod content_support;

use std::process::{Command, Output};

use serde_json::Value;

use content_support::FixtureFile;

fn inspect(file: &FixtureFile, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--inspect-gsc"])
        .arg(&file.0)
        .args(["--gsc-developer", mode])
        .output()
        .expect("Inspector")
}

#[test]
fn explicit_developer_mode_preserves_total_counts_and_selects_active_definitions() {
    let file =
        FixtureFile::from_bytes(b"Private_Main() {} /# Private_Hidden() {} #/ Private_Last() {}");
    for (mode, label, active) in [("disabled", "Disabled", 2), ("enabled", "Enabled", 3)] {
        let output = inspect(&file, mode);
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.to_lowercase().contains("private"));
        assert!(!logs.to_lowercase().contains("secret"));
        assert!(!logs.contains("server.started"));
        let records: Vec<Value> = logs
            .lines()
            .map(|row| serde_json::from_str(row).expect("JSON"))
            .collect();
        assert_eq!(records.len(), 2);
        let fields = &records[1]["fields"];
        assert_eq!(fields["event"], "script.syntax_inspected");
        assert_eq!(fields["developer_scripts"], label);
        assert_eq!(fields["functions"], 3);
        assert_eq!(fields["active_functions"], active);
        assert_eq!(fields["ordinary_functions"], 2);
        assert_eq!(fields["developer_functions"], 1);
        assert!(fields["binding_metadata_bytes"].as_u64().expect("Metadata") > 0);
    }
}

#[test]
fn duplicate_function_failure_reports_both_locations_without_names_or_paths() {
    let file = FixtureFile::from_bytes(b"private_secret() {}\nPrivate_Secret() {}");
    let output = inspect(&file, "enabled");
    assert!(!output.status.success());
    let logs = String::from_utf8(output.stderr).expect("Logs");
    assert!(!logs.to_lowercase().contains("secret"));
    assert!(!logs.contains("syntax_inspected"));
    let records: Vec<Value> = logs
        .lines()
        .filter(|row| row.starts_with('{'))
        .map(|row| serde_json::from_str(row).expect("JSON"))
        .collect();
    assert_eq!(records.len(), 2);
    let fields = &records[1]["fields"];
    assert_eq!(fields["event"], "script.failed");
    assert_eq!(fields["phase"], "binding");
    assert_eq!(fields["kind"], "DuplicateFunction");
    assert_eq!(fields["line"], 2);
    assert_eq!(fields["byte_column"], 1);
    assert_eq!(fields["previous_line"], 1);
    assert_eq!(fields["previous_byte_column"], 1);
    assert_eq!(fields["previous_offset"], 0);
}

#[test]
fn disabled_developer_duplicates_remain_recorded_and_invalid_regions_always_fail() {
    let file = FixtureFile::from_bytes(b"private_secret() {} /# Private_Secret() {} #/");
    assert!(inspect(&file, "disabled").status.success());
    assert!(!inspect(&file, "enabled").status.success());
    for bytes in [
        b"/# /# private_secret() {} #/ #/".as_slice(),
        b"#/ private_secret() {}",
        b"/# private_secret() {}",
        b"/# #using_animtree(\"private-secret\"); #/",
    ] {
        let file = FixtureFile::from_bytes(bytes);
        for mode in ["enabled", "disabled"] {
            let output = inspect(&file, mode);
            assert!(!output.status.success());
            let logs = String::from_utf8(output.stderr).expect("Logs");
            assert!(logs.contains("script.failed"));
            assert!(!logs.contains("syntax_inspected"));
            assert!(!logs.to_lowercase().contains("secret"));
        }
    }
}

#[test]
fn developer_flag_requires_a_supported_value_and_gsc_inspection_mode() {
    for arguments in [
        vec!["--gsc-developer"],
        vec!["--gsc-developer", "private-secret", "--query-only"],
        vec!["--gsc-developer", "enabled", "--query-only"],
        vec![
            "--inspect-gsc",
            "private-secret",
            "--gsc-developer",
            "private-secret",
        ],
        vec![
            "--verify-iwd",
            "private-secret",
            "--gsc-developer",
            "disabled",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(arguments)
            .output()
            .expect("Invalid options");
        assert!(!output.status.success());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.contains("private-secret"));
        assert!(!logs.contains("server.started"));
    }
}
