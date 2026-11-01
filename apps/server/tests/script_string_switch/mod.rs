// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Real offline processes must log switch counts and located failures without source labels."
)]
use crate::content_support::FixtureFile;
use crate::script_package_support;
use std::process::Command;

#[test]
fn offline_string_switch_example_returns_42_with_separate_label_counts_and_private_readable_logs() {
    let file = FixtureFile::from_bytes(include_bytes!(
        "../../../../examples/gsc/string_switches.gsc"
    ));
    for format in ["--json-logs", "--human-logs"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args([format, "--color", "never", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Source switches");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"integer 42\n");
        let logs = script_package_support::logs(&output, &[&file]);
        assert!(!logs.contains("ranger"));
        assert!(!logs.contains('\u{1b}'));
        if format == "--json-logs" {
            let compiled: serde_json::Value = logs
                .lines()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .find(|record| record["fields"]["event"] == "script.compiled")
                .expect("Compilation event");
            assert_eq!(compiled["fields"]["switch_labels"], 3);
            assert_eq!(compiled["fields"]["string_literals"], 1);
            assert!(
                compiled["fields"]["staged_switch_bytes"]
                    .as_u64()
                    .expect("Label bytes")
                    > 0
            );
        } else {
            assert!(logs.contains("switch_labels=3"));
            assert!(logs.contains("INFO"));
        }
    }
}

#[test]
fn supplied_include_package_switches_keep_independent_labels_and_counts_in_pretty_logs() {
    let root = FixtureFile::from_bytes(br#"#include lib;main(){switch("PrivateRoot"){case "PrivateRoot":return choose("PrivateLib");default:return 0;}}"#);
    let lib = FixtureFile::from_bytes(
        br#"choose(k){switch(k){case "PrivateLib":return 42;default:return 0;}}"#,
    );
    let output = script_package_support::execute(
        "root",
        &[("root", &root), ("lib", &lib)],
        &["--human-logs", "--color", "never"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"integer 42\n");
    let logs = script_package_support::logs(&output, &[&root, &lib]);
    assert!(logs.contains("switch_labels=2"));
    assert!(logs.contains("live_array_values=0"));
}

#[test]
fn duplicate_labels_and_rejected_array_keys_report_private_lines_and_cleanup() {
    for (source, phase, cause) in [
        (
            b"main(k){switch(k){\ncase \"Private\":return 1;\ncase \"Private\":return 2;}}"
                .as_slice(),
            "compile",
            "DuplicateCase",
        ),
        (
            b"main(){a=[];a[0]=1;\nswitch(a){case \"Private\":return 1;}\nreturn 0;}",
            "execute",
            "ExpectedSwitchKey",
        ),
    ] {
        let file = FixtureFile::from_bytes(source);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Rejected switch");
        let record = script_package_support::failed(&output, &[&file]);
        assert_eq!(record["fields"]["phase"], phase);
        assert_eq!(record["fields"]["cause"], cause);
        assert_eq!(
            record["fields"]["line"],
            if phase == "compile" { 3 } else { 2 }
        );
        if phase == "execute" {
            assert_eq!(record["fields"]["live_array_values"], 0);
        }
    }
}
