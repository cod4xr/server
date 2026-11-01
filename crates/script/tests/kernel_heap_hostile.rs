// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests generate bounded deterministic hostile typed inventories."
)]
use crate::kernel_call_support::definition;
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelInstruction,
    KernelModuleVerificationBudget, KernelValue, VerifiedKernelModule,
};
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::legacy::{LegacyArithmetic, LegacyComparison, LegacyNumber};

#[test]
fn hostile_string_call_inventories_preserve_proof_heap_and_execution_bounds() -> Result<(), String>
{
    let mut state = 0x19ad_2567_345b_1123;
    let mut admitted = 0;
    for _ in 0..4096 {
        let codes: Vec<_> = (0..4).map(|_| code(&mut state)).collect();
        let definitions: Vec<_> = codes.iter().map(|body| definition(body, 2, 2)).collect();
        let Ok(module) =
            VerifiedKernelModule::verify(&definitions, KernelModuleVerificationBudget::default())
        else {
            continue;
        };
        admitted += 1;
        let mut heap = kernel_heap_support::heap(8);
        let left = allocate(&mut heap, b"xy");
        let right = allocate(&mut heap, b"z");
        let roots = [left, right];
        let mut context = KernelHeapContext::new(
            &mut heap,
            &roots,
            HeapWorkBudget {
                max_steps: 1024,
                max_scratch_bytes: 4096,
            },
        );
        for entry in 0..4 {
            let result = module.execute_with_heap(
                entry,
                &[KernelValue::String(left), KernelValue::String(right)],
                KernelCallBudget {
                    max_steps: 128,
                    max_setup_work: 4096,
                    max_frames: 16,
                    ..KernelCallBudget::default()
                },
                &mut context,
            );
            match result {
                Ok(result) => {
                    assert!(result.usage.steps <= 128);
                    assert!(result.usage.peak_frames <= 16);
                    if let KernelValue::String(id) = result.value {
                        context.get(id).expect("Returned valid handle");
                    }
                }
                Err(error) => {
                    assert!(error.cause.work_steps <= 128);
                    assert!(error.peak_frames <= 16);
                    if !matches!(
                        error.cause.kind,
                        KernelErrorKind::BudgetExceeded { .. }
                            | KernelErrorKind::UnmatchingTypes
                            | KernelErrorKind::ExpectedStringIndex
                            | KernelErrorKind::StringIndexOutOfRange
                            | KernelErrorKind::ExpectedSized
                    ) {
                        return Err(format!("Unexpected hostile runtime failure: {error:?}"));
                    }
                }
            }
            assert!(context.usage().steps <= 1024);
            assert!(context.usage().peak_scratch_bytes <= 4096);
            assert!(context.heap_usage().live_strings <= 8);
            assert!(context.heap_usage().slots <= 8);
            assert_eq!(context.get(left).expect("External left"), b"xy");
            assert_eq!(context.get(right).expect("External right"), b"z");
        }
    }
    assert!(admitted >= 1024);
    Ok(())
}

fn code(state: &mut u64) -> Vec<KernelInstruction> {
    let mut code = vec![
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(1),
    ];
    let target = usize::try_from(next(state) % 6).expect("Small target");
    match next(state) % 10 {
        0 => code.push(KernelInstruction::Call {
            function: target,
            arguments: 2,
        }),
        1 => code.extend([
            KernelInstruction::FunctionReference(target),
            KernelInstruction::CallIndirect { arguments: 2 },
        ]),
        2 | 3 => code.push(KernelInstruction::Arithmetic(LegacyArithmetic::Add)),
        4 => code.push(KernelInstruction::Compare(LegacyComparison::Equal)),
        5 => code.push(KernelInstruction::Compare(LegacyComparison::NotEqual)),
        6 => code.push(KernelInstruction::Compare(LegacyComparison::Less)),
        7 => code.push(KernelInstruction::Divide),
        8 => {
            let index = i32::try_from(next(state) % 5).expect("Small index") - 1;
            code[0] =
                KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Integer(index)));
            code.push(KernelInstruction::Index);
        }
        _ => code.extend([KernelInstruction::Discard, KernelInstruction::Size]),
    }
    if next(state).is_multiple_of(8) {
        code.extend([KernelInstruction::Discard, KernelInstruction::Jump(0)]);
    } else {
        code.push(KernelInstruction::Return);
    }
    code
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}
