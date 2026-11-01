// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite typed values exercise strict integer admission and preserved words."
)]

use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn integer_admission_preserves_all_word_boundaries_without_encoded_id_limits() {
    for value in [
        i32::MIN,
        -8_388_608,
        -1,
        0,
        1,
        8_388_607,
        8_388_608,
        i32::MAX,
    ] {
        let proof = VerifiedKernelProgram::verify(
            &[
                integer(value),
                KernelInstruction::RequireInteger,
                KernelInstruction::Return,
            ],
            KernelVerificationBudget::default(),
        )
        .expect("Unary integer proof");
        let result = proof
            .execute(KernelExecutionBudget::default())
            .expect("Preserved integer");
        assert_eq!(
            result.value,
            KernelValue::Number(LegacyNumber::Integer(value))
        );
        assert_eq!(result.steps, 3);
    }
    assert_eq!(
        VerifiedKernelProgram::verify(
            &[KernelInstruction::RequireInteger, KernelInstruction::Return],
            KernelVerificationBudget::default()
        )
        .expect_err("Unary operand required")
        .kind,
        KernelErrorKind::StackUnderflow {
            required: 1,
            available: 0
        }
    );
}

#[test]
fn integer_admission_rejects_other_types_without_dereferencing_function_owners() {
    let leaf = [integer(0), KernelInstruction::Return];
    let module = kernel_call_support::verify(&[definition(&leaf, 0, 0)]);
    let reference = module.function_reference(0).expect("Opaque reference");
    drop(module);
    for value in [
        KernelValue::Undefined,
        KernelValue::Vector(LegacyVector::from_components([0.0; 3])),
        KernelValue::Function(reference),
        KernelValue::Number(LegacyNumber::Float(1.0)),
        KernelValue::Number(LegacyNumber::Float(-0.0)),
        KernelValue::Number(LegacyNumber::Float(f32::NAN)),
        KernelValue::Number(LegacyNumber::Float(f32::INFINITY)),
    ] {
        let proof = VerifiedKernelProgram::verify(
            &[
                KernelInstruction::Constant(value),
                KernelInstruction::RequireInteger,
                KernelInstruction::Return,
            ],
            KernelVerificationBudget::default(),
        )
        .expect("Dynamic value proof");
        let error = proof
            .execute(KernelExecutionBudget::default())
            .expect_err("No implicit conversion");
        assert_eq!(error.kind, KernelErrorKind::ExpectedInteger);
        assert_eq!(error.instruction, Some(1));
        assert_eq!(error.work_steps, 2);
    }
}
