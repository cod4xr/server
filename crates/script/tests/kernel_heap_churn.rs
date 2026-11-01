// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute finite admitted allocation loops and fuel failures."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelBranchCondition, KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelInstruction,
    KernelResource, KernelValue,
};
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::legacy::{LegacyArithmetic, LegacyComparison, LegacyIntegerUpdate};

fn allocation_loop(iterations: i32) -> Vec<KernelInstruction> {
    vec![
        integer(0),
        KernelInstruction::StoreLocal(3),
        KernelInstruction::LoadLocal(3),
        integer(iterations),
        KernelInstruction::Compare(LegacyComparison::Less),
        KernelInstruction::Branch {
            condition: KernelBranchCondition::IfFalse,
            target: 14,
        },
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::StoreLocal(2),
        KernelInstruction::LoadLocal(3),
        KernelInstruction::IntegerUpdate(LegacyIntegerUpdate::Increment),
        KernelInstruction::StoreLocal(3),
        KernelInstruction::Jump(2),
        KernelInstruction::LoadLocal(2),
        KernelInstruction::Return,
    ]
}

#[test]
fn overwritten_vm_temporaries_reclaim_under_four_slots_for_one_hundred_thousand_iterations()
-> Result<(), String> {
    let code = allocation_loop(100_000);
    let module = kernel_call_support::verify(&[definition(&code, 2, 4)]);
    let mut heap = kernel_heap_support::heap(4);
    let left = allocate(&mut heap, b"a");
    let right = allocate(&mut heap, b"b");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::String(left), KernelValue::String(right)],
            KernelCallBudget {
                max_steps: 2_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Bounded VM churn");
    let KernelValue::String(id) = result.value else {
        return Err("Expected final string".into());
    };
    assert_eq!(context.get(id).expect("Final result"), b"ab");
    assert_eq!(context.get(left).expect("Input local"), b"a");
    assert_eq!(context.get(right).expect("Input local"), b"b");
    assert_eq!(result.usage.steps, 1_200_008);
    assert_eq!(result.usage.peak_frames, 1);
    assert_eq!(context.heap_usage().slots, 4);
    assert_eq!(context.heap_usage().live_strings, 4);
    assert_eq!(context.heap_usage().payload_bytes, 6);
    assert_eq!(context.usage().collections, 99_998);
    assert!(context.usage().steps < 8_000_000);
    Ok(())
}

#[test]
fn fuel_failure_after_collections_keeps_external_roots_and_allows_subsequent_invocation()
-> Result<(), String> {
    let code = allocation_loop(100_000);
    let identity = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let module =
        kernel_call_support::verify(&[definition(&code, 2, 4), definition(&identity, 1, 1)]);
    let mut heap = kernel_heap_support::heap(5);
    let left = allocate(&mut heap, b"a");
    let right = allocate(&mut heap, b"b");
    let host = allocate(&mut heap, b"host result");
    let roots = [host];
    let mut context = KernelHeapContext::new(&mut heap, &roots, HeapWorkBudget::default());
    let error = module
        .execute_with_heap(
            0,
            &[KernelValue::String(left), KernelValue::String(right)],
            KernelCallBudget {
                max_steps: 1000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect_err("Instruction cancellation boundary");
    assert!(matches!(
        error.cause.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionSteps,
            ..
        }
    ));
    assert_eq!(error.cause.work_steps, 1000);
    assert!(context.usage().collections > 0);
    assert_eq!(
        context.get(host).expect("External root on failure"),
        b"host result"
    );
    let before = context.usage().steps;
    let result = module
        .execute_with_heap(
            1,
            &[KernelValue::String(host)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Fresh frames after failure");
    let KernelValue::String(id) = result.value else {
        return Err("Expected host string".into());
    };
    assert_eq!(context.get(id).expect("Preserved result"), b"host result");
    assert_eq!(context.usage().steps, before + 2);
    Ok(())
}
