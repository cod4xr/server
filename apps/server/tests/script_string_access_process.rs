// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real offline string index/size output, diagnostics and log privacy.
#![allow(
    clippy::expect_used,
    reason = "Tests run actual offline GSC string access processes."
)]
mod content_support;

use std::process::Command;

use content_support::FixtureFile;

#[test]
fn offline_source_string_access_prints_raw_bytes_and_size_without_logging_payloads() {
    for (text, expected) in [
        (
            "main(){return \"éPrivate\"[1];}".as_bytes(),
            b"string b\"\\xa9\"\n".as_slice(),
        ),
        (br#"main(){return "Private".size;}"#, b"integer 7\n"),
    ] {
        let file = FixtureFile::from_bytes(text);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Offline access");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected);
        let logs = String::from_utf8(output.stderr).expect("Private logs");
        assert!(!logs.contains("Private"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        assert!(logs.contains("heap_steps"));
    }
}

#[test]
fn offline_source_string_index_errors_remain_located_and_private() {
    for (text, category) in [
        (
            br#"main(){return "Private"[-1];}"#.as_slice(),
            "StringIndexOutOfRange",
        ),
        (br#"main(){return "Private"[1.0];}"#, "ExpectedStringIndex"),
    ] {
        let file = FixtureFile::from_bytes(text);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Access failure");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Located logs");
        assert!(logs.contains(category));
        assert!(logs.contains("\"phase\":\"execute\""));
        assert!(logs.contains("\"line\":1"));
        assert!(!logs.contains("Private"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
    }
}
