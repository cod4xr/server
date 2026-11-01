// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Synthetic builtin calls exercise deliberate runtime/compiler failures."
)]

use crate::{function_compile_support, module_compile_support};
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompileErrorKind, ModuleCompilationBudget,
    ModuleCompilationResource,
};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn wrong_builtin_counts_fail_only_on_selected_calls_after_argument_evaluation() {
    let module = module_compile_support::compile(
        b"main(flag){if(flag){return isint(1,2);}return 7;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Skipped arity valid");
    assert_eq!(
        module
            .program()
            .execute(
                0,
                &[KernelValue::Number(LegacyNumber::Integer(0))],
                KernelCallBudget::default()
            )
            .expect("Skipped wrong count")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
    let error = module
        .program()
        .execute(
            0,
            &[KernelValue::Number(LegacyNumber::Integer(1))],
            KernelCallBudget::default(),
        )
        .expect_err("Executed wrong count");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 1,
            actual: 2
        }
    );
    assert_eq!(error.peak_frames, 1);
    let text = b"main(){return isint(left(),right());}left(){return 1/0;}right(){return 2/0;}";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Dynamic argument errors");
    let error = module
        .program()
        .execute(0, &[], KernelCallBudget::default())
        .expect_err("Rightmost fails before arity");
    assert_eq!(error.function, Some(2));
    assert!(matches!(error.cause.kind, KernelErrorKind::Numeric(_)));
}

#[test]
fn builtin_source_spans_privacy_quotas_and_counts_do_not_inherit_native_byte_limits() {
    let text = format!(
        "PrivateMain(){{return isint({});}}",
        vec!["1"; 256].join(",")
    );
    let module = module_compile_support::compile(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("More than 255 builtin arguments admitted");
    let error = module
        .program()
        .execute(0, &[], KernelCallBudget::default())
        .expect_err("Wrong count at invocation");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 1,
            actual: 256
        }
    );
    assert_eq!(error.cause.work_steps, 257);
    let span = module
        .instruction_span(0, error.cause.instruction.expect("Call PC"))
        .expect("Source map");
    let start = usize::try_from(span.start).expect("Start");
    let end = usize::try_from(span.end).expect("End");
    assert!(text[start..end].starts_with("isint("));
    assert!(!format!("{error:?} {error}").contains("PrivateMain"));
    let mut budget = ModuleCompilationBudget::default();
    budget.function.max_call_arguments = 255;
    assert_eq!(
        module_compile_support::compile(text.as_bytes(), LegacyDeveloperMode::Disabled, budget)
            .expect_err("Per-call quota still enforced")
            .cause
            .kind,
        FunctionCompileErrorKind::ModuleBudgetExceeded {
            resource: ModuleCompilationResource::CallArguments,
            requested: 256,
            maximum: 255
        }
    );
}

#[test]
fn isolated_profiles_and_uninitialized_arguments_keep_their_contract() {
    let text = b"main(){return isint(1);}";
    assert!(
        function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
            .is_err()
    );
    assert!(
        function_compile_support::compile_scalar(text, FunctionCompilationBudget::default())
            .is_err()
    );
    let error = module_compile_support::compile(
        b"main(){return isint(PrivateMissing);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect_err("Builtin arguments still require initialized reads");
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::UninitializedLocal
    );
}
