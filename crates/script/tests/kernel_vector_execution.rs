// SPDX-License-Identifier: AGPL-3.0-or-later
//! Inline vector construction, local copies and rejected types retain scalar proof/fuel rules.
#![allow(
    clippy::expect_used,
    reason = "Tests own finite typed instructions and exact expected failures."
)]

use crate::kernel_support;
use cod4x_script::execution::{
    KernelBranchCondition, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{
    LegacyIntegerOperator, LegacyIntegerUpdate, LegacyNumber, LegacyVector,
};

#[test]
fn vector_constructor_order_and_local_copies_preserve_bits_and_caller_isolation()
-> Result<(), String> {
    let input = LegacyVector::from_components([f32::from_bits(0x7f80_0001), -0.0, 7.0]);
    let arguments = [KernelValue::Vector(input)];
    let layout = KernelFunctionLayout::new(1, 2).expect("Layout");
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Duplicate,
        KernelInstruction::StoreLocal(1),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Return,
    ];
    let program =
        VerifiedKernelProgram::verify_function(&code, layout, KernelVerificationBudget::default())
            .expect("Local vector proof");
    for _ in 0..2 {
        let outcome = program
            .execute_with_arguments(&arguments, KernelExecutionBudget::default())
            .expect("Fresh vector invocation");
        let KernelValue::Vector(result) = outcome.value else {
            return Err("Expected vector local result".into());
        };
        assert_eq!(
            result.components().map(f32::to_bits),
            input.components().map(f32::to_bits)
        );
        assert_eq!(
            outcome.local_bytes,
            2 * size_of::<Option<KernelValue>>() as u64
        );
    }
    let KernelValue::Vector(original) = arguments[0] else {
        return Err("Caller type changed".into());
    };
    assert_eq!(
        original.components().map(f32::to_bits),
        input.components().map(f32::to_bits)
    );
    let defaults = program
        .execute_legacy_script_arguments(&[], KernelExecutionBudget::default())
        .expect("Missing parameter stays initialized Undefined");
    assert_eq!(defaults.value, KernelValue::Undefined);
    assert_eq!(format!("{arguments:?}"), "[KernelValue::Vector(..)]");
    Ok(())
}

#[test]
fn vector_components_truth_and_integer_operators_reject_unsupported_types() {
    let vector = KernelValue::Vector(LegacyVector::from_components([1.0, 2.0, 3.0]));
    for invalid in [KernelValue::Undefined, vector] {
        for index in 0..3 {
            let mut inputs = [KernelValue::Number(LegacyNumber::Integer(1)); 3];
            inputs[index] = invalid;
            let code = [
                KernelInstruction::Constant(inputs[0]),
                KernelInstruction::Constant(inputs[1]),
                KernelInstruction::Constant(inputs[2]),
                KernelInstruction::MakeVector,
                KernelInstruction::Return,
            ];
            let error = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
                .expect("Stack proof")
                .execute(KernelExecutionBudget::default())
                .expect_err("Nonnumeric component");
            assert_eq!(error.kind, KernelErrorKind::ExpectedNumber);
            assert_eq!(error.instruction, Some(3));
            assert_eq!((error.work_steps, error.peak_stack_values), (4, 3));
        }
    }
    for operation in [
        KernelInstruction::CastBool,
        KernelInstruction::LogicalNot,
        KernelInstruction::BitwiseNot,
        KernelInstruction::IntegerUpdate(LegacyIntegerUpdate::Increment),
    ] {
        let code = [
            KernelInstruction::Constant(vector),
            operation,
            KernelInstruction::Return,
        ];
        assert_eq!(
            VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
                .expect("Unary proof")
                .execute(KernelExecutionBudget::default())
                .expect_err("Vector unary type")
                .kind,
            KernelErrorKind::ExpectedNumber
        );
    }
    let code = [
        KernelInstruction::Constant(vector),
        kernel_support::integer(1),
        KernelInstruction::Integer(LegacyIntegerOperator::Or),
        KernelInstruction::Return,
    ];
    assert_eq!(
        VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect("Integer proof")
            .execute(KernelExecutionBudget::default())
            .expect_err("Vector bitwise type")
            .kind,
        KernelErrorKind::ExpectedNumber
    );
    let code = [
        KernelInstruction::Constant(vector),
        KernelInstruction::Branch {
            condition: KernelBranchCondition::IfTrue,
            target: 2,
        },
        kernel_support::integer(0),
        KernelInstruction::Return,
    ];
    assert_eq!(
        VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect("Branch proof")
            .execute(KernelExecutionBudget::default())
            .expect_err("No vector truth conversion")
            .kind,
        KernelErrorKind::ExpectedNumber
    );
}
