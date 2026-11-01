// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests independently constrain each actual runtime storage resource."
)]
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelInstruction, KernelResource,
    KernelValue,
};
use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::legacy::LegacyArithmetic;

#[test]
fn live_payload_slot_and_metadata_pressure_each_reclaim_before_retrying_allocation()
-> Result<(), String> {
    let mut sample = kernel_heap_support::heap(1);
    allocate(&mut sample, b"a");
    let record_bytes = sample.usage().metadata_bytes;
    let program = kernel_heap_support::binary(KernelInstruction::Arithmetic(LegacyArithmetic::Add));
    for budget in [
        StringHeapBudget {
            max_live_strings: 3,
            ..StringHeapBudget::default()
        },
        StringHeapBudget {
            max_payload_bytes: 4,
            ..StringHeapBudget::default()
        },
        StringHeapBudget {
            max_slots: 3,
            ..StringHeapBudget::default()
        },
        StringHeapBudget {
            max_metadata_bytes: 3 * record_bytes,
            ..StringHeapBudget::default()
        },
    ] {
        let mut heap = StringHeap::new(budget).expect("Heap");
        let left = allocate(&mut heap, b"a");
        let right = allocate(&mut heap, b"b");
        let garbage = allocate(&mut heap, b"x");
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        let result = program
            .execute_with_heap(
                &[KernelValue::String(left), KernelValue::String(right)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect("Resource-specific pressure");
        let KernelValue::String(id) = result.value else {
            return Err("Expected string".into());
        };
        assert_eq!(context.get(id).expect("Complete output"), b"ab");
        assert_eq!(context.get(garbage), Err(HeapErrorKind::StaleReference));
        assert_eq!(context.usage().collections, 1);
        assert_eq!(context.heap_usage().slots, 3);
        assert_eq!(context.heap_usage().live_strings, 3);
        assert_eq!(context.heap_usage().payload_bytes, 4);
    }
    Ok(())
}

#[test]
fn noncollectable_limits_and_live_roots_cannot_be_evaded_by_retrying() {
    let program = kernel_heap_support::binary(KernelInstruction::Arithmetic(LegacyArithmetic::Add));
    let mut heap = StringHeap::new(StringHeapBudget {
        max_single_string_bytes: 1,
        ..StringHeapBudget::default()
    })
    .expect("Heap");
    let left = allocate(&mut heap, b"a");
    let right = allocate(&mut heap, b"b");
    let garbage = allocate(&mut heap, b"x");
    let before = heap.usage();
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let error = program
        .execute_with_heap(
            &[KernelValue::String(left), KernelValue::String(right)],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect_err("Single-string policy");
    assert!(matches!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::SingleStringBytes,
            requested: 2,
            maximum: 1
        }
    ));
    assert_eq!(context.usage().collections, 0);
    assert_eq!(context.heap_usage(), before);
    assert_eq!(
        context.get(garbage).expect("No gratuitous collection"),
        b"x"
    );
    let mut heap = kernel_heap_support::heap(2);
    let left = allocate(&mut heap, b"a");
    let right = allocate(&mut heap, b"b");
    let before = heap.usage();
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let error = program
        .execute_with_heap(
            &[KernelValue::String(left), KernelValue::String(right)],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect_err("All strings rooted");
    assert!(matches!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::StringValues,
            requested: 3,
            maximum: 2
        }
    ));
    assert_eq!(context.usage().collections, 1);
    assert_eq!(context.heap_usage(), before);
    assert_eq!(context.get(left).expect("Left root"), b"a");
    assert_eq!(context.get(right).expect("Right root"), b"b");
}
