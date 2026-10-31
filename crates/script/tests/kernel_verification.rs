// SPDX-License-Identifier: AGPL-3.0-or-later
//! Control-flow/return/stack attacks and wide instruction targets.
#![allow(
    clippy::expect_used,
    reason = "Tests assemble deliberately valid and invalid instruction streams."
)]

use cod4x_script::execution::{
    KernelBranchCondition, KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyArithmetic, LegacyNumber};

use crate::kernel_support;

#[test]
fn empty_bad_targets_underflows_returns_and_fallthrough_are_rejected_before_execution() {
    for (input, instruction, kind) in [
        (vec![], None, KernelErrorKind::EmptyProgram),
        (
            vec![KernelInstruction::Return],
            Some(0),
            KernelErrorKind::InvalidReturnStack { values: 0 },
        ),
        (
            vec![
                KernelInstruction::Arithmetic(LegacyArithmetic::Add),
                KernelInstruction::Return,
            ],
            Some(0),
            KernelErrorKind::StackUnderflow {
                required: 2,
                available: 0,
            },
        ),
        (
            vec![kernel_support::integer(1)],
            Some(0),
            KernelErrorKind::FallthroughPastEnd,
        ),
        (
            vec![
                kernel_support::integer(1),
                KernelInstruction::Branch {
                    condition: KernelBranchCondition::IfTrue,
                    target: 0,
                },
            ],
            Some(1),
            KernelErrorKind::FallthroughPastEnd,
        ),
        (
            vec![
                kernel_support::integer(1),
                kernel_support::integer(2),
                KernelInstruction::Return,
            ],
            Some(2),
            KernelErrorKind::InvalidReturnStack { values: 2 },
        ),
        (
            vec![
                kernel_support::integer(1),
                KernelInstruction::Return,
                KernelInstruction::Jump(99),
            ],
            Some(2),
            KernelErrorKind::InvalidTarget { target: 99 },
        ),
        (
            vec![KernelInstruction::Jump(usize::MAX)],
            Some(0),
            KernelErrorKind::InvalidTarget {
                target: usize::MAX as u64,
            },
        ),
        (
            vec![
                KernelInstruction::Branch {
                    condition: KernelBranchCondition::IfTrue,
                    target: 0,
                },
                KernelInstruction::Return,
            ],
            Some(0),
            KernelErrorKind::StackUnderflow {
                required: 1,
                available: 0,
            },
        ),
    ] {
        let error = VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
            .expect_err("Invalid program");
        assert_eq!(error.kind, kind);
        assert_eq!(error.instruction, instruction);
    }
}

#[test]
fn inconsistent_joins_and_stack_growing_loops_cannot_acquire_a_proof() -> Result<(), String> {
    for input in [
        vec![kernel_support::integer(1), KernelInstruction::Jump(0)],
        vec![
            kernel_support::integer(1),
            KernelInstruction::Branch {
                condition: KernelBranchCondition::IfFalse,
                target: 4,
            },
            kernel_support::integer(1),
            KernelInstruction::Jump(5),
            KernelInstruction::Jump(5),
            KernelInstruction::Jump(5),
        ],
    ] {
        let error = VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
            .expect_err("Unbalanced loop/join");
        let KernelErrorKind::StackJoin {
            expected, actual, ..
        } = error.kind
        else {
            return Err("Expected inconsistent-height join".to_owned());
        };
        assert_ne!(expected, actual);
        assert!(error.work_steps >= input.len() as u64);
    }
    Ok(())
}

#[test]
fn wide_targets_and_unreachable_stack_operations_preserve_valid_execution() {
    let target = 70_000;
    let mut input = vec![KernelInstruction::Discard; target + 2];
    input[0] = KernelInstruction::Jump(target);
    input[target] = kernel_support::integer(9);
    input[target + 1] = KernelInstruction::Return;
    let program = VerifiedKernelProgram::verify(&input, KernelVerificationBudget::default())
        .expect("Wide proof");
    assert_eq!(program.usage().instructions, 70_002);
    assert_eq!(program.usage().verification_steps, 70_007);
    assert_eq!(program.usage().max_stack_values, 1);
    let outcome = program
        .execute(KernelExecutionBudget::default())
        .expect("Wide target execution");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(9)));
    assert_eq!(outcome.steps, 3);
}
