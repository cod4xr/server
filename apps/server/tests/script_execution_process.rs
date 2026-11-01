// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real offline primitive execution reports results separately from sanitized logs.
#![allow(
    clippy::expect_used,
    reason = "Tests own synthetic source and real offline child processes."
)]

mod content_support;

use content_support::FixtureFile;
use serde_json::Value;
use std::process::Command;

#[test]
fn offline_execution_runs_real_calls_references_builtins_switch_and_reports_resources() {
    let text = b"// PrivateSourceSecret\nhelper(n){return n*2;}\nMaIn(){f=::helper;switch(isint(1)){case 1:return [[f]](21);default:return 0;}}";
    let file = FixtureFile::from_bytes(text);
    for format in ["--json-logs", "--human-logs"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args([format, "--color", "never", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Executor");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"integer 42\n");
        let logs = String::from_utf8(output.stderr).expect("Logs");
        assert!(!logs.contains("PrivateSourceSecret"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        assert!(!logs.contains("helper"));
        assert!(!logs.contains("server.started"));
        assert!(!logs.contains("script.syntax_inspected"));
        assert!(logs.contains("script.compiled") && logs.contains("script.executed"));
        if format == "--json-logs" {
            let records: Vec<Value> = logs
                .lines()
                .map(|line| serde_json::from_str(line).expect("JSON"))
                .collect();
            assert_eq!(records.len(), 3);
            assert_eq!(records[0]["fields"]["mode"], "primitive_execution");
            assert_eq!(records[1]["fields"]["profile"], "bound_primitive");
            assert_eq!(records[1]["fields"]["functions"], 2);
            let fields = &records[2]["fields"];
            assert_eq!(fields["result_kind"], "integer");
            assert!(fields["steps"].as_u64().expect("Steps") > 0);
            assert!(fields["setup_work"].as_u64().expect("Setup") > 0);
            assert_eq!(fields["peak_frames"], 2);
            assert!(fields.get("value").is_none());
        } else {
            assert!(logs.contains("INFO"));
            assert!(!logs.contains('\u{1b}'));
        }
    }
}

#[test]
fn offline_result_output_covers_every_primitive_category_without_exposing_function_identity() {
    for (text, result) in [
        ("main(){}", "undefined\n"),
        ("main(n){return isint(n);}", "integer 0\n"),
        ("main(){return 1.5;}", "float 1.5\n"),
        ("main(){return (1,2,3);}", "vector 1 2 3\n"),
        ("main(){return ::main;}", "function\n"),
    ] {
        let file = FixtureFile::from_bytes(text.as_bytes());
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Typed result");
        assert!(output.status.success());
        assert_eq!(output.stdout, result.as_bytes());
        assert!(
            !String::from_utf8(output.stderr)
                .expect("Logs")
                .contains("KernelFunctionReference")
        );
    }
}

#[test]
fn offline_entry_mapping_retains_disabled_declarations_and_explicit_developer_selection() {
    for (text, developer, expected) in [
        (
            "helper(){return 99;}/# disabled(){return \"private\";} #/ MAIN(){return 7;}",
            "disabled",
            7,
        ),
        (
            "helper(){return 99;}/# MaIn(){return 11;} #/",
            "enabled",
            11,
        ),
    ] {
        let file = FixtureFile::from_bytes(text.as_bytes());
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--gsc-developer", developer, "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Selected main");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, format!("integer {expected}\n").as_bytes());
    }
}
