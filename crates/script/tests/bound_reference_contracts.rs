// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Synthetic higher-order sources are structurally valid."
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
fn indirect_calls_preserve_missing_undefined_and_excess_runtime_arity() {
    let module = module_compile_support::compile(
        b"main(flag){f=::callee;if(flag){return [[f]](3,4);}return [[f]]();}callee(a){return a==undefined;}",
        LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("Runtime arity");
    assert_eq!(
        module
            .program()
            .execute(
                0,
                &[KernelValue::Number(LegacyNumber::Integer(0))],
                KernelCallBudget::default()
            )
            .expect("Missing argument")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
    let error = module
        .program()
        .execute(
            0,
            &[KernelValue::Number(LegacyNumber::Integer(1))],
            KernelCallBudget::default(),
        )
        .expect_err("Excess arguments");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 1,
            actual: 2
        }
    );
    assert_eq!(error.function, Some(0));
    assert_eq!(error.peak_frames, 1);
    assert!(
        module
            .instruction_span(0, error.cause.instruction.expect("Call PC"))
            .is_some()
    );
}

#[test]
fn indirect_calls_reuse_per_site_count_quotas_and_check_arguments_before_target() {
    let mut budget = ModuleCompilationBudget::default();
    budget.function.max_call_arguments = 1;
    let error = module_compile_support::compile(
        b"main(){return [[::sink]](1,2);}sink(a,b){}",
        LegacyDeveloperMode::Disabled,
        budget,
    )
    .expect_err("Indirect argument count quota");
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::ModuleBudgetExceeded {
            resource: ModuleCompilationResource::CallArguments,
            requested: 2,
            maximum: 1,
        }
    );
    let text = b"main(){return [[PrivateTarget]](PrivateLeft,PrivateRight);}";
    let error = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect_err("Rightmost argument checked first");
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::UninitializedLocal
    );
    let span = error.cause.span;
    assert_eq!(
        &text[usize::try_from(span.start).expect("Start")..usize::try_from(span.end).expect("End")],
        b"PrivateRight"
    );
}

#[test]
fn higher_order_recursion_uses_iterative_frames_and_developer_references_stay_scoped() {
    let module = module_compile_support::compile(
        b"main(){return recurse(::recurse,256);}recurse(f,n){if(n<=0){return 0;}return [[f]](f,n-1)+1;}",
        LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("Higher-order recursion");
    let outcome = module
        .program()
        .execute(0, &[], KernelCallBudget::default())
        .expect("Iterative execution");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(256))
    );
    assert_eq!(outcome.usage.peak_frames, 258);
    let module = module_compile_support::compile(
        b"/# main(){return [[::hidden]]();}hidden(){return 17;} #/",
        LegacyDeveloperMode::Enabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Developer-owned reference");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Developer target")
            .value,
        KernelValue::Number(LegacyNumber::Integer(17))
    );
}

#[test]
fn isolated_function_profiles_continue_to_reject_unbound_reference_authority() {
    for text in [
        b"main(){return ::main;}".as_slice(),
        b"main(f){return [[f]]();}",
    ] {
        assert!(
            function_compile_support::compile_primitive(text, FunctionCompilationBudget::default())
                .is_err()
        );
        assert!(
            function_compile_support::compile_scalar(text, FunctionCompilationBudget::default())
                .is_err()
        );
    }
}
