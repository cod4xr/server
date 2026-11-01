// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real offline field compilation/execution and sanitized operational events.
#![allow(
    clippy::expect_used,
    reason = "Tests execute bounded offline processes with private field names and values."
)]
mod content_support;

use std::process::Command;

use content_support::FixtureFile;

#[test]
fn offline_fields_return_values_and_log_counts_without_private_keys_or_payloads() {
    let source = br#"main(){root=spawnstruct();root.PrivateField="PrivateValue";alias=root;alias.PrivateField=undefined;root.PublicCount=71;return root.PublicCount+isdefined(root.PrivateField);}"#;
    let file = FixtureFile::from_bytes(source);
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Offline fields");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"integer 71\n");
    let logs = String::from_utf8(output.stderr).expect("Structured logs");
    assert!(!logs.contains("Private"));
    assert!(!logs.contains("PublicCount"));
    assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
    let records: Vec<serde_json::Value> = logs
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON event"))
        .collect();
    let compiled = &records
        .iter()
        .find(|record| record["fields"]["event"] == "script.compiled")
        .expect("Compiler event")["fields"];
    assert_eq!(compiled["field_names"], 5);
    assert!(
        compiled["staged_field_bytes"]
            .as_u64()
            .expect("Owned name bytes")
            > 0
    );
    let executed = &records
        .iter()
        .find(|record| record["fields"]["event"] == "script.executed")
        .expect("Execution event")["fields"];
    assert_eq!(executed["live_objects"], 1);
    assert_eq!(executed["live_fields"], 1);
    assert_eq!(executed["object_key_bytes"], 11);
}

#[test]
fn invalid_field_receivers_keep_private_human_logs_and_empty_result_output() {
    let file = FixtureFile::from_bytes(br#"main(){value="PrivatePayload";value.PrivateField=1;}"#);
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--color", "never", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Located field failure");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).expect("Human logs");
    assert!(logs.contains("ExpectedFieldObject"));
    assert!(!logs.contains("Private"));
    assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
    assert!(!logs.contains('\u{1b}'));
}

#[test]
fn offline_compound_and_postfix_field_updates_execute_with_readable_private_logs() {
    let file = FixtureFile::from_bytes(br"main(){o=spawnstruct();o.PrivateField=35;o.PrivateField*=2;o.PrivateField++;return o.PrivateField;}");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--color", "never", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Field updates in real process");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"integer 71\n");
    let logs = String::from_utf8(output.stderr).expect("Readable logs");
    assert!(logs.contains("script.executed"));
    assert!(!logs.contains("PrivateField"));
    assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
    assert!(!logs.contains('\u{1b}'));
}
