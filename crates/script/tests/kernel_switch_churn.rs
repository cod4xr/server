// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Repeated key comparisons cannot scan unrelated host owner histories or allocate label strings."
)]
use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelBranchCondition, KernelCallBudget, KernelHeapContext, KernelInstruction as Op,
    KernelSwitchComparison, KernelSwitchKey, KernelValue,
};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::{LegacyComparison, LegacyIntegerUpdate, LegacyNumber};

fn module() -> cod4x_script::execution::VerifiedKernelModule {
    let code = [
        integer(0),
        Op::StoreLocal(1),
        Op::LoadLocal(1),
        integer(100_000),
        Op::Compare(LegacyComparison::Less),
        Op::Branch {
            condition: KernelBranchCondition::IfFalse,
            target: 14,
        },
        Op::LoadLocal(0),
        Op::RequireSwitchKey,
        Op::SwitchCompare {
            key: KernelSwitchKey::StringLabel(0),
            comparison: KernelSwitchComparison::Equal,
        },
        Op::Discard,
        Op::LoadLocal(1),
        Op::IntegerUpdate(LegacyIntegerUpdate::Increment),
        Op::StoreLocal(1),
        Op::Jump(2),
        Op::LoadLocal(1),
        Op::Return,
    ];
    let labels = [b"needle".to_vec()];
    let mut entry = definition(&code, 1, 2);
    entry.switch_labels = &labels;
    kernel_call_support::verify(&[entry])
}

#[test]
fn one_hundred_thousand_string_key_comparisons_use_bounded_scratch_beside_seventy_thousand_host_aliases()
 {
    let module = module();
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 1,
        max_slots: 1,
        ..StringHeapBudget::default()
    })
    .expect("One runtime input");
    let id = strings.allocate(b"needle", work).expect("Input").id;
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 1,
            max_object_slots: 1,
            max_array_values: 70_001,
            max_array_value_slots: 70_001,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Unrelated host values");
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Host data").value;
    objects
        .array_put_unique(
            host,
            ArrayKey::Index(1),
            KernelValue::String(id),
            &strings,
            work,
        )
        .expect("Host string graph");
    for _ in 0..70_000 {
        objects.copy_array_value(host, work).expect("Host alias");
    }
    let mut context = KernelHeapContext::new(
        &mut strings,
        &[],
        HeapWorkBudget {
            max_scratch_bytes: 4096,
            ..work
        },
    )
    .with_objects(&mut objects, &[]);
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::String(id)],
            KernelCallBudget {
                max_steps: 2_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("No owner-row history scan or runtime case cache");
    assert_eq!(
        result.value,
        KernelValue::Number(LegacyNumber::Integer(100_000))
    );
    assert!(context.usage().peak_scratch_bytes <= 4096);
    assert!(context.usage().steps < 1_000_000);
    assert_eq!(
        (
            context.usage().collections,
            context.usage().object_collections
        ),
        (0, 0)
    );
    assert_eq!(context.heap_usage().slots, 1);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        70_001
    );
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Host owners unchanged")
            .value,
        70_001
    );
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(1), &strings, work)
            .expect("Host payload unchanged")
            .value,
        Some(KernelValue::String(id))
    );
}
