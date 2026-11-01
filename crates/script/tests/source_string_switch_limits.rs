// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Source label decoding, sorting and proof copies have exact independent resource scopes."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::literal::{LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource};
use std::fmt::Write;

#[test]
fn switch_label_decoding_charges_encoded_and_decoded_bytes_without_runtime_literal_rows() {
    let source = br#"main(k){switch(k){case "\n\n":return 1;default:return 0;}}"#;
    let mut exact = ModuleCompilationBudget::default();
    exact.function.literal = LegacyLiteralBudget {
        max_encoded_bytes: 6,
        max_string_bytes: 2,
    };
    let module = module_compile_support::compile(source, Disabled, exact).expect("Exact decoding");
    assert_eq!(
        (module.usage().string_literals, module.usage().switch_labels),
        (0, 1)
    );
    assert_eq!(
        module.usage().staged_switch_bytes,
        size_of::<Vec<u8>>() as u64 + 2
    );
    for (encoded, decoded, resource) in [
        (5, 2, LegacyLiteralResource::EncodedBytes),
        (6, 1, LegacyLiteralResource::StringBytes),
    ] {
        let mut short = exact;
        short.function.literal = LegacyLiteralBudget {
            max_encoded_bytes: encoded,
            max_string_bytes: decoded,
        };
        let error = module_compile_support::compile(source, Disabled, short)
            .expect_err("One less token quota");
        assert_eq!(
            error.cause.kind,
            FunctionCompileErrorKind::Literal(LegacyLiteralError::BudgetExceeded(resource))
        );
    }
}

#[test]
fn source_label_staging_sorting_and_proof_share_global_prefix_and_individual_function_budgets() {
    let source = br#"main(k){switch(k){case "zz":return 1;case -1:return 2;case "aa":return 3;default:return other(k);}}other(k){switch(k){case "bb":return 4;default:return 5;}}"#;
    let module =
        module_compile_support::compile(source, Disabled, ModuleCompilationBudget::default())
            .expect("Baseline");
    let usage = module.usage();
    assert_eq!(
        (usage.switch_labels, usage.verification.switch_labels),
        (3, 3)
    );
    assert_eq!(
        usage.staged_switch_bytes,
        3 * size_of::<Vec<u8>>() as u64 + 6
    );
    assert_eq!(usage.string_literals, 0);
    let mut exact = ModuleCompilationBudget {
        max_steps: usage.compiler_steps,
        max_instructions: usage.instructions,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..ModuleCompilationBudget::default()
    };
    exact.verification.max_steps = usage.verification.verification_steps;
    exact.verification.max_metadata_bytes = usage.verification.verification_metadata_bytes;
    let functions: Vec<_> = module
        .functions()
        .iter()
        .map(cod4x_script::compilation::CompiledModuleFunction::usage)
        .collect();
    exact.function.max_steps = functions
        .iter()
        .map(|item| item.compiler_steps)
        .max()
        .expect("Function");
    exact.function.max_instructions = functions
        .iter()
        .map(|item| item.instructions)
        .max()
        .expect("Code");
    exact.function.max_metadata_bytes = functions
        .iter()
        .map(|item| item.peak_metadata_bytes)
        .max()
        .expect("Tables");
    exact.function.max_control_bytes = functions
        .iter()
        .map(|item| item.peak_control_bytes)
        .max()
        .expect("Cases");
    exact.function.max_task_frames = functions
        .iter()
        .map(|item| item.peak_task_frames)
        .max()
        .expect("Tasks");
    assert_eq!(
        module_compile_support::compile(source, Disabled, exact)
            .expect("All exact")
            .usage(),
        usage
    );
    for resource in 0..10 {
        let mut short = exact;
        match resource {
            0 => short.max_steps -= 1,
            1 => short.max_instructions -= 1,
            2 => short.max_metadata_bytes -= 1,
            3 => short.verification.max_steps -= 1,
            4 => short.verification.max_metadata_bytes -= 1,
            5 => short.function.max_steps -= 1,
            6 => short.function.max_instructions -= 1,
            7 => short.function.max_metadata_bytes -= 1,
            8 => short.function.max_control_bytes -= 1,
            _ => short.function.max_task_frames -= 1,
        }
        let error = module_compile_support::compile(source, Disabled, short)
            .expect_err("One less independent scope");
        assert!(error.cause.work_steps <= short.max_steps);
    }
    for maximum in 0..usage.compiler_steps {
        let short = ModuleCompilationBudget {
            max_steps: maximum,
            ..exact
        };
        let error = module_compile_support::compile(source, Disabled, short)
            .expect_err("Every shorter compiler prefix");
        assert!(error.cause.work_steps <= maximum);
    }
}

#[test]
fn sorting_precharges_complete_common_prefix_bytes_before_fallible_comparisons() {
    fn source(prefix: usize) -> Vec<u8> {
        let mut source = String::from("main(k){switch(k){");
        for suffix in ["d", "b", "c", "a"] {
            write!(source, "case \"{}{suffix}\":break;", "x".repeat(prefix)).expect("Source case");
        }
        source.push_str("}return 0;}");
        source.into_bytes()
    }
    let short =
        module_compile_support::compile(&source(1), Disabled, ModuleCompilationBudget::default())
            .expect("Short");
    let long_source = source(1000);
    let long =
        module_compile_support::compile(&long_source, Disabled, ModuleCompilationBudget::default())
            .expect("Long");
    // Decoding alone costs three encoded-byte passes; sorting additionally pays
    // the common prefix for every comparison rather than charging only key IDs.
    assert!(long.usage().compiler_steps - short.usage().compiler_steps > 4 * 999 * 3);
    let budget = ModuleCompilationBudget {
        max_steps: short.usage().compiler_steps,
        ..ModuleCompilationBudget::default()
    };
    module_compile_support::compile(&long_source, Disabled, budget)
        .expect_err("Long bytes cannot use short-key work");
}
