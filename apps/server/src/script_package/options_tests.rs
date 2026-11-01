// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust CLI catalog admission without platform command-line length limits."
)]
use std::ffi::OsString;

use cod4x_script::compilation::ScriptPackageBudget;

use super::options::{self, ScriptPackageOptions};

#[test]
fn catalog_count_and_raw_name_bytes_are_bounded_before_source_file_access() {
    let budget = ScriptPackageBudget::default();
    let mut options = ScriptPackageOptions::default();
    for _ in 0..budget.max_modules {
        options
            .push(OsString::from("m"), OsString::from("PrivatePath"))
            .expect("Admitted row");
    }
    assert!(
        options
            .push(OsString::from("m"), OsString::from("PrivatePath"))
            .is_err()
    );
    assert_eq!(options.inputs.len() as u64, budget.max_modules);
    let raw = format!("{}m", "\n".repeat(1024 * 1024 - 1));
    let mut options = ScriptPackageOptions::default();
    for _ in 0..64 {
        options
            .push(OsString::from(&raw), OsString::from("PrivatePath"))
            .expect("Full raw bytes admitted");
    }
    assert_eq!(options.inputs[0].name.as_bytes(), b"m");
    assert!(
        options
            .push(OsString::from("m"), OsString::from("PrivatePath"))
            .is_err()
    );
    assert_eq!(options.inputs.len(), 64);
    assert!(!format!("{options:?}").contains("Private"));
    for name in [
        "",
        "\n\r",
        "Private\0Tail",
        "Privaté",
        &"a".repeat(1024 * 1024 + 1),
    ] {
        let error = options::canonical(OsString::from(name)).expect_err("Invalid CLI name");
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
}
