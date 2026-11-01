// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests admit field update lowering, proof and execution at exact resource boundaries."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;

#[test]
fn field_updates_admit_exact_function_module_and_proof_resources_and_reject_one_less() {
    let source =
        b"main(){o=spawnstruct();o.value=15;o.value*=2;o.value++;o.value--;return o.value;}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Measured field updates");
    let usage = module.usage();
    let function = module.functions()[0].usage();
    let mut exact = ModuleCompilationBudget {
        max_steps: usage.compiler_steps,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..ModuleCompilationBudget::default()
    };
    exact.function.max_steps = function.compiler_steps;
    exact.function.max_instructions = function.instructions;
    exact.function.max_task_frames = function.peak_task_frames;
    exact.function.max_metadata_bytes = function.peak_metadata_bytes;
    exact.function.max_identifier_bytes = function.identifier_bytes;
    exact.verification.max_steps = usage.verification.verification_steps;
    exact.verification.max_metadata_bytes = usage.verification.verification_metadata_bytes;
    assert_eq!(
        module_compile_support::compile(source, LegacyDeveloperMode::Disabled, exact)
            .expect("Exact all resource scopes")
            .usage(),
        usage
    );
    for resource in 0..9 {
        let mut budget = exact;
        match resource {
            0 => budget.max_steps -= 1,
            1 => budget.max_metadata_bytes -= 1,
            2 => budget.function.max_steps -= 1,
            3 => budget.function.max_instructions -= 1,
            4 => budget.function.max_task_frames -= 1,
            5 => budget.function.max_metadata_bytes -= 1,
            6 => budget.function.max_identifier_bytes -= 1,
            7 => budget.verification.max_steps -= 1,
            _ => budget.verification.max_metadata_bytes -= 1,
        }
        assert!(
            module_compile_support::compile(source, LegacyDeveloperMode::Disabled, budget).is_err(),
            "resource {resource}"
        );
    }
}
