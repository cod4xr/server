// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real supplied-file packages share typed output, developer rules and private logs.
#![allow(
    clippy::expect_used,
    reason = "Tests own complete source catalogs and real offline processes."
)]
mod content_support;
mod script_package_support;

use content_support::FixtureFile;
use script_package_support::{execute, logs};
use serde_json::Value;
use std::fs;
use std::process::Command;

#[test]
fn package_mode_runs_qualified_references_local_names_and_nonzero_main_mapping() {
    let root = FixtureFile::from_bytes(
        b"// PrivatePayload\nhelper(){return 1;}MaIn(){f=maps\\lib::twice;return [[f]](20);}",
    );
    let lib = FixtureFile::from_bytes(b"helper(){return 2;}twice(n){return n*2+helper();}");
    for format in ["--json-logs", "--human-logs"] {
        let output = execute(
            "///PRIVATE/ROOT",
            &[("maps/lib", &lib), ("Private/Root", &root)],
            &[format, "--color", "never"],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"integer 42\n");
        let logs = logs(&output, &[&root, &lib]);
        assert!(!logs.contains("twice") && !logs.contains("helper"));
        if format == "--json-logs" {
            let records: Vec<Value> = logs
                .lines()
                .map(|line| serde_json::from_str(line).expect("JSON"))
                .collect();
            assert_eq!(records.len(), 3);
            assert_eq!(records[0]["fields"]["mode"], "primitive_package_execution");
            assert_eq!(records[1]["fields"]["profile"], "bound_primitive_package");
            assert_eq!(records[1]["fields"]["modules"], 2);
            assert_eq!(records[1]["fields"]["functions"], 4);
            for resource in [
                "source_bytes",
                "source_index_bytes",
                "grammar_steps",
                "scan_steps",
                "binding_steps",
            ] {
                assert!(
                    records[1]["fields"][resource]
                        .as_u64()
                        .expect("Frontend metrics")
                        > 0
                );
            }
            assert_eq!(records[2]["fields"]["result_kind"], "integer");
            assert_eq!(records[2]["fields"]["peak_frames"], 3);
            assert!(records[2]["fields"].get("value").is_none());
        } else {
            assert!(logs.contains("INFO"));
            assert!(!logs.contains('\u{1b}'));
        }
    }
}

#[test]
fn package_recursive_graph_executes_and_developer_entries_remain_explicit() {
    let root = FixtureFile::from_bytes(b"/# MaIn(){return lib::even(100);}#/ ");
    let lib = FixtureFile::from_bytes(
        b"even(n){if(n==0){return 1;}return odd(n-1);}odd(n){if(n==0){return 0;}return even(n-1);}",
    );
    let inputs = [("PrivateRoot", &root), ("lib", &lib)];
    let disabled = execute("PrivateRoot", &inputs, &["--json-logs"]);
    let failure = script_package_support::failed(&disabled, &[&root, &lib]);
    assert_eq!(failure["fields"]["phase"], "entry");
    assert_eq!(failure["fields"]["module"], 0);
    let enabled = execute(
        "PrivateRoot",
        &inputs,
        &["--json-logs", "--gsc-developer", "enabled"],
    );
    assert!(
        enabled.status.success(),
        "{}",
        String::from_utf8_lossy(&enabled.stderr)
    );
    assert_eq!(enabled.stdout, b"integer 1\n");
    logs(&enabled, &[&root, &lib]);
}

#[test]
fn package_returns_heap_values_and_function_categories_through_existing_stdout() {
    for (body, expected) in [
        ("return \"PrivateString\";", "string b\"PrivateString\"\n"),
        (
            "return &\"PrivateLocalized\";",
            "localized_string b\"PrivateLocalized\"\n",
        ),
        ("return spawnstruct();", "object\n"),
        ("return ::value;", "function\n"),
        ("return (1,2,3);", "vector 1 2 3\n"),
    ] {
        let root = FixtureFile::from_bytes(b"main(){return lib::value();}");
        let lib = FixtureFile::from_bytes(format!("value(){{{body}}}").as_bytes());
        let output = execute("root", &[("root", &root), ("lib", &lib)], &["--json-logs"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected.as_bytes());
        logs(&output, &[&root, &lib]);
    }
}

#[test]
fn package_file_logs_remain_json_and_flush_before_the_offline_process_exits() {
    let root = FixtureFile::from_bytes(b"main(){return lib::value();}");
    let lib = FixtureFile::from_bytes(b"value(){return 42;}");
    let directory = root.0.with_extension("package-logs");
    fs::create_dir(&directory).expect("New test-owned log directory");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--human-logs", "--color", "never", "--log-dir"])
        .arg(&directory)
        .args([
            "--execute-gsc-package",
            "PrivateRoot",
            "--gsc-module",
            "PrivateRoot",
        ])
        .arg(&root.0)
        .args(["--gsc-module", "lib"])
        .arg(&lib.0)
        .output()
        .expect("File logger process");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"integer 42\n");
    let console = logs(&output, &[&root, &lib]);
    assert!(console.contains("INFO"));
    let files: Vec<_> = fs::read_dir(&directory)
        .expect("Owned directory")
        .map(|entry| entry.expect("Entry").path())
        .collect();
    assert_eq!(files.len(), 1);
    let text = fs::read_to_string(&files[0]).expect("Flushed file");
    let records: Vec<Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON file record"))
        .collect();
    assert_eq!(records.len(), 3);
    assert_eq!(records[2]["fields"]["event"], "script.executed");
    assert_eq!(records[2]["fields"]["profile"], "bound_primitive_package");
    assert!(!text.contains("Private"));
    assert!(!text.contains(root.0.to_string_lossy().as_ref()));
    assert!(!text.contains(lib.0.to_string_lossy().as_ref()));
    for file in files {
        fs::remove_file(file).expect("Remove exact test-owned log file");
    }
    fs::remove_dir(directory).expect("Remove empty test-owned directory");
}
