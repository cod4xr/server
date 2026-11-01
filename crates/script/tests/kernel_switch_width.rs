// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Wide immutable labels must not create runtime cache slots or string allocations."
)]
use crate::{kernel_call_support, kernel_switch_support};
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelInstruction, KernelSwitchComparison as Comparison,
    KernelSwitchKey as Key, KernelValue,
};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::legacy::{LegacyArithmetic, LegacyNumber};

fn number(value: i32) -> KernelValue {
    KernelValue::Number(LegacyNumber::Integer(value))
}

#[test]
fn seventy_thousand_labels_and_wide_operands_use_no_runtime_string_cache_or_scratch_for_integer_keys()
 {
    let mut labels = vec![Vec::new(); 70_000];
    labels[69_999] = vec![b'x'; 70_000];
    let module =
        kernel_switch_support::module(Key::StringLabel(69_999), Comparison::Equal, &labels);
    drop(labels);
    assert_eq!(
        (module.usage().string_literals, module.usage().switch_labels),
        (0, 70_000)
    );
    assert_eq!(
        module.usage().retained_switch_bytes,
        (70_000 * size_of::<Vec<u8>>() + 70_000) as u64
    );
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 0,
        max_payload_bytes: 0,
        max_slots: 0,
        max_metadata_bytes: 0,
        max_single_string_bytes: 0,
    })
    .expect("No heap payload admission");
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_scratch_bytes: 0,
            ..HeapWorkBudget::default()
        },
    );
    assert_eq!(
        module
            .execute_with_heap(0, &[number(1)], KernelCallBudget::default(), &mut context)
            .expect("No case-cache slots")
            .value,
        number(0)
    );
    assert_eq!(context.heap_usage().slots, 0);
    assert_eq!(context.usage().peak_scratch_bytes, 0);
}

#[test]
fn complete_seventy_thousand_byte_string_keys_match_owned_labels_with_one_input_slot() {
    let bytes = vec![b'x'; 70_000];
    let module = kernel_switch_support::module(
        Key::StringLabel(0),
        Comparison::Equal,
        std::slice::from_ref(&bytes),
    );
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 1,
        max_slots: 1,
        max_payload_bytes: 70_000,
        max_single_string_bytes: 70_000,
        ..StringHeapBudget::default()
    })
    .expect("One input");
    let id = strings
        .allocate(&bytes, HeapWorkBudget::default())
        .expect("Full input")
        .id;
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_scratch_bytes: 0,
            ..HeapWorkBudget::default()
        },
    );
    assert_eq!(
        module
            .execute_with_heap(
                0,
                &[KernelValue::String(id)],
                KernelCallBudget::default(),
                &mut context
            )
            .expect("No narrowed string lengths or label allocations")
            .value,
        number(1)
    );
    assert_eq!(
        (
            context.heap_usage().slots,
            context.heap_usage().payload_bytes
        ),
        (1, 70_000)
    );
    assert_eq!(context.usage().peak_scratch_bytes, 0);
}

#[test]
fn calls_keep_label_operands_function_local_without_literal_cache_offsets() {
    let entry_code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::SwitchCompare {
            key: Key::StringLabel(0),
            comparison: Comparison::Equal,
        },
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Call {
            function: 1,
            arguments: 1,
        },
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let invoked_code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::SwitchCompare {
            key: Key::StringLabel(0),
            comparison: Comparison::Equal,
        },
        KernelInstruction::Return,
    ];
    let alpha = [b"alpha".to_vec()];
    let beta = [b"beta".to_vec()];
    let mut first = kernel_call_support::definition(&entry_code, 2, 2);
    first.switch_labels = &alpha;
    let mut second = kernel_call_support::definition(&invoked_code, 1, 1);
    second.switch_labels = &beta;
    let module = kernel_call_support::verify(&[first, second]);
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 2,
        max_slots: 2,
        ..StringHeapBudget::default()
    })
    .expect("Inputs only");
    let left = strings.allocate(b"alpha", work).expect("Left").id;
    let right = strings.allocate(b"beta", work).expect("Right").id;
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_scratch_bytes: 0,
            ..work
        },
    );
    assert_eq!(
        module
            .execute_with_heap(
                0,
                &[KernelValue::String(left), KernelValue::String(right)],
                KernelCallBudget::default(),
                &mut context
            )
            .expect("Independent code tables")
            .value,
        number(2)
    );
    assert_eq!(
        (module.usage().string_literals, module.usage().switch_labels),
        (0, 2)
    );
    assert_eq!(context.heap_usage().slots, 2);
}
