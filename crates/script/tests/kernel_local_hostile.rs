// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded hostile local/branch streams cannot invalidate an owned stack/layout proof.
#![allow(
    clippy::expect_used,
    reason = "Tests generate bounded deterministic instructions/layouts."
)]

use cod4x_script::execution::{
    KernelBranchCondition, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout,
    KernelInstruction, KernelResource, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyArithmetic, LegacyNumber};

use crate::kernel_support;

fn next(state: &mut u32) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state
}

#[test]
fn hostile_local_streams_reject_or_execute_within_layout_stack_and_fuel() -> Result<(), String> {
    let mut state = 0x164f_5a2b;
    let layout = KernelFunctionLayout::new(2, 8).expect("Finite layout");
    let verification = KernelVerificationBudget {
        max_instructions: 64,
        max_steps: 512,
        max_stack_values: 64,
        max_local_values: 8,
        max_metadata_bytes: 16 * 1024,
    };
    let execution = KernelExecutionBudget {
        max_steps: 128,
        max_stack_values: 64,
        max_stack_bytes: 4096,
        max_local_values: 8,
        max_local_bytes: 4096,
    };
    let arguments = [
        KernelValue::Number(LegacyNumber::Integer(3)),
        KernelValue::Undefined,
    ];
    let mut admitted = 0;
    for iteration in 0..4096 {
        let count = usize::try_from(next(&mut state) % 32).expect("Small count");
        let mut input = Vec::new();
        for _ in 0..count {
            let choice = next(&mut state);
            let slot = usize::try_from(next(&mut state) % 12).expect("Small local");
            let target =
                usize::try_from(next(&mut state)).expect("Supported test host") % (count + 3);
            input.push(match choice % 10 {
                0 => KernelInstruction::LoadLocal(slot),
                1 => KernelInstruction::StoreLocal(slot),
                2 => kernel_support::integer(choice.cast_signed()),
                3 => KernelInstruction::Return,
                4 => KernelInstruction::Jump(target),
                5 => KernelInstruction::Branch {
                    condition: KernelBranchCondition::IfFalse,
                    target,
                },
                6 => KernelInstruction::Arithmetic(LegacyArithmetic::Add),
                7 => KernelInstruction::Discard,
                8 => KernelInstruction::Duplicate,
                _ => KernelInstruction::Constant(KernelValue::Undefined),
            });
        }
        if iteration % 32 == 0 {
            input = vec![
                KernelInstruction::LoadLocal(0),
                KernelInstruction::StoreLocal(7),
                KernelInstruction::LoadLocal(7),
                KernelInstruction::Return,
            ];
        }
        let Ok(program) = VerifiedKernelProgram::verify_function(&input, layout, verification)
        else {
            continue;
        };
        admitted += 1;
        match program.execute_with_arguments(&arguments, execution) {
            Ok(outcome) => {
                assert!(outcome.steps <= execution.max_steps);
                assert!(outcome.peak_stack_values <= program.usage().max_stack_values);
                assert_eq!(outcome.local_values, 8);
            }
            Err(error) => {
                assert!(error.work_steps <= execution.max_steps);
                assert!(error.peak_stack_values <= program.usage().max_stack_values);
                assert_eq!(error.admitted_local_values, 8);
                if !matches!(
                    error.kind,
                    KernelErrorKind::UninitializedLocal { .. }
                        | KernelErrorKind::ExpectedNumber
                        | KernelErrorKind::Numeric(_)
                        | KernelErrorKind::BudgetExceeded {
                            resource: KernelResource::ExecutionSteps,
                            ..
                        }
                ) {
                    return Err(format!(
                        "Verified local stream violated execution contract: {:?}",
                        error.kind
                    ));
                }
            }
        }
        assert_eq!(arguments[0], KernelValue::Number(LegacyNumber::Integer(3)));
        assert_eq!(arguments[1], KernelValue::Undefined);
    }
    assert!(admitted >= 128);
    Ok(())
}
