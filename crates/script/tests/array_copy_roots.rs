// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests stop copying at shared struct identity and reject invalid payload references atomically."
)]
use crate::array_copy_support::{graph, unchanged};
use crate::kernel_call_support::{definition, verify};
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::{KernelCallBudget, KernelInstruction, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn array_copy_stops_at_shared_identity_even_when_it_reaches_a_twelve_thousand_struct_cycle() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let source = heap.new_array_value(budget).expect("Source").value;
    let first = allocate(&mut heap);
    let mut previous = first;
    for _ in 1..12_000 {
        let next = allocate(&mut heap);
        heap.put(
            previous,
            b"next",
            KernelValue::Object(next),
            &strings,
            budget,
        )
        .expect("Chain");
        previous = next;
    }
    heap.put(
        previous,
        b"next",
        KernelValue::Object(first),
        &strings,
        budget,
    )
    .expect("Close cycle");
    heap.array_put_unique(
        source,
        ArrayKey::Index(0),
        KernelValue::Object(first),
        &strings,
        budget,
    )
    .expect("Shared graph reference");
    let alias = heap.copy_array_value(source, budget).expect("Alias").value;
    let copied = heap
        .make_array_unique(
            alias,
            &strings,
            HeapWorkBudget {
                max_steps: 512,
                max_scratch_bytes: 1024,
            },
        )
        .expect("One entry copy must not traverse/clone shared struct graphs");
    assert!(copied.usage.steps <= 512 && copied.usage.scratch_bytes <= 1024);
    let data = heap.array_data(alias, budget).expect("Copy data").value;
    assert_eq!(
        heap.array_get(data, ArrayKey::Index(0), &strings, budget)
            .expect("Same graph identity")
            .value,
        Some(KernelValue::Object(first))
    );
    heap.release_array_value(source, budget)
        .expect("Release original");
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Retain graph through copy")
            .objects,
        1
    );
    heap.release_array_value(alias, budget)
        .expect("Release copy");
    let collection = heap
        .reclaim(&[], budget)
        .expect("Iterative graph reclamation");
    assert_eq!((collection.objects, collection.fields), (12_001, 12_001));
}

#[test]
fn foreign_string_store_and_stale_stored_string_reject_before_copy_publication() {
    let mut state = graph(2, 0, ObjectHeapBudget::default());
    let (foreign_strings, _) = stores(ObjectHeapBudget::default());
    let before = state.heap.usage();
    assert_eq!(
        state
            .heap
            .make_array_unique(state.selected, &foreign_strings, HeapWorkBudget::default())
            .expect_err("Foreign store")
            .kind,
        ObjectHeapErrorKind::ForeignStringHeap
    );
    unchanged(&state, before);
    let (mut strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Source").value;
    let string = strings
        .allocate(b"PrivateStalePayload", budget)
        .expect("String")
        .id;
    heap.array_put_unique(
        value,
        ArrayKey::Bytes(b"PrivateKey"),
        KernelValue::String(string),
        &strings,
        budget,
    )
    .expect("String entry");
    let alias = heap.copy_array_value(value, budget).expect("Alias").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    strings
        .reclaim(&[], budget)
        .expect("Deliberately invalidate stored string by omitting its required roots");
    let before = heap.usage();
    let error = heap
        .make_array_unique(alias, &strings, budget)
        .expect_err("Stale nested string");
    assert!(matches!(error.kind, ObjectHeapErrorKind::StringHeap(_)));
    assert!(!format!("{error:?} {error}").contains("Private"));
    assert_eq!(heap.usage(), before);
    assert_eq!(
        heap.array_data(alias, budget)
            .expect("Same selected data")
            .value,
        data
    );
    assert_eq!(
        heap.array_owner_count(data, budget)
            .expect("No partial owner decrement")
            .value,
        2
    );
}

#[test]
fn copied_function_references_preserve_module_identity_without_granting_lifetime_or_host_authority()
{
    let code = [
        KernelInstruction::FunctionReference(0),
        KernelInstruction::Return,
    ];
    let module = verify(&[definition(&code, 0, 0)]);
    let function = module
        .execute(0, &[], KernelCallBudget::default())
        .expect("Function snapshot")
        .value;
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Array").value;
    heap.array_put_unique(
        value,
        ArrayKey::Bytes(b"function"),
        function,
        &strings,
        budget,
    )
    .expect("Function entry");
    heap.array_put_unique(value, ArrayKey::Index(0), number(1), &strings, budget)
        .expect("Other entry");
    let alias = heap.copy_array_value(value, budget).expect("Alias").value;
    heap.make_array_unique(alias, &strings, budget)
        .expect("Copy");
    drop(module);
    let data = heap.array_data(alias, budget).expect("Copy data").value;
    assert_eq!(
        heap.array_get(data, ArrayKey::Bytes(b"function"), &strings, budget)
            .expect("Same nonowning opaque reference")
            .value,
        Some(function)
    );
}
