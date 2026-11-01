// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests wide typed writes and bounded unique updates without receiver aliases beside unrelated host owners."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::{
    KernelBranchCondition, KernelCallBudget, KernelHeapContext, KernelInstruction,
    VerifiedKernelModule,
};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::{LegacyArithmetic, LegacyComparison, LegacyIntegerUpdate};

#[derive(Clone, Copy)]
enum Keys {
    Counter,
    Zero,
}

fn loop_module(count: i32, keys: Keys) -> VerifiedKernelModule {
    let key = match keys {
        Keys::Counter => KernelInstruction::LoadLocal(1),
        Keys::Zero => integer(0),
    };
    let mut code = vec![
        KernelInstruction::EmptyArray,
        KernelInstruction::StoreLocal(0),
        integer(0),
        KernelInstruction::StoreLocal(1),
        KernelInstruction::LoadLocal(1),
        integer(count),
        KernelInstruction::Compare(LegacyComparison::Less),
        KernelInstruction::Branch {
            condition: KernelBranchCondition::IfFalse,
            target: 15,
        },
        KernelInstruction::LoadLocal(1),
        key,
        KernelInstruction::StoreArrayLocal(0),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::IntegerUpdate(LegacyIntegerUpdate::Increment),
        KernelInstruction::StoreLocal(1),
        KernelInstruction::Jump(4),
    ];
    code.extend([
        integer(if matches!(keys, Keys::Counter) {
            count - 1
        } else {
            0
        }),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Index,
    ]);
    if matches!(keys, Keys::Counter) {
        code.extend([
            KernelInstruction::LoadLocal(0),
            KernelInstruction::Size,
            KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        ]);
    }
    code.push(KernelInstruction::Return);
    kernel_call_support::verify(&[definition(&code, 0, 2)])
}

#[test]
fn seventy_thousand_typed_entries_preserve_wide_size_and_index_without_receiver_copies() {
    let module = loop_module(70_000, Keys::Counter);
    let (mut strings, mut objects) = stores(ObjectHeapBudget {
        max_live_objects: 1,
        max_object_slots: 1,
        max_array_values: 2,
        max_array_value_slots: 2,
        max_live_fields: 70_000,
        max_field_slots: 70_000,
        max_key_bytes: 630_000,
        ..ObjectHeapBudget::default()
    });
    let work = HeapWorkBudget::default();
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let result = module
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Wide unique typed writes");
    assert_eq!(result.value, number(139_999));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (
            usage.live_arrays,
            usage.live_array_values,
            usage.array_value_slots
        ),
        (1, 0, 2)
    );
    assert_eq!(
        (usage.live_fields, usage.field_slots, usage.key_bytes),
        (70_000, 70_000, 630_000)
    );
    assert_eq!(context.usage().object_collections, 0);
    drop(context);
    assert_eq!(
        objects
            .reclaim(&[], work)
            .expect("All independent data unowned")
            .fields,
        70_000
    );
}

#[test]
fn one_hundred_thousand_unique_typed_updates_reuse_one_field_beside_seventy_thousand_host_aliases()
{
    let module = loop_module(100_000, Keys::Zero);
    let (mut strings, mut objects) = stores(ObjectHeapBudget {
        max_live_objects: 2,
        max_object_slots: 2,
        max_array_values: 70_003,
        max_array_value_slots: 70_003,
        max_live_fields: 2,
        max_field_slots: 2,
        max_key_bytes: 18,
        max_index_buckets: 8,
        ..ObjectHeapBudget::default()
    });
    let work = HeapWorkBudget::default();
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Host data").value;
    objects
        .array_put_unique(host, ArrayKey::Index(1), number(77), &strings, work)
        .expect("Host entry");
    for _ in 0..70_000 {
        objects.copy_array_value(host, work).expect("Host alias");
    }
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_steps: 16_000_000,
            max_scratch_bytes: 4096,
        },
    )
    .with_objects(&mut objects, &[]);
    let result = module
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget {
                max_steps: 2_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Bounded updates independent of unrelated host rows");
    assert_eq!(result.value, number(99_999));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (
            usage.live_arrays,
            usage.object_slots,
            usage.live_fields,
            usage.field_slots
        ),
        (2, 2, 2, 2)
    );
    assert_eq!(
        (usage.live_array_values, usage.array_value_slots),
        (70_001, 70_003)
    );
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (0, 0)
    );
    assert!(context.usage().peak_scratch_bytes <= 4096);
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("All host owners retained")
            .value,
        70_001
    );
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(1), &strings, work)
            .expect("Host payload unchanged")
            .value,
        Some(number(77))
    );
}
