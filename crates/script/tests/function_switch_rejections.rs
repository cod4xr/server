// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Synthetic source deliberately exercises typed switch rejection paths."
)]

use crate::{function_compile_support, module_compile_support};
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompileErrorKind, ModuleCompilationBudget,
};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn duplicate_misplaced_unsupported_and_unreachable_switch_source_fails_explicitly() {
    for (text, expected) in [
        (
            "main(n){switch(n){case true:break;}}",
            FunctionCompileErrorKind::UnsupportedCase,
        ),
        (
            "main(n){switch(n){case false:break;}}",
            FunctionCompileErrorKind::UnsupportedCase,
        ),
        (
            "main(n){switch(n){case (1):break;}}",
            FunctionCompileErrorKind::UnsupportedCase,
        ),
        (
            "main(n){switch(n){case 1:break;case 1:break;}}",
            FunctionCompileErrorKind::DuplicateCase,
        ),
        (
            "main(n){switch(n){case 0:break;case -0:break;}}",
            FunctionCompileErrorKind::DuplicateCase,
        ),
        (
            "main(n){switch(n){default:break;default:break;}}",
            FunctionCompileErrorKind::DuplicateCase,
        ),
        (
            "main(){case 1:return 0;}",
            FunctionCompileErrorKind::IllegalCase,
        ),
        (
            "main(){default:return 0;}",
            FunctionCompileErrorKind::IllegalCase,
        ),
        (
            "main(n){switch(n){case 1:{case 2:break;}}}",
            FunctionCompileErrorKind::IllegalCase,
        ),
        (
            "main(n){switch(n){case 1.0:break;}}",
            FunctionCompileErrorKind::UnsupportedCase,
        ),
        (
            "main(n){switch(n){case \"PrivateLabel\":break;}}",
            FunctionCompileErrorKind::UnsupportedCase,
        ),
        (
            "main(n){switch(n){case undefined:break;}}",
            FunctionCompileErrorKind::UnsupportedCase,
        ),
        (
            "main(n){switch(n){x=1;case 1:break;}}",
            FunctionCompileErrorKind::UnsupportedBody,
        ),
        (
            "main(n){switch(n){case 1:continue;}}",
            FunctionCompileErrorKind::IllegalContinue,
        ),
        (
            "main(n){switch(n){case 1:break;x=1;}}",
            FunctionCompileErrorKind::UnreachableStatement,
        ),
        (
            "main(n){switch(n){case 1:return 1;x=1;}}",
            FunctionCompileErrorKind::ReturnNotLast,
        ),
        (
            "main(n){switch(n){case 1:return 1;default:return 2;}return 3;}",
            FunctionCompileErrorKind::UnreachableStatement,
        ),
    ] {
        let error = module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect_err("Explicit primitive switch policy");
        assert_eq!(error.cause.kind, expected, "{text}");
        assert!(!format!("{error:?} {error}").contains("PrivateLabel"));
    }
    assert!(
        function_compile_support::compile_scalar(
            b"main(){switch(1){default:return 1;}}",
            FunctionCompilationBudget::default()
        )
        .is_err()
    );
}

#[test]
fn switch_key_type_errors_preserve_caller_source_maps_and_failed_work() {
    let text = b"PrivateMain(n){switch(n){case 1:return 7;default:return 9;}}";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Dynamic switched value");
    for value in [
        KernelValue::Undefined,
        KernelValue::Number(LegacyNumber::Float(1.0)),
        KernelValue::Vector(LegacyVector::from_components([1.0; 3])),
        KernelValue::Function(
            module
                .program()
                .function_reference(0)
                .expect("Function value"),
        ),
    ] {
        let error = module
            .program()
            .execute(0, &[value], KernelCallBudget::default())
            .expect_err("Strict integer key");
        assert_eq!(error.cause.kind, KernelErrorKind::ExpectedInteger);
        assert_eq!(error.function, Some(0));
        assert_eq!(error.cause.work_steps, 2);
        assert_eq!(
            module.instruction_span(0, error.cause.instruction.expect("Require PC")),
            Some(15..u64::try_from(text.len() - 1).expect("Closing function brace"))
        );
        assert!(!format!("{error:?} {error}").contains("PrivateMain"));
    }
}
