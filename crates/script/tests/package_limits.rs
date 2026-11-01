// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare admitted package usage against exact limits and every failed work prefix."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::{
    FunctionCompilationResource, FunctionCompileErrorKind, ModuleCompilationResource,
    ScriptPackageBudget,
};

const INPUTS: &[(&[u8], &[u8])] = &[
    (b"z", b"main(){f=a::helper;return [[f]](2);}"),
    (b"A", b"helper(x){return x+1;}"),
    (b"middle", b""),
];

#[test]
fn package_global_limits_admit_exact_usage_and_reject_each_one_less() -> Result<(), String> {
    let usage = compile(INPUTS, Disabled, ScriptPackageBudget::default())
        .expect("Baseline")
        .usage();
    let mut exact = ScriptPackageBudget {
        max_modules: 3,
        max_path_input_bytes: 8,
        ..ScriptPackageBudget::default()
    };
    exact.compilation.max_declarations = usage.compilation.declarations;
    exact.compilation.max_functions = usage.compilation.functions;
    exact.compilation.max_steps = usage.compilation.compiler_steps;
    exact.compilation.max_instructions = usage.compilation.instructions;
    exact.compilation.max_metadata_bytes = usage.compilation.peak_metadata_bytes;
    assert_eq!(
        compile(INPUTS, Disabled, exact)
            .expect("Exact shared resources")
            .usage(),
        usage
    );
    assert!(usage.retained_metadata_bytes <= usage.compilation.peak_metadata_bytes);
    for resource in [
        ModuleCompilationResource::Modules,
        ModuleCompilationResource::ModuleNameBytes,
        ModuleCompilationResource::Declarations,
        ModuleCompilationResource::Functions,
        ModuleCompilationResource::Steps,
        ModuleCompilationResource::Instructions,
        ModuleCompilationResource::MetadataBytes,
    ] {
        let mut budget = exact;
        match resource {
            ModuleCompilationResource::Modules => budget.max_modules -= 1,
            ModuleCompilationResource::ModuleNameBytes => budget.max_path_input_bytes -= 1,
            ModuleCompilationResource::Declarations => budget.compilation.max_declarations -= 1,
            ModuleCompilationResource::Functions => budget.compilation.max_functions -= 1,
            ModuleCompilationResource::Steps => budget.compilation.max_steps -= 1,
            ModuleCompilationResource::Instructions => budget.compilation.max_instructions -= 1,
            ModuleCompilationResource::MetadataBytes => budget.compilation.max_metadata_bytes -= 1,
            ModuleCompilationResource::CallArguments => {
                return Err("Per-call limit checked separately".into());
            }
        }
        let error = compile(INPUTS, Disabled, budget).expect_err("One less shared resource");
        assert!(
            matches!(error.cause.kind,
            FunctionCompileErrorKind::ModuleBudgetExceeded { resource: actual, requested, maximum }
                if actual == resource && requested > maximum),
            "{error:?}"
        );
        assert!(error.cause.work_steps <= budget.compilation.max_steps);
        assert!(error.cause.peak_metadata_bytes <= budget.compilation.max_metadata_bytes);
    }
    Ok(())
}

#[test]
fn every_insufficient_work_prefix_fails_atomically_including_sort_and_lookup() {
    let baseline = compile(INPUTS, Disabled, ScriptPackageBudget::default()).expect("Baseline");
    let steps = baseline.usage().compilation.compiler_steps;
    for maximum in 0..steps {
        let mut budget = ScriptPackageBudget::default();
        budget.compilation.max_steps = maximum;
        let error = compile(INPUTS, Disabled, budget).expect_err("No partial package");
        assert!(
            matches!(error.cause.kind, FunctionCompileErrorKind::ModuleBudgetExceeded {
            resource: ModuleCompilationResource::Steps, requested, maximum: actual,
        } if requested > maximum && actual == maximum),
            "{maximum}: {error:?}"
        );
        assert!(error.cause.work_steps <= maximum, "{maximum}: {error:?}");
    }
    assert_eq!(
        compile(INPUTS, Disabled, ScriptPackageBudget::default())
            .expect("Clean retry")
            .usage(),
        baseline.usage()
    );
}

#[test]
fn path_function_and_proof_quotas_remain_separately_admitted() {
    let mut budget = ScriptPackageBudget::default();
    budget.path.max_input_bytes = 5;
    assert!(matches!(
        compile(INPUTS, Disabled, budget)
            .expect_err("Per-name quota")
            .cause
            .kind,
        FunctionCompileErrorKind::ScriptPath(_)
    ));
    let mut budget = ScriptPackageBudget::default();
    budget.compilation.function.max_identifier_bytes = 0;
    assert!(matches!(
        compile(INPUTS, Disabled, budget)
            .expect_err("Per-function names")
            .cause
            .kind,
        FunctionCompileErrorKind::BudgetExceeded {
            resource: FunctionCompilationResource::IdentifierBytes,
            ..
        }
    ));
    let mut budget = ScriptPackageBudget::default();
    budget.compilation.verification.max_functions = 1;
    let error = compile(INPUTS, Disabled, budget).expect_err("One complete image proof");
    assert!(matches!(
        error.cause.kind,
        FunctionCompileErrorKind::Kernel(_)
    ));
    assert_eq!(error.module_index(), None);
    assert_eq!(error.declaration_index(), None);
}
