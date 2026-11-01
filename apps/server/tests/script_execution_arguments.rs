// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline execution remains an explicit mode with no query/network settings.
#![allow(
    clippy::expect_used,
    reason = "Tests provide synthetic files and deliberately invalid CLI combinations."
)]

mod content_support;

use content_support::FixtureFile;
use std::process::Command;

#[test]
fn execution_rejects_network_options_conflicting_modes_and_missing_paths() {
    let file = FixtureFile::from_bytes(b"main(){return 7;}");
    for options in [
        ["--bind", "127.0.0.1:0"].as_slice(),
        ["--run-for-ms", "1"].as_slice(),
        ["--max-datagrams", "1"].as_slice(),
        ["--query-only"].as_slice(),
        ["--handshake-probe"].as_slice(),
        ["--inspect-gsc", "unused"].as_slice(),
        ["--gsc-developer", "invalid"].as_slice(),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .arg("--execute-gsc")
            .arg(&file.0)
            .args(options)
            .output()
            .expect("Invalid offline combination");
        assert!(!output.status.success(), "{options:?}");
        assert!(output.stdout.is_empty());
        assert!(
            !String::from_utf8(output.stderr)
                .expect("Error")
                .contains("script.started")
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .arg("--execute-gsc")
        .output()
        .expect("Missing path");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .expect("Error")
            .contains("needs a path")
    );
}
