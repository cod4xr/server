// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real execution failures stay located, private, bounded and free of partial results.
#![allow(
    clippy::expect_used,
    reason = "Tests own finite synthetic source and bounded child executions."
)]

mod content_support;

use content_support::FixtureFile;
use serde_json::Value;
use std::process::{Command, Output};

fn failed(bytes: &[u8]) -> (Output, Vec<Value>) {
    let file = FixtureFile::from_bytes(bytes);
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Failed executor");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr.clone()).expect("Logs");
    assert!(!logs.contains("Private"));
    assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
    assert!(!logs.contains("script.executed"));
    assert!(!logs.contains("server.started"));
    assert!(logs.contains("cod4x-server: "));
    let records = logs
        .lines()
        .filter(|line| !line.starts_with("cod4x-server: "))
        .map(|line| serde_json::from_str(line).expect("Structured log"))
        .collect();
    (output, records)
}

#[test]
fn source_compilation_and_runtime_failures_report_original_locations_without_source() {
    let (_, records) = failed(b"main(){\n  return PrivateMissing();\n}");
    assert_eq!(records.len(), 2);
    assert_eq!(records[1]["fields"]["phase"], "compile");
    assert_eq!(records[1]["fields"]["line"], 2);
    assert_eq!(records[1]["fields"]["byte_column"], 10);
    assert_eq!(records[1]["fields"]["declaration"], 0);
    assert_eq!(records[1]["fields"]["cause"], "UnknownFunction");
    let (_, records) = failed(b"main(){\n return 1/0;\n}");
    assert_eq!(records.len(), 3);
    assert_eq!(records[2]["fields"]["phase"], "execute");
    assert_eq!(records[2]["fields"]["line"], 2);
    assert_eq!(records[2]["fields"]["byte_column"], 10);
    assert_eq!(records[2]["fields"]["function"], 0);
    assert_eq!(records[2]["fields"]["steps"], 3);
}

#[test]
fn imports_animation_directives_and_missing_or_disabled_entries_never_fabricate_execution() {
    for (text, phase) in [
        (
            "#include maps\\mp\\PrivateInclude;main(){return 7;}",
            "directives",
        ),
        (
            "#using_animtree(\"PrivateTree\");main(){return 7;}",
            "directives",
        ),
        ("helper(){return 99;}", "entry"),
        ("", "entry"),
        ("/# main(){return 7;} #/", "entry"),
    ] {
        let (_, records) = failed(text.as_bytes());
        assert_eq!(records.len(), 2, "{text}");
        assert_eq!(records[1]["fields"]["phase"], phase);
        assert_eq!(records[1]["fields"]["kind"], "unsupported");
    }
}

#[test]
fn every_active_body_must_compile_and_unknown_services_gain_no_host_authority() {
    for text in [
        "main(){return 7;}PrivateUnused(){wait 1;}",
        "main(){return openfile(\"PrivatePath\");}",
        "main(){self thread PrivateWorker();}PrivateWorker(){}",
    ] {
        let (_, records) = failed(text.as_bytes());
        assert_eq!(records.len(), 2);
        assert_eq!(records[1]["fields"]["phase"], "compile");
    }
}

#[test]
fn offline_infinite_control_flow_stops_at_global_instruction_fuel() {
    let (_, records) = failed(b"main(){while(1){}}");
    assert_eq!(records.len(), 3);
    assert_eq!(records[2]["fields"]["phase"], "execute");
    assert_eq!(records[2]["fields"]["steps"], 1_000_000);
    assert_eq!(records[2]["fields"]["peak_frames"], 1);
    assert!(
        records[2]["fields"]["cause"]
            .as_str()
            .expect("Typed cause")
            .contains("ExecutionSteps")
    );
}
