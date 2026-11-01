// SPDX-License-Identifier: AGPL-3.0-or-later
//! Package mode remains explicit, offline and independent of script-derived paths.
#![allow(
    clippy::expect_used,
    reason = "Tests supply invalid CLI combinations and owned regular-file snapshots."
)]
mod content_support;

use std::process::Command;

use content_support::FixtureFile;

#[test]
fn package_mode_rejects_network_options_conflicting_modes_and_missing_inputs_before_reading() {
    for options in [
        ["--execute-gsc-package"].as_slice(),
        &["--execute-gsc-package", "PrivateRoot"],
        &["--execute-gsc-package", "PrivateRoot", "--gsc-module"],
        &[
            "--execute-gsc-package",
            "PrivateRoot",
            "--gsc-module",
            "PrivateRoot",
        ],
        &["--query-only", "--gsc-module", "PrivateRoot", "PrivatePath"],
        &[
            "--execute-gsc",
            "PrivatePath",
            "--gsc-module",
            "PrivateRoot",
            "PrivatePath",
        ],
        &[
            "--execute-gsc-package",
            "",
            "--gsc-module",
            "PrivateRoot",
            "PrivatePath",
        ],
        &[
            "--execute-gsc-package",
            "PrivateRoot",
            "--gsc-module",
            "PrivatéRoot",
            "PrivatePath",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(options)
            .output()
            .expect("Invalid CLI");
        assert!(!output.status.success(), "{options:?}");
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Error");
        assert!(!logs.contains("Private"));
        assert!(!logs.contains("script.started"));
    }
    for extra in [
        ["--bind", "127.0.0.1:0"].as_slice(),
        &["--run-for-ms", "1"],
        &["--max-datagrams", "1"],
        &["--query-only"],
        &["--handshake-probe"],
        &["--inspect-gsc", "PrivatePath"],
        &["--gsc-developer", "invalid"],
        &["--execute-gsc-package", "PrivateRoot"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args([
                "--execute-gsc-package",
                "PrivateRoot",
                "--gsc-module",
                "PrivateRoot",
                "PrivatePath",
            ])
            .args(extra)
            .output()
            .expect("Offline combination");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Error");
        assert!(!logs.contains("Private"));
        assert!(!logs.contains("script.started"));
    }
}

#[test]
fn explicitly_supplied_non_regular_or_unreadable_files_fail_privately_without_source_discovery() {
    let file = FixtureFile::from_bytes(b"main(){return 1;}");
    for path in [file.0.join("PrivateChild"), std::env::temp_dir()] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args([
                "--json-logs",
                "--execute-gsc-package",
                "PrivateRoot",
                "--gsc-module",
                "PrivateRoot",
            ])
            .arg(&path)
            .output()
            .expect("Rejected snapshot");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Error");
        assert!(!logs.contains("Private"));
        assert!(!logs.contains(path.to_string_lossy().as_ref()));
        let records: Vec<serde_json::Value> = logs
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        assert_eq!(records[1]["fields"]["phase"], "read");
        assert_eq!(records[1]["fields"]["module"], 0);
    }
}
