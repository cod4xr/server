// SPDX-License-Identifier: AGPL-3.0-or-later
//! Failed package phases locate original modules and publish no result.
#![allow(
    clippy::expect_used,
    reason = "Tests own failing source files and inspect sanitized real-process events."
)]
mod content_support;
mod script_package_support;

use content_support::FixtureFile;
use script_package_support::{execute, failed};

#[test]
fn later_module_parse_binding_and_compile_errors_keep_private_source_coordinates() {
    let root = FixtureFile::from_bytes(b"main(){return 1;}");
    for (text, phase, declaration, line) in [
        (b"// PrivatePayload\nbad( {".as_slice(), "parse", None, 2),
        (b"first(){}\nFiRsT(){}", "binding", None, 2),
        (
            b"first(){}\nother(){return PrivateMissing();}",
            "compile",
            Some(1),
            2,
        ),
    ] {
        let lib = FixtureFile::from_bytes(text);
        let output = execute(
            "PrivateRoot",
            &[("PrivateRoot", &root), ("lib", &lib)],
            &["--json-logs"],
        );
        let error = failed(&output, &[&root, &lib]);
        assert_eq!(error["fields"]["phase"], phase);
        assert_eq!(error["fields"]["module"], 1);
        assert_eq!(error["fields"]["line"], line);
        if let Some(declaration) = declaration {
            assert_eq!(error["fields"]["declaration"], declaration);
        }
    }
}

#[test]
fn package_runtime_errors_and_fuel_exhaustion_keep_callee_locations() {
    let root = FixtureFile::from_bytes(b"main(){return lib::value();}");
    for text in [
        b"value(){\nreturn 1/0;\n}".as_slice(),
        b"value(){\nwhile(1){}\n}",
    ] {
        let lib = FixtureFile::from_bytes(text);
        let output = execute(
            "PrivateRoot",
            &[("PrivateRoot", &root), ("lib", &lib)],
            &["--json-logs"],
        );
        let error = failed(&output, &[&root, &lib]);
        assert_eq!(error["fields"]["phase"], "execute");
        assert_eq!(error["fields"]["module"], 1);
        assert_eq!(error["fields"]["line"], 2);
        assert!(error["fields"]["steps"].as_u64().expect("Charged steps") <= 1_000_000);
    }
}

#[test]
fn all_supplied_directives_are_rejected_before_execution_including_unused_files() {
    let root = FixtureFile::from_bytes(b"main(){return 1;}");
    for directive in [
        b"#include maps\\PrivateCommon;".as_slice(),
        b"#using_animtree(\"PrivateAnim\");",
    ] {
        let lib = FixtureFile::from_bytes(directive);
        let output = execute(
            "PrivateRoot",
            &[("PrivateRoot", &root), ("lib", &lib)],
            &["--json-logs"],
        );
        let error = failed(&output, &[&root, &lib]);
        assert_eq!(error["fields"]["phase"], "directives");
        assert_eq!(error["fields"]["module"], 1);
    }
}

#[test]
fn duplicate_unknown_and_missing_entry_identities_fail_without_fallback() {
    for (source, second_name, entry, phase, kind) in [
        (
            b"main(){return PrivateAbsent::isint(1);}".as_slice(),
            "lib",
            "root",
            "compile",
            "UnknownModule",
        ),
        (
            b"main(){return lib::isint(1);}",
            "lib",
            "root",
            "compile",
            "UnknownFunction",
        ),
        (
            b"main(){return 1;}",
            "//ROOT",
            "root",
            "compile",
            "DuplicateModule",
        ),
        (b"main(){return 1;}", "lib", "PrivateAbsent", "entry", ""),
        (b"helper(){return 1;}", "lib", "root", "entry", ""),
    ] {
        let root = FixtureFile::from_bytes(source);
        let lib = FixtureFile::from_bytes(b"other(){}");
        let output = execute(
            entry,
            &[("root", &root), (second_name, &lib)],
            &["--json-logs"],
        );
        let error = failed(&output, &[&root, &lib]);
        assert_eq!(error["fields"]["phase"], phase);
        if !kind.is_empty() {
            assert!(
                error["fields"]["cause"]
                    .as_str()
                    .expect("Category")
                    .contains(kind)
            );
        }
    }
}

#[test]
fn combined_snapshots_are_bounded_before_parsing_or_execution() {
    let bytes = vec![b' '; 33 * 1024 * 1024];
    let file = FixtureFile::from_bytes(&bytes);
    let output = execute(
        "PrivateRoot",
        &[("PrivateRoot", &file), ("lib", &file)],
        &["--json-logs"],
    );
    let error = failed(&output, &[&file]);
    assert_eq!(error["fields"]["phase"], "read");
    assert_eq!(error["fields"]["module"], 1);
    assert_eq!(error["fields"]["kind"], "file_budget");
}
