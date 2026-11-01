// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real offline quoted-string execution, escaped output and diagnostic privacy.
#![allow(
    clippy::expect_used,
    reason = "Tests execute real offline source strings through owned temporary fixtures."
)]
mod content_support;

use std::process::Command;

use content_support::FixtureFile;
use serde_json::Value;

#[test]
fn offline_source_string_results_escape_bytes_and_keep_payloads_out_of_human_and_json_logs() {
    let mut text = br#"main(){value=wrap("PrivateValue\n"#.to_vec();
    text.push(0xff);
    text.extend_from_slice(
        br#"");if(isstring(value)){return value;}return undefined;}wrap(x){return "["+x+"]";}"#,
    );
    let file = FixtureFile::from_bytes(&text);
    for format in ["--json-logs", "--human-logs"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args([format, "--color", "never", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("String executor");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"string b\"[PrivateValue\\n\\xff]\"\n");
        let logs = String::from_utf8(output.stderr).expect("Sanitized logs");
        assert!(!logs.contains("PrivateValue"));
        assert!(!logs.contains("wrap"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        assert!(!logs.contains("server.started"));
        if format == "--json-logs" {
            let records: Vec<Value> = logs
                .lines()
                .map(|line| serde_json::from_str(line).expect("JSON"))
                .collect();
            assert_eq!(records.len(), 3);
            assert_eq!(records[1]["fields"]["string_literals"], 3);
            let fields = &records[2]["fields"];
            assert_eq!(fields["result_kind"], "string");
            assert_eq!(fields["live_strings"], 5);
            assert!(fields["heap_steps"].as_u64().expect("Heap work") > 0);
            assert!(
                fields["heap_scratch_bytes"]
                    .as_u64()
                    .expect("Cache metadata")
                    > 0
            );
            assert!(fields.get("value").is_none());
        } else {
            assert!(logs.contains("INFO"));
            assert!(!logs.contains('\u{1b}'));
        }
    }
}

#[test]
fn offline_source_string_failures_keep_payloads_private_and_retain_compiler_runtime_locations() {
    for (text, phase, cause) in [
        (
            br#"main(){return &"PrivateLocalized".size;}"#.as_slice(),
            "execute",
            "ExpectedSized",
        ),
        (
            br#"main(){return "PrivatePayload"+1;}"#.as_slice(),
            "execute",
            "UnmatchingTypes",
        ),
    ] {
        let file = FixtureFile::from_bytes(text);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Failing string executor");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Sanitized logs");
        assert!(!logs.contains("Private"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        let records: Vec<Value> = logs
            .lines()
            .filter(|line| !line.starts_with("cod4x-server: "))
            .map(|line| serde_json::from_str(line).expect("JSON"))
            .collect();
        let failure = &records.last().expect("Failure")["fields"];
        assert_eq!(failure["event"], "script.failed");
        assert_eq!(failure["phase"], phase);
        assert_eq!(failure["line"], 1);
        assert!(
            failure["cause"]
                .as_str()
                .expect("Fixed category")
                .contains(cause)
        );
        assert!(failure["offset"].as_u64().expect("Offset") < text.len() as u64);
        if phase == "execute" {
            assert!(failure["heap_steps"].as_u64().expect("Charged work") > 0);
        }
    }
}

#[test]
fn offline_large_string_output_preserves_the_complete_admitted_payload() {
    let mut text = b"main(){return \"".to_vec();
    text.extend(std::iter::repeat_n(b'x', 100_000));
    text.extend_from_slice(b"\";}");
    let file = FixtureFile::from_bytes(&text);
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Large string executor");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), 100_011);
    assert!(output.stdout.starts_with(b"string b\""));
    assert!(output.stdout[9..100_009].iter().all(|byte| *byte == b'x'));
    assert!(output.stdout.ends_with(b"\"\n"));
    let logs = String::from_utf8(output.stderr).expect("Counts only");
    assert!(!logs.contains("xxxxxxxx"));
    assert!(logs.contains("string_payload_bytes"));
}
