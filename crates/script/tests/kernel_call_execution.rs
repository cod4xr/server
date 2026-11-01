// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests use finite valid internal call inventories and deliberate runtime failures."
)]

use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelInstruction, KernelValue};
use cod4x_script::legacy::{
    LegacyArithmetic, LegacyComparison, LegacyNumber, LegacyNumberError, LegacyVector,
};

#[test]
fn forward_calls_preserve_parameter_order_caller_operands_and_owned_inventory() {
    let mut root = vec![
        integer(99),
        integer(2),
        integer(7),
        KernelInstruction::Call {
            function: 1,
            arguments: 2,
        },
        KernelInstruction::Arithmetic(LegacyArithmetic::Subtract),
        KernelInstruction::Return,
    ];
    let mut callee = vec![
        KernelInstruction::LoadLocal(0),
        integer(10),
        KernelInstruction::Arithmetic(LegacyArithmetic::Multiply),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[definition(&root, 0, 0), definition(&callee, 2, 2)]);
    root.clear();
    callee.clear();
    let outcome = module
        .execute(0, &[], KernelCallBudget::default())
        .expect("Caller resumes with operands");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(27))
    );
    assert_eq!(outcome.usage.peak_frames, 2);
    assert_eq!(outcome.usage.steps, 12);
    let args = [
        KernelValue::Number(LegacyNumber::Integer(1)),
        KernelValue::Number(LegacyNumber::Integer(9)),
    ];
    assert_eq!(
        module
            .execute(1, &args, KernelCallBudget::default())
            .expect("Source-ordered entry arguments")
            .value,
        KernelValue::Number(LegacyNumber::Integer(19))
    );
    assert!(!format!("{module:?}").contains("Constant"));
}

#[test]
fn missing_argument_suffixes_and_copied_vector_parameters_remain_private() -> Result<(), String> {
    let root = [
        integer(8),
        KernelInstruction::Call {
            function: 1,
            arguments: 1,
        },
        KernelInstruction::Return,
    ];
    let callee = [
        integer(1),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Compare(LegacyComparison::Equal),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::MakeVector,
        KernelInstruction::Return,
    ];
    let identity = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::StoreLocal(1),
        integer(17),
        KernelInstruction::StoreLocal(0),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[
        definition(&root, 0, 0),
        definition(&callee, 2, 2),
        definition(&identity, 1, 2),
    ]);
    assert_eq!(
        module
            .execute(0, &[], KernelCallBudget::default())
            .expect("Missing parameter is initialized Undefined")
            .value,
        KernelValue::Vector(LegacyVector::from_components([8.0, 1.0, 1.0]))
    );
    let input = LegacyVector::from_components([f32::from_bits(0x7f80_0001), -0.0, 3.0]);
    let arguments = [KernelValue::Vector(input)];
    for _ in 0..2 {
        let KernelValue::Vector(returned) = module
            .execute(2, &arguments, KernelCallBudget::default())
            .expect("Copied private parameter")
            .value
        else {
            return Err("Expected vector return".into());
        };
        assert_eq!(
            returned.components().map(f32::to_bits),
            input.components().map(f32::to_bits)
        );
    }
    let KernelValue::Vector(original) = arguments[0] else {
        return Err("Expected unchanged vector argument".into());
    };
    assert_eq!(
        original.components().map(f32::to_bits),
        input.components().map(f32::to_bits)
    );
    Ok(())
}

#[test]
fn callee_failures_are_located_and_private_invocations_recover_cleanly() {
    let root = [
        KernelInstruction::Call {
            function: 1,
            arguments: 0,
        },
        KernelInstruction::Return,
    ];
    let failing = [
        integer(17379),
        integer(0),
        KernelInstruction::Divide,
        KernelInstruction::Return,
    ];
    let healthy = [integer(5), KernelInstruction::Return];
    let module = kernel_call_support::verify(&[
        definition(&root, 0, 0),
        definition(&failing, 0, 0),
        definition(&healthy, 0, 0),
    ]);
    let error = module
        .execute(0, &[], KernelCallBudget::default())
        .expect_err("Callee fails");
    assert_eq!(error.function, Some(1));
    assert_eq!(error.cause.instruction, Some(2));
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    assert_eq!(error.cause.work_steps, 4);
    assert_eq!(error.peak_frames, 2);
    assert!(!format!("{error:?} {error}").contains("17379"));
    assert_eq!(
        module
            .execute(0, &[], KernelCallBudget::default())
            .expect_err("Fresh failing invocation"),
        error
    );
    assert_eq!(
        module
            .execute(2, &[], KernelCallBudget::default())
            .expect("Independent healthy invocation")
            .value,
        KernelValue::Number(LegacyNumber::Integer(5))
    );
}

#[test]
fn missing_callee_parameters_do_not_initialize_nonparameter_locals() {
    let bad_root = [
        KernelInstruction::Call {
            function: 1,
            arguments: 0,
        },
        KernelInstruction::Return,
    ];
    let uninitialized = [KernelInstruction::LoadLocal(1), KernelInstruction::Return];
    let parameter = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let good_root = [
        KernelInstruction::Call {
            function: 2,
            arguments: 0,
        },
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[
        definition(&bad_root, 0, 0),
        definition(&uninitialized, 1, 2),
        definition(&parameter, 1, 2),
        definition(&good_root, 0, 0),
    ]);
    let error = module
        .execute(0, &[], KernelCallBudget::default())
        .expect_err("Only missing parameter prefix is initialized");
    assert_eq!(error.function, Some(1));
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::UninitializedLocal { slot: 1 }
    );
    assert_eq!(error.cause.instruction, Some(0));
    assert_eq!(error.cause.work_steps, 2);
    assert_eq!(error.cause.admitted_local_values, 2);
    assert_eq!(
        module
            .execute(3, &[], KernelCallBudget::default())
            .expect("Missing parameter itself is initialized")
            .value,
        KernelValue::Undefined
    );
}
