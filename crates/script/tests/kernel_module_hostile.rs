// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Deterministic bounded hostile inventories may reject proof or invocation."
)]

use crate::kernel_call_support::{definition, integer};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelInstruction, KernelModuleVerificationBudget,
    KernelValue, VerifiedKernelModule,
};
use cod4x_script::legacy::{LegacyComparison, LegacyNumber, LegacyVector};

#[test]
fn hostile_forward_recursive_and_cyclic_inventories_remain_bounded_after_proof()
-> Result<(), String> {
    let mut state = 0x5b1d_c620_7cae_385f;
    let mut admitted = 0;
    let budget = KernelCallBudget {
        max_steps: 128,
        max_setup_work: 256,
        max_frames: 8,
        max_frame_bytes: 1024 * 1024,
        max_stack_values: 64,
        max_stack_bytes: 64 * size_of::<KernelValue>() as u64,
        max_local_values: 64,
        max_local_bytes: 64 * size_of::<Option<KernelValue>>() as u64,
    };
    for _ in 0..4096 {
        let codes: Vec<_> = (0..4).map(|_| code(&mut state)).collect();
        let definitions: Vec<_> = codes.iter().map(|code| definition(code, 1, 2)).collect();
        let Ok(module) = VerifiedKernelModule::verify(
            &definitions,
            KernelModuleVerificationBudget {
                max_functions: 4,
                max_instructions: 64,
                max_steps: 1024,
                max_metadata_bytes: 64 * 1024,
                max_function_stack_values: 16,
                max_function_local_values: 4,
            },
        ) else {
            continue;
        };
        admitted += 1;
        let arguments = [KernelValue::Number(LegacyNumber::Integer(3))];
        for entry in 0..4 {
            match module.execute(entry, &arguments, budget) {
                Ok(outcome) => {
                    assert!(outcome.usage.steps <= budget.max_steps);
                    assert!(outcome.usage.setup_work <= budget.max_setup_work);
                    assert!(outcome.usage.peak_frames <= budget.max_frames);
                    assert!(outcome.usage.peak_stack_values <= budget.max_stack_values);
                    assert!(outcome.usage.peak_local_values <= budget.max_local_values);
                }
                Err(error) => {
                    assert!(error.cause.work_steps <= budget.max_steps);
                    assert!(error.setup_work <= budget.max_setup_work);
                    assert!(error.peak_frames <= budget.max_frames);
                    assert!(error.peak_reserved_stack_values <= budget.max_stack_values);
                    assert!(error.peak_local_values <= budget.max_local_values);
                    if !matches!(
                        error.cause.kind,
                        KernelErrorKind::BudgetExceeded { .. }
                            | KernelErrorKind::ArgumentCount { .. }
                            | KernelErrorKind::ExpectedFunctionReference
                            | KernelErrorKind::UninitializedLocal { .. }
                            | KernelErrorKind::ExpectedNumber
                            | KernelErrorKind::ExpectedIndexable
                            | KernelErrorKind::UnmatchingTypes
                            | KernelErrorKind::Numeric(_)
                            | KernelErrorKind::Vector(_)
                    ) {
                        return Err(format!(
                            "Verified hostile module violated runtime contract: {error:?}"
                        ));
                    }
                }
            }
        }
        assert_eq!(arguments[0], KernelValue::Number(LegacyNumber::Integer(3)));
    }
    assert!(
        admitted >= 64,
        "Corpus must execute admitted call inventories"
    );
    Ok(())
}

fn code(state: &mut u64) -> Vec<KernelInstruction> {
    let function = usize::try_from(next(state) % 6).expect("Small function index");
    let arguments = usize::try_from(next(state) % 3).expect("Small arity");
    let mut code = Vec::new();
    for _ in 0..arguments {
        code.push(KernelInstruction::Constant(match next(state) % 3 {
            0 => KernelValue::Undefined,
            1 => KernelValue::Number(LegacyNumber::Float(-0.0)),
            _ => KernelValue::Vector(LegacyVector::from_components([1.0, 2.0, 3.0])),
        }));
    }
    match next(state) % 3 {
        0 => code.push(KernelInstruction::Call {
            function,
            arguments,
        }),
        1 => {
            code.push(KernelInstruction::FunctionReference(function));
            code.push(KernelInstruction::CallIndirect { arguments });
        }
        _ => {
            code.push(integer(7));
            code.push(KernelInstruction::CallIndirect { arguments });
        }
    }
    match next(state) % 6 {
        0 => code.extend([integer(0), KernelInstruction::Divide]),
        1 => code.extend([
            KernelInstruction::Constant(KernelValue::Undefined),
            KernelInstruction::Compare(LegacyComparison::Equal),
        ]),
        2 => code.extend([integer(0), KernelInstruction::Index]),
        3 => code.extend([KernelInstruction::Discard, KernelInstruction::LoadLocal(1)]),
        4 => {
            code.extend([KernelInstruction::Discard, KernelInstruction::Jump(0)]);
            return code;
        }
        _ => {}
    }
    code.push(KernelInstruction::Return);
    code
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}
