// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests admit complete include/alias staging and exhaust every compiler-work prefix."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::{
    FunctionCompileErrorKind, ModuleCompilationResource, ScriptPackageBudget,
};

const INPUTS: &[(&[u8], &[u8])] = &[
    (
        b"root",
        b"#include lib;main(){f=::work;return work(1)+[[f]](2);}",
    ),
    (b"lib", b"work(a){return helper(a);}helper(a){return a+1;}"),
];

#[test]
fn include_and_alias_limits_are_separate_exact_admission_with_atomic_failure() -> Result<(), String>
{
    let baseline =
        compile(INPUTS, Disabled, ScriptPackageBudget::default()).expect("Include baseline");
    let usage = baseline.usage();
    let mut exact = ScriptPackageBudget {
        max_include_edges: usage.include_edges,
        max_imported_functions: usage.imported_functions,
        ..ScriptPackageBudget::default()
    };
    exact.compilation.max_steps = usage.compilation.compiler_steps;
    exact.compilation.max_metadata_bytes = usage.compilation.peak_metadata_bytes;
    assert_eq!(
        compile(INPUTS, Disabled, exact)
            .expect("Exact staging/work")
            .usage(),
        usage
    );
    for resource in [
        ModuleCompilationResource::IncludeEdges,
        ModuleCompilationResource::ImportedFunctions,
        ModuleCompilationResource::Steps,
        ModuleCompilationResource::MetadataBytes,
    ] {
        let mut budget = exact;
        match resource {
            ModuleCompilationResource::IncludeEdges => budget.max_include_edges -= 1,
            ModuleCompilationResource::ImportedFunctions => budget.max_imported_functions -= 1,
            ModuleCompilationResource::Steps => budget.compilation.max_steps -= 1,
            ModuleCompilationResource::MetadataBytes => budget.compilation.max_metadata_bytes -= 1,
            ModuleCompilationResource::Modules
            | ModuleCompilationResource::ModuleNameBytes
            | ModuleCompilationResource::Declarations
            | ModuleCompilationResource::Functions
            | ModuleCompilationResource::Instructions
            | ModuleCompilationResource::CallArguments => {
                return Err("Other compiler quotas have dedicated package tests".into());
            }
        }
        let error = compile(INPUTS, Disabled, budget).expect_err("One less admitted resource");
        assert!(
            matches!(error.cause.kind, FunctionCompileErrorKind::ModuleBudgetExceeded {
            resource: actual, requested, maximum } if actual == resource && requested > maximum),
            "{error:?}"
        );
        assert!(error.cause.work_steps <= budget.compilation.max_steps);
        assert!(error.cause.peak_metadata_bytes <= budget.compilation.max_metadata_bytes);
    }
    Ok(())
}

#[test]
fn every_shorter_include_compiler_work_budget_stops_without_partial_publication() {
    let usage = compile(INPUTS, Disabled, ScriptPackageBudget::default())
        .expect("Baseline")
        .usage();
    for maximum in 0..usage.compilation.compiler_steps {
        let mut budget = ScriptPackageBudget::default();
        budget.compilation.max_steps = maximum;
        let error = compile(INPUTS, Disabled, budget)
            .expect_err("All failed namespace/inventory/lowering/proof-map prefixes");
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
        usage
    );
}
