// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Discarded code must pay peak temporary compiler admission without retaining code or tables."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode::{Disabled, Enabled};
use cod4x_script::compilation::{
    CompiledModule, FunctionCompileErrorKind, FunctionLoweringBudget, ModuleCompilationBudget,
};
use cod4x_script::literal::{LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource};

pub const SOURCE: &[u8] = br#"main(){a=[];a[0]=1;/# b=a;b[1]=2;o=spawnstruct();o.text="xx";
    switch("zz"){case "zz":b[2]=3;break;} #/return helper(a.size);}helper(n){return n;}"#;

fn exact(module: &CompiledModule) -> ModuleCompilationBudget {
    let usage = module.usage();
    let main = module.functions()[0].usage();
    let mut budget = ModuleCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.peak_instructions,
        max_metadata_bytes: usage.peak_metadata_bytes,
        function: FunctionLoweringBudget {
            max_steps: main.compiler_steps,
            max_instructions: main.peak_instructions,
            max_task_frames: main.peak_task_frames,
            max_metadata_bytes: main.peak_metadata_bytes,
            max_local_bindings: main.local_bindings,
            max_identifier_bytes: main.identifier_bytes,
            max_control_bytes: main.peak_control_bytes,
            ..FunctionLoweringBudget::default()
        },
        ..ModuleCompilationBudget::default()
    };
    budget.verification.max_steps = usage.verification.verification_steps;
    budget.verification.max_metadata_bytes = usage.verification.verification_metadata_bytes;
    budget
}

#[test]
fn discarded_code_and_tables_pay_peak_function_global_and_proof_budgets_at_exact_boundaries() {
    for mode in [Disabled, Enabled] {
        let module =
            module_compile_support::compile(SOURCE, mode, ModuleCompilationBudget::default())
                .expect("Baseline");
        let usage = module.usage();
        if mode == Disabled {
            assert!(usage.peak_instructions > usage.instructions);
            assert_eq!(
                (
                    usage.string_literals,
                    usage.field_names,
                    usage.switch_labels
                ),
                (0, 0, 0)
            );
        }
        let exact = exact(&module);
        assert_eq!(
            module_compile_support::compile(SOURCE, mode, exact)
                .expect("All exact")
                .usage(),
            usage
        );
        for resource in 0..12 {
            let mut short = exact;
            match resource {
                0 => short.max_steps -= 1,
                1 => short.max_instructions -= 1,
                2 => short.max_metadata_bytes -= 1,
                3 => short.function.max_steps -= 1,
                4 => short.function.max_instructions -= 1,
                5 => short.function.max_task_frames -= 1,
                6 => short.function.max_metadata_bytes -= 1,
                7 => short.function.max_local_bindings -= 1,
                8 => short.function.max_identifier_bytes -= 1,
                9 => short.function.max_control_bytes -= 1,
                10 => short.verification.max_steps -= 1,
                _ => short.verification.max_metadata_bytes -= 1,
            }
            module_compile_support::compile(SOURCE, mode, short)
                .expect_err("One less independent scope");
        }
        for maximum in 0..usage.compiler_steps {
            let short = ModuleCompilationBudget {
                max_steps: maximum,
                ..exact
            };
            let error = module_compile_support::compile(SOURCE, mode, short)
                .expect_err("Every shorter global compiler prefix");
            assert!(error.cause.work_steps <= maximum);
        }
    }
}

#[test]
fn discarded_literals_and_case_labels_cannot_bypass_encoded_or_decoded_token_limits() {
    let source = br#"main(){/# switch(0){case "\n\n":break;} #/return 0;}"#;
    for mode in [Disabled, Enabled] {
        let mut budget = ModuleCompilationBudget::default();
        budget.function.literal = LegacyLiteralBudget {
            max_encoded_bytes: 6,
            max_string_bytes: 2,
        };
        module_compile_support::compile(source, mode, budget).expect("Exact literal quota");
        for (encoded, decoded, resource) in [
            (5, 2, LegacyLiteralResource::EncodedBytes),
            (6, 1, LegacyLiteralResource::StringBytes),
        ] {
            budget.function.literal = LegacyLiteralBudget {
                max_encoded_bytes: encoded,
                max_string_bytes: decoded,
            };
            let error = module_compile_support::compile(source, mode, budget)
                .expect_err("Discard still validates complete literals");
            assert_eq!(
                error.cause.kind,
                FunctionCompileErrorKind::Literal(LegacyLiteralError::BudgetExceeded(resource))
            );
        }
    }
}
