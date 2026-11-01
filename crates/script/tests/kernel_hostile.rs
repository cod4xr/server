// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic hostile typed streams cannot turn a valid proof into invalid state.
#![allow(
    clippy::expect_used,
    reason = "Tests use bounded deterministic instruction operands."
)]

use cod4x_script::execution::{
    KernelBranchCondition, KernelBuiltin, KernelErrorKind, KernelExecutionBudget,
    KernelInstruction, KernelResource, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::legacy::{
    LegacyArithmetic, LegacyComparison, LegacyIntegerOperator, LegacyIntegerUpdate, LegacyNumber,
    LegacyVector,
};

fn next(state: &mut u32) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state
}

fn instruction(state: &mut u32, count: usize) -> KernelInstruction {
    let choice = next(state) % 25;
    let operand = next(state);
    let target = usize::try_from(operand).expect("Supported test host") % (count + 3);
    match choice {
        0 => KernelInstruction::Constant(KernelValue::Undefined),
        1 => KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Integer(
            operand.cast_signed(),
        ))),
        2 => KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Float(f32::from_bits(
            operand,
        )))),
        3 => KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        4 => KernelInstruction::Arithmetic(LegacyArithmetic::Subtract),
        5 => KernelInstruction::Compare(LegacyComparison::LessOrEqual),
        6 => KernelInstruction::Integer(LegacyIntegerOperator::ShiftRight),
        7 => KernelInstruction::Divide,
        8 => KernelInstruction::CastBool,
        9 => KernelInstruction::LogicalNot,
        10 => KernelInstruction::BitwiseNot,
        11 => KernelInstruction::Duplicate,
        12 => KernelInstruction::Discard,
        13 => KernelInstruction::Jump(target),
        14 => KernelInstruction::Branch {
            condition: KernelBranchCondition::IfTrue,
            target,
        },
        15 => KernelInstruction::Branch {
            condition: KernelBranchCondition::IfFalse,
            target,
        },
        17 => KernelInstruction::IntegerUpdate(LegacyIntegerUpdate::Increment),
        18 => KernelInstruction::IntegerUpdate(LegacyIntegerUpdate::Decrement),
        19 => KernelInstruction::Constant(KernelValue::Vector(LegacyVector::from_components([
            f32::from_bits(operand),
            f32::from_bits(operand.rotate_left(7)),
            f32::from_bits(operand.rotate_right(11)),
        ]))),
        20 => KernelInstruction::MakeVector,
        21 => KernelInstruction::Index,
        22..=24 => KernelInstruction::Builtin {
            function: match choice {
                22 => KernelBuiltin::IsInt,
                23 => KernelBuiltin::IsFloat,
                _ => KernelBuiltin::IsVector,
            },
            arguments: usize::try_from(operand % 4).expect("Small builtin count"),
        },
        _ => KernelInstruction::Return,
    }
}

#[test]
fn hostile_programs_are_rejected_or_execute_within_their_proof_and_fuel() -> Result<(), String> {
    let mut state = 0x72ac_01f3;
    let mut admitted = 0;
    let verification = KernelVerificationBudget {
        max_instructions: 64,
        max_steps: 256,
        max_stack_values: 64,
        max_metadata_bytes: 16 * 1024,
        max_local_values: 0,
    };
    let execution = KernelExecutionBudget {
        max_steps: 128,
        max_stack_values: 64,
        max_stack_bytes: 64 * size_of::<KernelValue>() as u64,
        max_local_values: 0,
        max_local_bytes: 0,
    };
    for _ in 0..4096 {
        let count = usize::try_from(next(&mut state) % 32).expect("Small count");
        let input: Vec<_> = (0..count).map(|_| instruction(&mut state, count)).collect();
        let Ok(program) = VerifiedKernelProgram::verify(&input, verification) else {
            continue;
        };
        admitted += 1;
        match program.execute(execution) {
            Ok(outcome) => {
                assert!(outcome.steps <= execution.max_steps);
                assert!(outcome.peak_stack_values <= program.usage().max_stack_values);
            }
            Err(error) => {
                assert!(error.work_steps <= execution.max_steps);
                assert!(error.peak_stack_values <= program.usage().max_stack_values);
                if !matches!(
                    error.kind,
                    KernelErrorKind::ExpectedNumber
                        | KernelErrorKind::UnmatchingTypes
                        | KernelErrorKind::ExpectedIndexable
                        | KernelErrorKind::Numeric(_)
                        | KernelErrorKind::Vector(_)
                        | KernelErrorKind::ArgumentCount { expected: 1, .. }
                        | KernelErrorKind::BudgetExceeded {
                            resource: KernelResource::ExecutionSteps,
                            ..
                        }
                ) {
                    return Err(format!(
                        "A verified hostile program violated its execution contract: {error:?}"
                    ));
                }
            }
        }
    }
    assert!(
        admitted > 0,
        "The corpus must exercise admitted programs too"
    );
    Ok(())
}
