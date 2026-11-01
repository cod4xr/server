// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Real source developer processes need selected execution, private failures and readable accounting."
)]
use crate::content_support::FixtureFile;
use crate::script_package_support;
use std::process::Command;

#[test]
fn offline_developer_mode_selects_regions_and_reports_retained_and_peak_instructions_privately() {
    let file = FixtureFile::from_bytes(include_bytes!(
        "../../../../examples/gsc/developer_blocks.gsc"
    ));
    for (mode, expected) in [
        ("disabled", b"integer 20\n".as_slice()),
        ("enabled", b"integer 42\n"),
    ] {
        for format in ["--json-logs", "--human-logs"] {
            let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
                .args([
                    format,
                    "--color",
                    "never",
                    "--gsc-developer",
                    mode,
                    "--execute-gsc",
                ])
                .arg(&file.0)
                .output()
                .expect("Developer process");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, expected);
            let logs = script_package_support::logs(&output, &[&file]);
            assert!(!logs.contains('\u{1b}'));
            if format == "--json-logs" {
                let compiled: serde_json::Value = logs
                    .lines()
                    .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                    .find(|record| record["fields"]["event"] == "script.compiled")
                    .expect("Compile accounting");
                assert!(
                    compiled["fields"]["peak_instructions"]
                        .as_u64()
                        .expect("Temporary peak")
                        >= compiled["fields"]["instructions"]
                            .as_u64()
                            .expect("Retained code")
                );
            } else {
                assert!(logs.contains("peak_instructions="));
            }
        }
    }
}

#[test]
fn supplied_package_discards_unknown_developer_names_without_executable_targets_or_tables() {
    let root = FixtureFile::from_bytes(
        b"#include lib;main(){/# unknown(\"Private\");f=absent::missing; #/return score();}",
    );
    let lib = FixtureFile::from_bytes(b"score(){/# missing(); #/return 42;}");
    let output = script_package_support::execute(
        "root",
        &[("root", &root), ("lib", &lib)],
        &[
            "--gsc-developer",
            "disabled",
            "--human-logs",
            "--color",
            "never",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"integer 42\n");
    let logs = script_package_support::logs(&output, &[&root, &lib]);
    assert!(logs.contains("string_literals=0"));
    assert!(logs.contains("live_array_values=0"));
}

#[test]
fn disabled_nested_regions_and_active_unknown_targets_keep_compile_locations_private() {
    for (source, mode, cause, line) in [
        (
            b"main(){\n/#\n/# PrivateUnknown(); #/\n#/return 0;}".as_slice(),
            "disabled",
            "NestedDeveloperBlock",
            3,
        ),
        (
            b"main(){\n/# PrivateUnknown(); #/\nreturn 0;}",
            "enabled",
            "UnknownFunction",
            2,
        ),
    ] {
        let file = FixtureFile::from_bytes(source);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--gsc-developer", mode, "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Rejected developer source");
        let record = script_package_support::failed(&output, &[&file]);
        assert_eq!(record["fields"]["phase"], "compile");
        assert_eq!(record["fields"]["cause"], cause);
        assert_eq!(record["fields"]["line"], line);
    }
}
