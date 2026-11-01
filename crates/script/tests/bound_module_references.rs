// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile finite valid synthetic source modules."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn local_references_copy_return_compare_and_invoke_through_source() {
    for (text, expected) in [
        (
            "main(){f=choose();return [[f]](7);}choose(){return ::TwIcE;}twice(n){return n*2;}",
            14,
        ),
        ("main(){return [[::twice]](9);}twice(n){return n*2;}", 18),
        (
            "main(){return apply(::twice,11);}apply(f,n){return [[f]](n);}twice(n){return n*2;}",
            22,
        ),
        ("main(){f=::main;return f==::MaIn;}", 1),
        ("main(){return ::main!=::other;}other(){}", 1),
        (
            "main(){return [[choose()]]();}choose(){return ::leaf;}leaf(){return 23;}",
            23,
        ),
    ] {
        let module = module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Bound references");
        assert_eq!(
            module
                .program()
                .execute(0, &[], KernelCallBudget::default())
                .expect("Indirect result")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected)),
            "{text}"
        );
    }
}

#[test]
fn indirect_arguments_run_rightmost_first_before_the_callee_expression() {
    let text = b"main(){return [[bad()]](left(),right());}bad(){return 1/0;}left(){return 2/0;}right(){return 3/0;}";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Ordered indirect call");
    let error = module
        .program()
        .execute(0, &[], KernelCallBudget::default())
        .expect_err("Right argument fails first");
    assert_eq!(error.function, Some(3));
    let span = module
        .instruction_span(3, error.cause.instruction.expect("Error PC"))
        .expect("Mapped error");
    assert_eq!(
        &text[usize::try_from(span.start).expect("Start")..usize::try_from(span.end).expect("End")],
        b"/"
    );
}

#[test]
fn nonfunction_callees_and_missing_reference_arguments_fail_at_invocation() {
    for text in ["main(){return [[7]]();}", "main(f){return [[f]]();}"] {
        let module = module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Dynamic type checked at execution");
        let error = module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect_err("Not a function");
        assert_eq!(error.cause.kind, KernelErrorKind::ExpectedFunctionReference);
    }
}

#[test]
fn reference_binding_preserves_unknown_external_and_developer_restrictions() {
    for (text, mode, expected) in [
        (
            "main(){return ::missing;}",
            LegacyDeveloperMode::Disabled,
            FunctionCompileErrorKind::UnknownFunction,
        ),
        (
            "main(){return external::helper;}",
            LegacyDeveloperMode::Disabled,
            FunctionCompileErrorKind::UnsupportedCall,
        ),
        (
            "main(){return ::helper;}/# helper(){} #/",
            LegacyDeveloperMode::Enabled,
            FunctionCompileErrorKind::DeveloperFunctionReference,
        ),
        (
            "main(){return ::helper;}/# helper(){} #/",
            LegacyDeveloperMode::Disabled,
            FunctionCompileErrorKind::UnknownFunction,
        ),
    ] {
        let error = module_compile_support::compile(
            text.as_bytes(),
            mode,
            ModuleCompilationBudget::default(),
        )
        .expect_err("Reference cannot bypass symbol policy");
        assert_eq!(error.cause.kind, expected, "{text}");
    }
}
