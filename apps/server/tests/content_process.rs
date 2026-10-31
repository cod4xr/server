// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline content verification, exit status, and log disclosure regression tests.
#![allow(
    clippy::expect_used,
    reason = "Tests own exact synthetic files and child processes."
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
        static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "cod4x-secret-content-{}-{}.iwd",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        let (_, hex) = include_str!("../../../crates/content/tests/fixtures/archives.txt")
            .lines()
            .filter_map(|line| line.split_once(' '))
            .find(|(name, _)| *name == label)
            .expect("Independent fixture");
        let bytes: Vec<u8> = hex
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
        file.write_all(&bytes).expect("Synthetic fixture");
        Self(path)
    }
}

impl Drop for FixtureFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
fn offline_verification_emits_counts_without_paths_names_or_payloads() {
    let file = FixtureFile::new("single");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--verify-iwd"])
        .arg(&file.0)
        .output()
        .expect("Verifier");
    assert!(output.status.success());
    let logs = String::from_utf8(output.stderr).expect("UTF-8 logs");
    assert!(!logs.to_lowercase().contains("secret"));
    let records: Vec<Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).expect("Structured JSON"))
        .collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["fields"]["event"], "content.started");
    assert_eq!(records[1]["fields"]["event"], "content.verified");
    assert_eq!(records[1]["fields"]["entries"], 1);
    assert_eq!(records[1]["fields"]["files"], 1);
}

#[test]
fn corrupt_hidden_content_causes_a_sanitized_failure_exit() {
    for label in ["bad_crc", "hidden_bad_crc", "bad_local_name", "traversal"] {
        let file = FixtureFile::new(label);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--verify-iwd"])
            .arg(&file.0)
            .output()
            .expect("Verifier");
        assert!(!output.status.success());
        let logs = String::from_utf8(output.stderr).expect("UTF-8 logs");
        assert!(!logs.to_lowercase().contains("secret"));
        assert!(logs.contains("content.failed"));
        assert!(!logs.contains("content.verified"));
        assert!(!logs.contains("server.started"));
    }
}

#[test]
fn incompatible_modes_missing_paths_and_query_only_settings_are_rejected() {
    for arguments in [
        vec!["--verify-iwd"],
        vec!["--verify-iwd", "missing", "--query-only"],
        vec!["--verify-iwd", "missing", "--bind", "127.0.0.1:0"],
        vec!["--verify-iwd", "missing", "--run-for-ms", "1"],
        vec!["--verify-iwd", "missing", "--max-datagrams", "1"],
        vec!["--verify-iwd", "missing", "--verify-iwd", "again"],
        vec!["--verify-iwd", "missing"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(arguments)
            .output()
            .expect("Invalid verifier arguments");
        assert!(!output.status.success());
    }
}
