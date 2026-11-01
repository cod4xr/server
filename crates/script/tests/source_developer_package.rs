// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Supplied packages must preserve each module's developer policy across includes and calls."
)]
use crate::{array_read_support::number, package_compile_support::compile_with_modes};
use cod4x_script::binding::LegacyDeveloperMode::{Disabled, Enabled};
use cod4x_script::compilation::ScriptPackageBudget;
use cod4x_script::execution::KernelCallBudget;

#[test]
fn included_packages_preserve_distinct_per_module_modes_and_shared_peak_admission() {
    let inputs: &[(&[u8], &[u8])] = &[
        (
            b"root",
            b"#include lib;main(){/# unknown(\"Private\");return 99; #/return score();}",
        ),
        (b"lib", b"score(){n=1;/# n+=41; #/return n;}"),
    ];
    let modes = [Disabled, Enabled];
    let package = compile_with_modes(inputs, &modes, ScriptPackageBudget::default())
        .expect("Mixed module policies");
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Per-module selected regions")
            .value,
        number(42)
    );
    assert_eq!(package.usage().compilation.string_literals, 0);
    assert_eq!(package.modules()[0].developer_mode, Disabled);
    assert_eq!(package.modules()[1].developer_mode, Enabled);
    let usage = package.usage().compilation;
    let mut exact = ScriptPackageBudget::default();
    exact.compilation.max_steps = usage.compiler_steps;
    exact.compilation.max_metadata_bytes = usage.peak_metadata_bytes;
    exact.compilation.max_instructions = usage.peak_instructions;
    assert_eq!(
        compile_with_modes(inputs, &modes, exact)
            .expect("Exact global temporary/staged overlap")
            .usage(),
        package.usage()
    );
    for resource in 0..3 {
        let mut short = exact;
        match resource {
            0 => short.compilation.max_steps -= 1,
            1 => short.compilation.max_metadata_bytes -= 1,
            _ => short.compilation.max_instructions -= 1,
        }
        compile_with_modes(inputs, &modes, short).expect_err("One less shared package resource");
    }
}
