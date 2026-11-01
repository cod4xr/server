// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare real compiler accounting with exact and independently exhausted limits."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompilationResource, FunctionCompileErrorKind, ModuleCompilationBudget,
    ModuleCompilationResource,
};
use cod4x_script::execution::{KernelErrorKind, KernelResource};

const SOURCE: &[u8] = b"main(a){x=1;if(a){x=worker(x);}return x;}worker(x){return x+1;}";

#[test]
fn global_declaration_function_work_instruction_and_metadata_admission_is_exact()
-> Result<(), String> {
    let baseline = module_compile_support::compile(
        SOURCE,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Baseline global accounting");
    let usage = baseline.usage();
    let exact = ModuleCompilationBudget {
        max_declarations: usage.declarations,
        max_functions: usage.functions,
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..ModuleCompilationBudget::default()
    };
    assert_eq!(
        module_compile_support::compile(SOURCE, LegacyDeveloperMode::Disabled, exact)
            .expect("Exact global scopes")
            .usage(),
        usage
    );
    for resource in [
        ModuleCompilationResource::Declarations,
        ModuleCompilationResource::Functions,
        ModuleCompilationResource::Steps,
        ModuleCompilationResource::Instructions,
        ModuleCompilationResource::MetadataBytes,
    ] {
        let mut budget = exact;
        match resource {
            ModuleCompilationResource::Declarations => budget.max_declarations -= 1,
            ModuleCompilationResource::Functions => budget.max_functions -= 1,
            ModuleCompilationResource::Steps => budget.max_steps -= 1,
            ModuleCompilationResource::Instructions => budget.max_instructions -= 1,
            ModuleCompilationResource::MetadataBytes => budget.max_metadata_bytes -= 1,
            ModuleCompilationResource::CallArguments => {
                return Err("Call quota has its own per-site test".into());
            }
            ModuleCompilationResource::Modules
            | ModuleCompilationResource::ModuleNameBytes
            | ModuleCompilationResource::IncludeEdges
            | ModuleCompilationResource::ImportedFunctions => {
                return Err("Package namespace quotas have dedicated tests".into());
            }
        }
        let error = module_compile_support::compile(SOURCE, LegacyDeveloperMode::Disabled, budget)
            .expect_err("One less aggregate resource");
        assert!(
            matches!(error.cause.kind, FunctionCompileErrorKind::ModuleBudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum),
            "{error:?}"
        );
        assert!(error.cause.work_steps <= budget.max_steps);
        assert!(error.cause.peak_metadata_bytes <= budget.max_metadata_bytes);
    }
    Ok(())
}

#[test]
fn individual_lowering_limits_remain_distinct_from_module_caps() {
    for resource in [
        FunctionCompilationResource::Steps,
        FunctionCompilationResource::Instructions,
        FunctionCompilationResource::TaskFrames,
        FunctionCompilationResource::MetadataBytes,
        FunctionCompilationResource::LocalBindings,
        FunctionCompilationResource::IdentifierBytes,
        FunctionCompilationResource::ControlBytes,
    ] {
        let mut budget = ModuleCompilationBudget::default();
        match resource {
            FunctionCompilationResource::Steps => budget.function.max_steps = 0,
            FunctionCompilationResource::Instructions => budget.function.max_instructions = 0,
            FunctionCompilationResource::TaskFrames => budget.function.max_task_frames = 0,
            FunctionCompilationResource::MetadataBytes => budget.function.max_metadata_bytes = 0,
            FunctionCompilationResource::LocalBindings => budget.function.max_local_bindings = 0,
            FunctionCompilationResource::IdentifierBytes => {
                budget.function.max_identifier_bytes = 0;
            }
            FunctionCompilationResource::ControlBytes => budget.function.max_control_bytes = 0,
        }
        let error = module_compile_support::compile(SOURCE, LegacyDeveloperMode::Disabled, budget)
            .expect_err("Individual function scope remains enforced");
        assert_eq!(error.declaration_index(), Some(0));
        assert!(
            matches!(error.cause.kind, FunctionCompileErrorKind::BudgetExceeded { resource: actual, maximum: 0, .. } if actual == resource)
        );
        assert!(error.cause.work_steps > 0);
    }
}

#[test]
fn call_quotas_and_nested_body_literal_and_kernel_scopes_are_not_ignored() {
    let source = b"main(){return pair(1,2);}pair(a,b){return a+b;}";
    let mut budget = ModuleCompilationBudget::default();
    budget.function.max_call_arguments = 1;
    let error = module_compile_support::compile(source, LegacyDeveloperMode::Disabled, budget)
        .expect_err("Preflight call quota");
    assert!(matches!(
        error.cause.kind,
        FunctionCompileErrorKind::ModuleBudgetExceeded {
            resource: ModuleCompilationResource::CallArguments,
            requested: 2,
            maximum: 1
        }
    ));
    let mut budget = ModuleCompilationBudget::default();
    budget.function.body.max_statements = 0;
    assert!(matches!(
        module_compile_support::compile(source, LegacyDeveloperMode::Disabled, budget)
            .expect_err("Body-index scope")
            .cause
            .kind,
        FunctionCompileErrorKind::BodyList(_)
    ));
    let mut budget = ModuleCompilationBudget::default();
    budget.function.literal.max_encoded_bytes = 0;
    assert!(matches!(
        module_compile_support::compile(source, LegacyDeveloperMode::Disabled, budget)
            .expect_err("Literal scope")
            .cause
            .kind,
        FunctionCompileErrorKind::Literal(_)
    ));
    let mut budget = ModuleCompilationBudget::default();
    budget.verification.max_functions = 1;
    let error = module_compile_support::compile(source, LegacyDeveloperMode::Disabled, budget)
        .expect_err("Complete kernel proof scope");
    assert!(
        matches!(error.cause.kind, FunctionCompileErrorKind::Kernel(cause) if matches!(cause.kind, KernelErrorKind::BudgetExceeded { resource: KernelResource::Functions, .. }))
    );
    assert_eq!(error.declaration_index(), None);
    let mut budget = ModuleCompilationBudget::default();
    budget.verification.max_function_local_values = 0;
    let error = module_compile_support::compile(source, LegacyDeveloperMode::Disabled, budget)
        .expect_err("Located later kernel layout failure");
    assert_eq!(error.declaration_index(), Some(1));
    assert!(
        matches!(error.cause.kind, FunctionCompileErrorKind::Kernel(cause) if matches!(cause.kind, KernelErrorKind::BudgetExceeded { resource: KernelResource::LocalValues, .. }))
    );
}
