// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests source sizes beyond native widths, complete binary keys and bounded managed read churn."
)]
use crate::array_read_support::number;
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

fn stores(budget: ObjectHeapBudget) -> (StringHeap, ObjectHeap) {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let objects = ObjectHeap::new(&strings, budget).expect("Objects");
    (strings, objects)
}

#[test]
fn source_sizes_exceed_u16_and_reads_preserve_complete_binary_keys_and_typed_u64_keys() {
    let module = module_compile_support::compile(
        b"main(a,key){return a.size+a[69999]+a[key];}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Wide read source");
    let work = HeapWorkBudget::default();
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Data").value;
    for index in 0..70_000_i32 {
        objects
            .array_put_unique(
                host,
                ArrayKey::Index(u64::try_from(index).expect("Index")),
                number(index),
                &strings,
                work,
            )
            .expect("Wide array");
    }
    let mut bytes = vec![b'x'; 70_000];
    bytes[1] = 0;
    bytes[69_999] = b'Y';
    let key = strings
        .allocate(&bytes, work)
        .expect("Complete binary key")
        .id;
    objects
        .array_put_unique(host, ArrayKey::Bytes(&bytes), number(17), &strings, work)
        .expect("Long NUL-bearing key");
    objects
        .array_put_unique(host, ArrayKey::Bytes(b"x"), number(91), &strings, work)
        .expect("Different visible prefix");
    objects
        .array_put_unique(host, ArrayKey::Index(u64::MAX), number(23), &strings, work)
        .expect("Wide typed key remains independent of legacy expression range");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Array(host), KernelValue::String(key)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("No count or key truncation");
    assert_eq!(result.value, number(70_003 + 69_999 + 17));
    drop(context);
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(u64::MAX), &strings, work)
            .expect("Unchanged typed key")
            .value,
        Some(number(23))
    );
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Host owner")
            .value,
        1
    );
}

#[test]
fn one_hundred_thousand_reads_reuse_two_vm_slots_beside_seventy_thousand_host_aliases() {
    let module = module_compile_support::compile(
        b"main(a){i=0;sum=0;while(i<100000){sum+=a[0];i++;}return sum;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Read churn");
    let work = HeapWorkBudget::default();
    let (mut strings, mut objects) = stores(ObjectHeapBudget {
        max_array_values: 70_003,
        max_array_value_slots: 70_003,
        ..ObjectHeapBudget::default()
    });
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Data").value;
    objects
        .array_put_unique(host, ArrayKey::Index(0), number(7), &strings, work)
        .expect("Read payload");
    for _ in 0..70_000 {
        objects.copy_array_value(host, work).expect("Host aliases");
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
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Array(host)],
            KernelCallBudget {
                max_steps: 4_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Bounded per-read work independent of host aliases");
    assert_eq!(result.value, number(700_000));
    assert!(context.usage().peak_scratch_bytes <= 4096);
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (usage.live_array_values, usage.array_value_slots),
        (70_001, 70_003)
    );
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Host owners")
            .value,
        70_001
    );
}
