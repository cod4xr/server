// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure exact owned execution admission and deliberate failures."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelInstruction,
    KernelResource, KernelValue,
};
use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget};
use cod4x_script::legacy::LegacyArithmetic;

#[test]
fn pressure_reclamation_work_and_simultaneous_scratch_have_exact_one_less_limits()
-> Result<(), String> {
    let program = kernel_heap_support::binary(KernelInstruction::Arithmetic(LegacyArithmetic::Add));
    let mut measured = None;
    for limits in 0..3 {
        let mut heap = kernel_heap_support::heap(3);
        let left = allocate(&mut heap, b"a");
        let right = allocate(&mut heap, b"b");
        let garbage = allocate(&mut heap, b"old");
        let before = heap.usage();
        let mut budget = HeapWorkBudget::default();
        if let Some((steps, scratch)) = measured {
            budget.max_steps = steps;
            budget.max_scratch_bytes = scratch;
            if limits == 2 {
                budget.max_scratch_bytes -= 1;
            }
        }
        let mut context = KernelHeapContext::new(&mut heap, &[], budget);
        let result = program.execute_with_heap(
            &[KernelValue::String(left), KernelValue::String(right)],
            KernelExecutionBudget::default(),
            &mut context,
        );
        if limits == 2 {
            let error = result.expect_err("One less shared scratch byte");
            assert!(matches!(
                error.kind,
                KernelErrorKind::BudgetExceeded {
                    resource: KernelResource::HeapScratchBytes,
                    ..
                }
            ));
            assert_eq!(context.heap_usage(), before);
            assert_eq!(context.get(garbage).expect("No deletion"), b"old");
            assert_eq!(context.usage().collections, 0);
        } else {
            let result = result.expect("Measured/exact work and scratch");
            let KernelValue::String(id) = result.value else {
                return Err("Expected string".into());
            };
            assert_eq!(context.get(id).expect("Complete result"), b"ab");
            assert_eq!(context.get(garbage), Err(HeapErrorKind::StaleReference));
            assert_eq!(context.usage().steps, 35);
            assert_eq!(context.usage().collections, 1);
            measured = Some((context.usage().steps, context.usage().peak_scratch_bytes));
        }
    }
    let (steps, _) = measured.ok_or("Missing observation")?;
    for limit in 0..steps {
        let mut heap = kernel_heap_support::heap(3);
        let left = allocate(&mut heap, b"a");
        let right = allocate(&mut heap, b"b");
        allocate(&mut heap, b"old");
        let mut context = KernelHeapContext::new(
            &mut heap,
            &[],
            HeapWorkBudget {
                max_steps: limit,
                ..HeapWorkBudget::default()
            },
        );
        let error = program
            .execute_with_heap(
                &[KernelValue::String(left), KernelValue::String(right)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect_err("Every smaller global work budget");
        match error.kind {
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::HeapWorkSteps,
                requested,
                maximum,
            } => {
                assert_eq!(maximum, limit);
                assert!(requested > maximum);
            }
            _ => return Err("Unexpected work failure".into()),
        }
        assert!(context.usage().steps <= limit);
        assert_eq!(context.get(left).expect("Operands remain live"), b"a");
        assert_eq!(context.get(right).expect("Operands remain live"), b"b");
    }
    Ok(())
}

#[test]
fn heap_work_is_shared_across_callees_and_reused_invocations() {
    let add = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let root = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Call {
            function: 1,
            arguments: 2,
        },
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Call {
            function: 1,
            arguments: 2,
        },
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[definition(&root, 1, 1), definition(&add, 2, 2)]);
    let mut heap = kernel_heap_support::heap(4);
    let id = allocate(&mut heap, b"x");
    let mut context = KernelHeapContext::new(
        &mut heap,
        &[],
        HeapWorkBudget {
            max_steps: 8,
            ..HeapWorkBudget::default()
        },
    );
    let error = module
        .execute_with_heap(
            0,
            &[KernelValue::String(id)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("Second callee cannot reset work");
    assert_eq!(error.function, Some(1));
    assert!(matches!(
        error.cause.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::HeapWorkSteps,
            maximum: 8,
            ..
        }
    ));
    assert_eq!(context.usage().steps, 8);
    let error = module
        .execute_with_heap(
            1,
            &[KernelValue::String(id), KernelValue::String(id)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("New entry cannot reset context work");
    assert_eq!(error.cause.work_steps, 0);
    assert_eq!(context.usage().steps, 8);
    assert_eq!(context.get(id).expect("Original string"), b"x");
}
