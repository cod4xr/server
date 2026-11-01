// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure exact decoded/compiler/module/proof admission."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::literal::{LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource};

#[test]
fn decoded_source_literal_admission_uses_exact_bytes_instead_of_encoded_escape_length() {
    let text = br#"main(){return "\n\n";}"#;
    let mut exact = ModuleCompilationBudget::default();
    exact.function.literal = LegacyLiteralBudget {
        max_encoded_bytes: 6,
        max_string_bytes: 2,
    };
    let module = module_compile_support::compile(text, LegacyDeveloperMode::Disabled, exact)
        .expect("Exact decoded two-byte payload");
    assert_eq!(module.usage().string_literals, 1);
    assert_eq!(
        module.usage().staged_string_bytes,
        size_of::<Vec<u8>>() as u64 + 2
    );
    for (encoded, decoded, resource) in [
        (5, 2, LegacyLiteralResource::EncodedBytes),
        (6, 1, LegacyLiteralResource::StringBytes),
    ] {
        let mut budget = exact;
        budget.function.literal = LegacyLiteralBudget {
            max_encoded_bytes: encoded,
            max_string_bytes: decoded,
        };
        let error = module_compile_support::compile(text, LegacyDeveloperMode::Disabled, budget)
            .expect_err("One less per-token resource");
        assert_eq!(
            error.cause.kind,
            FunctionCompileErrorKind::Literal(LegacyLiteralError::BudgetExceeded(resource))
        );
    }
    let mut empty = exact;
    empty.function.literal.max_string_bytes = 0;
    let module = module_compile_support::compile(
        br#"main(){return "";}"#,
        LegacyDeveloperMode::Disabled,
        empty,
    )
    .expect("Empty payload still retains one row");
    assert_eq!(
        module.usage().staged_string_bytes,
        size_of::<Vec<u8>>() as u64
    );
}

#[test]
fn source_tables_pay_complete_staging_and_separate_proof_budgets_at_exact_boundaries() {
    let text = br#"main(){return helper("a\n","bbb");}helper(a,b){return a+b+"ccc";}"#;
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Measured compiler scopes");
    let usage = module.usage();
    assert_eq!(usage.string_literals, 3);
    assert_eq!(
        usage.staged_string_bytes,
        3 * size_of::<Vec<u8>>() as u64 + 8
    );
    let mut exact = ModuleCompilationBudget {
        max_steps: usage.compiler_steps,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..ModuleCompilationBudget::default()
    };
    exact.verification.max_steps = usage.verification.verification_steps;
    exact.verification.max_metadata_bytes = usage.verification.verification_metadata_bytes;
    assert_eq!(
        module_compile_support::compile(text, LegacyDeveloperMode::Disabled, exact)
            .expect("Exact all scopes")
            .usage(),
        usage
    );
    for resource in 0..4 {
        let mut budget = exact;
        match resource {
            0 => budget.max_steps -= 1,
            1 => budget.max_metadata_bytes -= 1,
            2 => budget.verification.max_steps -= 1,
            _ => budget.verification.max_metadata_bytes -= 1,
        }
        let error = module_compile_support::compile(text, LegacyDeveloperMode::Disabled, budget)
            .expect_err("One less independent complete scope");
        assert!(matches!(
            error.cause.kind,
            FunctionCompileErrorKind::ModuleBudgetExceeded { .. }
                | FunctionCompileErrorKind::Kernel(_)
        ));
        assert!(error.cause.work_steps <= budget.max_steps);
    }
}
