// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real offline source construction preserves opaque output, accounting and private located failures.
#![allow(
    clippy::expect_used,
    reason = "Tests execute synthetic source through the actual offline command and require complete JSON records."
)]
mod content_support;
mod script_package_support;

use std::process::Command;

use content_support::FixtureFile;

#[test]
fn supplied_package_array_writes_return_populated_data_across_includes_with_readable_logs() {
    let root = FixtureFile::from_bytes(b"#include lib;main(){a=make();a[1]=20;return a[0]+a[1];}");
    let lib = FixtureFile::from_bytes(b"make(){a=[];a[0]=22;return a;}");
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
    assert!(script_package_support::logs(&output, &[&root, &lib]).contains("live_array_values=0"));
}

#[test]
fn failed_source_write_reports_its_private_location_and_zero_live_vm_owners_after_cow() {
    let file = FixtureFile::from_bytes(
        b"main(){/* PrivateSource */a=[];a[0]=1;\n b=a;\n a[-1]=9;\n return 0;}",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Failed source write");
    let event = script_package_support::failed(&output, &[&file]);
    let fields = &event["fields"];
    assert_eq!(fields["phase"], "execute");
    assert_eq!(fields["cause"], "ArrayIndexOutOfRange");
    assert_eq!(fields["line"], 3);
    assert_eq!(fields["live_array_values"], 0);
    assert_eq!(fields["live_arrays"], 2);
}

#[test]
fn offline_source_writes_populate_independent_arrays_and_keep_results_and_logs_private() {
    let file = FixtureFile::from_bytes(include_bytes!("../../../examples/gsc/array_writes.gsc"));
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Offline writable source");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"integer 42\n");
    let text = script_package_support::logs(&output, &[&file]);
    let event = text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|record| record["fields"]["event"] == "script.executed")
        .expect("Completion event");
    assert_eq!(event["fields"]["live_array_values"], 0);
    assert!(
        event["fields"]["live_fields"]
            .as_u64()
            .expect("Numeric field count")
            >= 3
    );
}

#[test]
fn supplied_package_scopes_keep_calls_and_switch_loop_exits_owned_and_logs_readable() {
    let root = FixtureFile::from_bytes(b"#include lib;main(){return run();}");
    let source =
        include_str!("../../../examples/gsc/local_scopes.gsc").replacen("main()", "run()", 1);
    let lib = FixtureFile::from_bytes(source.as_bytes());
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
    assert!(logs.contains("live_array_values=0"));
    assert!(!logs.contains('\u{1b}'));
}

#[test]
fn offline_empty_arrays_report_opaque_results_and_managed_counts_without_implicit_world_state() {
    for (source, expected, owners) in [
        (
            b"main(){/* PrivateSource */return []; }".as_slice(),
            b"array\n".as_slice(),
            1,
        ),
        (
            include_bytes!("../../../examples/gsc/empty_arrays.gsc"),
            b"integer 1\n",
            0,
        ),
    ] {
        let file = FixtureFile::from_bytes(source);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Offline constructor");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected);
        let text = script_package_support::logs(&output, &[&file]);
        assert!(!text.contains("ArrayValueId"));
        assert!(!text.contains("ObjectId"));
        let records: Vec<serde_json::Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).expect("Complete JSON event"))
            .collect();
        let fields = &records
            .iter()
            .find(|record| record["fields"]["event"] == "script.executed")
            .expect("Completion event")["fields"];
        assert_eq!(fields["live_arrays"], 1);
        assert_eq!(fields["live_array_values"], owners);
        assert_eq!(fields["array_metadata_rows"], 1);
        assert!(fields["array_value_slots"].as_u64().expect("Value slots") >= 1);
        assert_eq!(fields["live_fields"], 0);
        assert_eq!(fields["array_owner_audits"], 0);
        assert_eq!(fields["array_owner_audit_reuses"], 0);
    }
}

#[test]
fn supplied_package_constructors_keep_development_logs_readable_and_results_private() {
    let root =
        FixtureFile::from_bytes(b"#include lib;main(){a=make();return a.size+isdefined(a);}");
    let lib = FixtureFile::from_bytes(b"make(){return [];}");
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
    assert_eq!(output.stdout, b"integer 1\n");
    let text = script_package_support::logs(&output, &[&root, &lib]);
    assert!(text.contains("script.executed"));
    assert!(text.contains("live_array_values=0"));
    assert!(!text.contains('\u{1b}'));
}

#[test]
fn invalid_index_after_construction_is_located_and_logs_zero_live_vm_owners_after_cleanup() {
    let file = FixtureFile::from_bytes(b"main(){/* PrivateSource */\n a=[];\n return a[-1];\n}");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--json-logs", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Invalid read process");
    let record = script_package_support::failed(&output, &[&file]);
    let fields = &record["fields"];
    assert_eq!(fields["phase"], "execute");
    assert_eq!(fields["cause"], "ArrayIndexOutOfRange");
    assert_eq!(fields["line"], 3);
    assert_eq!(fields["live_array_values"], 0);
    assert_eq!(fields["live_arrays"], 1);
}
