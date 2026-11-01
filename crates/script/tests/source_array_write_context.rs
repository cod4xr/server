// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Unsupported targets and literal budgets must fail before publication."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::FunctionLoweringBudget;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::literal::{LegacyLiteralError, LegacyLiteralResource};

#[test]
fn never_initialized_or_cleared_private_targets_remain_rejected() {
    for source in [
        b"main(){a[0]=1;}".as_slice(),
        b"main(){if(1){a=[];}a[0]=1;}",
    ] {
        let error = module_compile_support::compile(
            source,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect_err("Target needs existing initialized local authority");
        assert_eq!(
            error.cause.kind,
            FunctionCompileErrorKind::UninitializedLocal
        );
    }
}

#[test]
fn literal_clear_cannot_bypass_literal_decoding_admission() {
    for rhs in ["undefined", "(undefined)"] {
        let source = format!("main(){{a=[];a[0]={rhs};return 0;}}");
        let mut budget = ModuleCompilationBudget::default();
        budget.function.literal.max_encoded_bytes = 8;
        let error = module_compile_support::compile(
            source.as_bytes(),
            LegacyDeveloperMode::Disabled,
            budget,
        )
        .expect_err("Undefined spelling is nine bytes");
        assert_eq!(
            error.cause.kind,
            FunctionCompileErrorKind::Literal(LegacyLiteralError::BudgetExceeded(
                LegacyLiteralResource::EncodedBytes
            ))
        );
    }
}

#[test]
fn indexed_assignment_and_literal_clear_share_all_compiler_budgets() {
    let source = b"main(){a=[];a[0]=5;if(1){b=a;}a[1]=(undefined);a[0]=undefined;return a.size;}";
    let compile = |function| {
        module_compile_support::compile(
            source,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget {
                function,
                ..ModuleCompilationBudget::default()
            },
        )
    };
    let usage = compile(FunctionLoweringBudget::default())
        .expect("Baseline")
        .functions()[0]
        .usage();
    let exact = FunctionLoweringBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_task_frames: usage.peak_task_frames,
        max_metadata_bytes: usage.peak_metadata_bytes,
        max_local_bindings: usage.local_bindings,
        max_identifier_bytes: usage.identifier_bytes,
        max_control_bytes: usage.peak_control_bytes,
        ..FunctionLoweringBudget::default()
    };
    assert_eq!(
        compile(exact).expect("All exact scopes").functions()[0].usage(),
        usage
    );
    for maximum in 0..usage.compiler_steps {
        compile(FunctionLoweringBudget {
            max_steps: maximum,
            ..exact
        })
        .expect_err("Every shorter planning/name/literal/store prefix");
    }
    for resource in 0..6 {
        let mut short = exact;
        match resource {
            0 => short.max_instructions -= 1,
            1 => short.max_task_frames -= 1,
            2 => short.max_metadata_bytes -= 1,
            3 => short.max_local_bindings -= 1,
            4 => short.max_identifier_bytes -= 1,
            _ => short.max_control_bytes -= 1,
        }
        compile(short).expect_err("One-less compiler resource");
    }
}
