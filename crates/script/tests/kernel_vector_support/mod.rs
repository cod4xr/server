// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{
    LegacyArithmetic, LegacyComparison, LegacyNumber, LegacyVector, LegacyVectorOperand,
    LegacyVectorOperation,
};

use crate::vector_support::evaluator::{EvaluationFailure, VectorEvaluator};

pub struct KernelEvaluator;

impl VectorEvaluator for KernelEvaluator {
    fn construct(values: [LegacyNumber; 3]) -> Result<LegacyVector, EvaluationFailure> {
        vector(execute(&[
            KernelInstruction::Constant(KernelValue::Number(values[2])),
            KernelInstruction::Constant(KernelValue::Number(values[1])),
            KernelInstruction::Constant(KernelValue::Number(values[0])),
            KernelInstruction::MakeVector,
            KernelInstruction::Return,
        ])?)
    }

    fn arithmetic(
        operation: LegacyVectorOperation,
        left: LegacyVectorOperand,
        right: LegacyVectorOperand,
    ) -> Result<LegacyVector, EvaluationFailure> {
        let instruction = match operation {
            LegacyVectorOperation::Add => KernelInstruction::Arithmetic(LegacyArithmetic::Add),
            LegacyVectorOperation::Subtract => {
                KernelInstruction::Arithmetic(LegacyArithmetic::Subtract)
            }
            LegacyVectorOperation::Multiply => {
                KernelInstruction::Arithmetic(LegacyArithmetic::Multiply)
            }
            LegacyVectorOperation::Divide => KernelInstruction::Divide,
        };
        vector(execute(&[
            constant(left),
            constant(right),
            instruction,
            KernelInstruction::Return,
        ])?)
    }

    fn compare(
        operation: LegacyComparison,
        left: LegacyVectorOperand,
        right: LegacyVectorOperand,
    ) -> Result<bool, EvaluationFailure> {
        match execute(&[
            constant(left),
            constant(right),
            KernelInstruction::Compare(operation),
            KernelInstruction::Return,
        ])? {
            KernelValue::Number(LegacyNumber::Integer(value @ (0 | 1))) => Ok(value == 1),
            _ => Err(unexpected("Kernel vector comparison returned another type")),
        }
    }
}

fn constant(value: LegacyVectorOperand) -> KernelInstruction {
    KernelInstruction::Constant(match value {
        LegacyVectorOperand::Number(value) => KernelValue::Number(value),
        LegacyVectorOperand::Vector(value) => KernelValue::Vector(value),
    })
}

fn execute(code: &[KernelInstruction]) -> Result<KernelValue, EvaluationFailure> {
    let program = VerifiedKernelProgram::verify(code, KernelVerificationBudget::default())
        .map_err(|error| unexpected(&error.to_string()))?;
    match program.execute(KernelExecutionBudget::default()) {
        Ok(outcome) => Ok(outcome.value),
        Err(error) => match error.kind {
            KernelErrorKind::Vector(error) => Err(EvaluationFailure::Domain(error)),
            _ => Err(unexpected(&error.to_string())),
        },
    }
}

fn vector(value: KernelValue) -> Result<LegacyVector, EvaluationFailure> {
    match value {
        KernelValue::Vector(value) => Ok(value),
        _ => Err(unexpected("Kernel returned another type")),
    }
}

fn unexpected(message: &str) -> EvaluationFailure {
    EvaluationFailure::Unexpected(message.to_owned())
}
