// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline primitive definedness and evaluated-extra error behavior.
#![allow(
    clippy::expect_used,
    reason = "Tests run actual bounded offline definedness scripts."
)]
mod content_support;

use std::process::Command;

use content_support::FixtureFile;

#[test]
fn offline_definedness_preserves_optional_values_extras_and_private_evaluation_failures() {
    for (text, expected, failure) in [
        (
            br"main(){return isdefined(optional(),1);}optional(value){return value;}".as_slice(),
            b"integer 0\n".as_slice(),
            None,
        ),
        (
            br"main(){return isdefined(0,undefined);}",
            b"integer 1\n",
            None,
        ),
        (br"main(){return isdefined();}", b"", Some("ArgumentCount")),
        (
            br#"main(){return isdefined(undefined,bad());}bad(){return "PrivateIgnored"/1;}"#,
            b"",
            Some("UnmatchingTypes"),
        ),
    ] {
        let file = FixtureFile::from_bytes(text);
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Offline definedness");
        assert_eq!(output.status.success(), failure.is_none());
        assert_eq!(output.stdout, expected);
        let logs = String::from_utf8(output.stderr).expect("Private logs");
        assert!(!logs.contains("PrivateIgnored"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        if let Some(category) = failure {
            assert!(logs.contains(category));
            assert!(logs.contains("\"phase\":\"execute\""));
        }
    }
}
