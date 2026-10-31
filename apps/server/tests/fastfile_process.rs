// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline envelope inspection cannot report asset-graph or gameplay readiness.
#![allow(
    clippy::expect_used,
    reason = "Tests own synthetic files and child processes."
)]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

struct FixtureFile(PathBuf);

impl FixtureFile {
    fn new(label: &str) -> Self {
        Self::from_dataset(
            label,
            include_str!("../../../crates/content/tests/fixtures/fastfiles.txt"),
        )
    }

    fn zone(label: &str) -> Self {
        Self::from_dataset(
            label,
            include_str!("../../../crates/content/tests/fixtures/zones.txt"),
        )
    }

    fn from_dataset(label: &str, dataset: &str) -> Self {
        static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "cod4x-secret-fastfile-{}-{}.ff",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        let row = dataset
            .lines()
            .map(|line| line.split_ascii_whitespace().collect::<Vec<_>>())
            .find(|row| row[0] == label)
            .expect("Independent fixture");
        let bytes: Vec<u8> = row[1]
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16).expect("Byte")
            })
            .collect();
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("Exclusive test-owned file");
        file.write_all(&bytes).expect("Synthetic envelope");
        Self(path)
    }
}

#[test]
fn zone_inspection_reports_only_root_counts_and_leaves_bodies_unvalidated() {
    let file = FixtureFile::zone("mixed");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--inspect-zone-prefix"])
        .arg(&file.0)
        .output()
        .expect("Inspector");
    assert!(output.status.success());
    let logs = String::from_utf8(output.stderr).expect("Logs");
    assert!(!logs.to_lowercase().contains("secret"));
    let records: Vec<Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON"))
        .collect();
    assert_eq!(records.len(), 2);
    let fields = &records[1]["fields"];
    assert_eq!(fields["event"], "content.prefix_inspected");
    assert_eq!(fields["script_strings"], 5);
    assert_eq!(fields["asset_records"], 3);
    assert_eq!(fields["virtual_bytes"], 56);
    assert_eq!(fields["prefix_bytes"], 70);
    assert_eq!(fields["remaining_bytes"], 11);
    assert!(
        fields["message"]
            .as_str()
            .expect("Message")
            .contains("unvalidated")
    );
    assert!(!logs.contains("server.started"));
}

#[test]
fn corrupt_root_metadata_fails_after_valid_envelope_without_disclosing_input() {
    for name in ["short_table", "missing_strings", "short_block"] {
        let file = FixtureFile::zone(name);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--inspect-zone-prefix"])
            .arg(&file.0)
            .output()
            .expect("Inspector");
        assert!(!output.status.success());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.to_lowercase().contains("secret"));
        assert!(logs.contains("content.failed"));
        assert!(!logs.contains("prefix_inspected"));
    }
}

impl Drop for FixtureFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
fn inspection_reports_envelope_metadata_without_publishing_asset_readiness_or_secrets() {
    let file = FixtureFile::new("small");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--inspect-fastfile"])
        .arg(&file.0)
        .output()
        .expect("Inspector");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).expect("UTF-8 logs");
    assert!(!logs.to_lowercase().contains("secret"));
    let records: Vec<Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON"))
        .collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["fields"]["event"], "content.started");
    assert_eq!(records[0]["fields"]["mode"], "fastfile_inspection");
    assert_eq!(records[1]["fields"]["event"], "content.envelope_verified");
    assert_eq!(records[1]["fields"]["payload_bytes"], 25);
    assert_eq!(records[1]["fields"]["external_bytes"], 1_223_352);
    assert_eq!(records[1]["fields"]["total_block_bytes"], 387);
    assert!(
        records[1]["fields"]["message"]
            .as_str()
            .expect("Message")
            .contains("unvalidated")
    );
    assert!(!logs.contains("server.started"));
}

#[test]
fn invalid_and_signed_envelopes_fail_without_names_or_partial_success() {
    for label in [
        "signed",
        "bad_checksum",
        "trailer",
        "short_payload",
        "huge_declaration",
    ] {
        let file = FixtureFile::new(label);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--inspect-fastfile"])
            .arg(&file.0)
            .output()
            .expect("Inspector");
        assert!(!output.status.success());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.to_lowercase().contains("secret"));
        assert!(logs.contains("content.failed"));
        assert!(!logs.contains("envelope_verified"));
        assert!(!logs.contains("server.started"));
    }
}

#[test]
fn inspection_requires_one_mode_one_path_and_no_network_options() {
    for arguments in [
        vec!["--inspect-fastfile"],
        vec!["--inspect-fastfile", "secret-missing"],
        vec![
            "--inspect-fastfile",
            "secret-missing",
            "--verify-iwd",
            "other",
        ],
        vec!["--inspect-fastfile", "secret-missing", "--handshake-probe"],
        vec!["--inspect-zone-prefix"],
        vec![
            "--inspect-zone-prefix",
            "secret-missing",
            "--inspect-fastfile",
            "other",
        ],
        vec![
            "--inspect-zone-prefix",
            "secret-missing",
            "--bind",
            "127.0.0.1:0",
        ],
        vec!["--inspect-zone-prefix", "secret-missing", "--query-only"],
        vec!["--query-only", "--inspect-fastfile", "secret-missing"],
        vec![
            "--inspect-fastfile",
            "secret-missing",
            "--bind",
            "127.0.0.1:0",
        ],
        vec!["--inspect-fastfile", "secret-missing", "--run-for-ms", "1"],
        vec![
            "--inspect-fastfile",
            "secret-missing",
            "--max-datagrams",
            "1",
        ],
        vec![
            "--inspect-fastfile",
            "secret-missing",
            "--inspect-fastfile",
            "again",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(arguments)
            .output()
            .expect("Invalid arguments");
        assert!(!output.status.success());
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.contains("secret-missing"));
        assert!(!logs.contains("server.started"));
    }
}
