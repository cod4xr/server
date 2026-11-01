// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline constructors, opaque output, structured object usage and private failures.
#![allow(
    clippy::expect_used,
    reason = "Tests execute bounded real offline constructor processes."
)]
mod content_support;

use std::process::Command;

use content_support::FixtureFile;

#[test]
fn offline_struct_constructors_return_opaque_kinds_and_report_numeric_usage_without_payloads() {
    for (source, expected, count, kind) in [
        (
            br"main(){return spawnstruct();}".as_slice(),
            b"object\n".as_slice(),
            1,
            "object",
        ),
        (
            br"main(){value=spawnstruct();return value.size+isdefined(value);}",
            b"integer 2\n",
            1,
            "integer",
        ),
        (
            br#"main(){return spawnstruct("PrivateIgnoredCtorArg");}"#,
            b"object\n",
            1,
            "object",
        ),
        (
            br"main(){return spawnstruct();}spawnstruct(){return 7;}",
            b"integer 7\n",
            0,
            "integer",
        ),
        (
            br"main(){return spawnstruct(spawnstruct(),spawnstruct());}",
            b"object\n",
            3,
            "object",
        ),
    ] {
        let file = FixtureFile::from_bytes(source);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Offline constructor process");
        assert!(output.status.success());
        assert_eq!(output.stdout, expected);
        let logs = String::from_utf8(output.stderr).expect("Structured logs");
        assert!(!logs.contains("PrivateIgnoredCtorArg"));
        assert!(!logs.contains("ObjectId"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        let records: Vec<serde_json::Value> = logs
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSON event"))
            .collect();
        let fields = &records
            .iter()
            .find(|record| record["fields"]["event"] == "script.executed")
            .expect("Completion event")["fields"];
        assert_eq!(fields["result_kind"], kind);
        assert_eq!(fields["live_objects"], count);
        assert_eq!(fields["object_slots"], count);
        assert_eq!(fields["live_fields"], 0);
        assert_eq!(fields["object_collections"], 0);
        assert_eq!(
            fields["object_metadata_bytes"].as_u64().expect("Metadata") == 0,
            count == 0
        );
    }
}

#[test]
fn offline_constructor_argument_errors_keep_empty_output_and_do_not_expose_private_source() {
    let file = FixtureFile::from_bytes(br#"main(){return spawnstruct("PrivateCtorArg",bad());}bad(){return "PrivateCtorFailure"/1;}"#);
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--color", "never", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Constructor argument failure");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).expect("Readable logs");
    assert!(logs.contains("UnmatchingTypes"));
    assert!(!logs.contains("PrivateCtor"));
    assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
    assert!(!logs.contains('\u{1b}'));
}
