// SPDX-License-Identifier: AGPL-3.0-or-later
//! Raw localized-value output preserves tags/bytes without leaking data to logs.
#![allow(
    clippy::expect_used,
    reason = "Tests execute real localized source through owned fixtures."
)]
mod content_support;

use std::process::Command;

use content_support::FixtureFile;
use serde_json::Value;

#[test]
fn offline_localized_values_keep_their_category_and_are_escaped_only_on_stdout() {
    let source = br#"main(){value=&"PrivateLocalized\n";if(isstring(value)){return 99;}return copy(value);}copy(x){return x;}"#;
    let file = FixtureFile::from_bytes(source);
    for format in ["--json-logs", "--human-logs"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args([format, "--color", "never", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Localized executor");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            b"localized_string b\"PrivateLocalized\\n\"\n"
        );
        let logs = String::from_utf8(output.stderr).expect("Sanitized logs");
        assert!(!logs.contains("PrivateLocalized"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        if format == "--json-logs" {
            let records: Vec<Value> = logs
                .lines()
                .map(|line| serde_json::from_str(line).expect("JSON"))
                .collect();
            assert_eq!(records.len(), 3);
            assert_eq!(records[1]["fields"]["string_literals"], 1);
            assert_eq!(records[2]["fields"]["result_kind"], "localized_string");
            assert_eq!(records[2]["fields"]["live_strings"], 1);
            assert!(records[2]["fields"].get("value").is_none());
        }
    }
}

#[test]
fn localized_mixed_operations_fail_with_locations_without_partial_output_or_private_data() {
    for (source, kind) in [
        (
            br#"main(){return &"PrivateKey"+"plain";}"#.as_slice(),
            "UnmatchingTypes",
        ),
        (
            br#"main(){if(&"PrivateKey"){return 1;}return 0;}"#,
            "ExpectedNumber",
        ),
    ] {
        let file = FixtureFile::from_bytes(source);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Invalid localized operation");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Sanitized logs");
        assert!(!logs.contains("PrivateKey"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        let records: Vec<Value> = logs
            .lines()
            .filter(|line| !line.starts_with("cod4x-server: "))
            .map(|line| serde_json::from_str(line).expect("JSON"))
            .collect();
        assert_eq!(records.len(), 3);
        let fields = &records[2]["fields"];
        assert_eq!(fields["phase"], "execute");
        assert_eq!(fields["line"], 1);
        assert!(
            fields["cause"]
                .as_str()
                .expect("Typed cause")
                .contains(kind)
        );
        assert!(fields["heap_steps"].as_u64().expect("Failed work") > 0);
    }
}
