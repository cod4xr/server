// SPDX-License-Identifier: AGPL-3.0-or-later
//! Closed-package includes import function names without discovering host files.
#![allow(
    clippy::expect_used,
    reason = "Tests execute supplied include graphs through the real offline server process."
)]
mod content_support;
mod script_package_support;

use content_support::FixtureFile;
use script_package_support::{execute, failed, logs};

#[test]
fn included_calls_and_pointers_keep_callee_import_scopes_and_report_alias_counts() {
    let root = FixtureFile::from_bytes(b"#include middle;main(){f=::work;return work()+[[f]]();}");
    let middle = FixtureFile::from_bytes(b"#include lib;work(){return helper();}");
    let lib = FixtureFile::from_bytes(b"helper(){return 21;}");
    let output = execute(
        "PrivateRoot",
        &[("PrivateRoot", &root), ("middle", &middle), ("lib", &lib)],
        &["--json-logs"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"integer 42\n");
    let text = logs(&output, &[&root, &middle, &lib]);
    let records: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON"))
        .collect();
    assert_eq!(records[1]["fields"]["include_edges"], 2);
    assert_eq!(records[1]["fields"]["imported_functions"], 2);
    assert_eq!(records[1]["fields"]["functions"], 3);
}

#[test]
fn named_builtins_and_imported_references_keep_separate_resolution_with_developer_code() {
    for (source, lib_source, expected) in [
        (
            b"#include lib;main(){f=::isint;return isint(1)+[[f]](1);}".as_slice(),
            b"isint(a){return 4;}".as_slice(),
            b"integer 5\n".as_slice(),
        ),
        (
            b"#include lib;main(){return isint(1);}",
            b"/# isint(a){return 4;}#/",
            b"integer 1\n",
        ),
    ] {
        let root = FixtureFile::from_bytes(source);
        let lib = FixtureFile::from_bytes(lib_source);
        let output = execute(
            "PrivateRoot",
            &[("PrivateRoot", &root), ("lib", &lib)],
            &["--json-logs", "--gsc-developer", "enabled"],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected);
        logs(&output, &[&root, &lib]);
    }
}

#[test]
fn duplicate_conflicting_and_non_exported_include_names_fail_before_any_result() {
    for (source, kind) in [
        (
            b"#include middle;#include MIDDLE;main(){}".as_slice(),
            "DuplicateInclude",
        ),
        (
            b"#include middle;work(){}main(){}",
            "IncludeFunctionConflict",
        ),
        (
            b"#include middle;main(){return helper();}",
            "UnknownFunction",
        ),
        (b"main(){return middle::helper();}", "UnknownFunction"),
    ] {
        let root = FixtureFile::from_bytes(source);
        let middle = FixtureFile::from_bytes(b"#include lib;work(){return helper();}");
        let lib = FixtureFile::from_bytes(b"helper(){return 21;}");
        let output = execute(
            "PrivateRoot",
            &[("PrivateRoot", &root), ("middle", &middle), ("lib", &lib)],
            &["--json-logs"],
        );
        let error = failed(&output, &[&root, &middle, &lib]);
        assert_eq!(error["fields"]["phase"], "compile");
        assert_eq!(error["fields"]["module"], 0);
        assert!(
            error["fields"]["cause"]
                .as_str()
                .expect("Category")
                .contains(kind)
        );
    }
}

#[test]
fn included_runtime_failures_locate_the_original_module_and_line() {
    let root = FixtureFile::from_bytes(b"#include lib;main(){return work();}");
    let lib = FixtureFile::from_bytes(b"work(){\nreturn 1/0;\n}");
    let output = execute(
        "PrivateRoot",
        &[("PrivateRoot", &root), ("lib", &lib)],
        &["--json-logs"],
    );
    let error = failed(&output, &[&root, &lib]);
    assert_eq!(error["fields"]["phase"], "execute");
    assert_eq!(error["fields"]["module"], 1);
    assert_eq!(error["fields"]["line"], 2);
}
