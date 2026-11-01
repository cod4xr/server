// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise real source calls and selected type/argument failures."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};

#[test]
fn rightmost_source_argument_failure_and_name_checks_retain_function_instruction_maps() {
    let text = b"PrivateMain(a){return sink(left(a),right(a));}sink(x,y){return 0;}left(a){return 1/a;}right(a){return 2/a;}";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Ordered source calls");
    let error = module
        .program()
        .execute(
            0,
            &[KernelValue::Number(LegacyNumber::Integer(0))],
            KernelCallBudget::default(),
        )
        .expect_err("Rightmost call executes first");
    assert_eq!(error.function, Some(3));
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    let span = module
        .instruction_span(
            error.function.expect("Function"),
            error.cause.instruction.expect("PC"),
        )
        .expect("Original source map");
    let offset = u64::try_from(
        text.iter()
            .rposition(|byte| *byte == b'/')
            .expect("Rightmost function division"),
    )
    .expect("Offset");
    assert_eq!(span, offset..offset + 1);
    assert!(!format!("{error:?} {error}").contains("PrivateMain"));
    let text = b"main(){return sink(PrivateLeft,PrivateRight);}sink(a,b){return 0;}";
    let error = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect_err("Rightmost uninitialized source argument is checked first");
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::UninitializedLocal
    );
    let start = usize::try_from(error.cause.span.start).expect("Offset");
    let end = usize::try_from(error.cause.span.end).expect("Offset");
    assert_eq!(&text[start..end], b"PrivateRight");
}

#[test]
fn source_calls_preserve_missing_undefined_and_runtime_excess_error_timing() {
    let module = module_compile_support::compile(
        b"main(flag){if(flag){return callee(3,4);}return callee();}callee(a){return a==undefined;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Dynamic source arity is not a publication error");
    assert_eq!(
        module
            .program()
            .execute(
                0,
                &[KernelValue::Number(LegacyNumber::Integer(0))],
                KernelCallBudget::default()
            )
            .expect("Missing parameter is initialized Undefined")
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
        .expect_err("Selected excess source arguments fail at runtime");
    assert_eq!(error.function, Some(0));
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 1,
            actual: 2
        }
    );
    assert!(
        module
            .instruction_span(0, error.cause.instruction.expect("Call PC"))
            .is_some()
    );
    let module = module_compile_support::compile(
        b"main(){return callee(1/0);}callee(){return 0;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Failing argument remains an invocation concern");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect_err("Argument fails before count")
            .cause
            .kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
}
