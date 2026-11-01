// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests check populated copy order, isolation and unchanged primitive/shared identities."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    ArrayCowOutcome, ArrayKey, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapBudget,
    ObjectId, StringHeap,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn populated_detachment_reverses_order_preserves_payloads_and_isolates_the_selected_value() {
    let (mut strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let original = heap.new_array_value(budget).expect("Original").value;
    let original_data = heap
        .array_data(original, budget)
        .expect("Original data")
        .value;
    let object = allocate(&mut heap);
    let string = strings
        .allocate(b"PrivateCopiedPayload", budget)
        .expect("String")
        .id;
    let elements = [
        number(1),
        KernelValue::Object(object),
        KernelValue::String(string),
        KernelValue::LocalizedString(string),
        KernelValue::Undefined,
        KernelValue::Vector(LegacyVector::from_components([-0.0, 1.0, 2.0])),
    ];
    for (index, element) in elements.into_iter().enumerate() {
        heap.array_put_unique(
            original,
            ArrayKey::Index(index as u64),
            element,
            &strings,
            budget,
        )
        .expect("Populate");
    }
    let selected = heap
        .copy_array_value(original, budget)
        .expect("Selected alias")
        .value;
    assert_eq!(
        heap.make_array_unique(selected, &strings, budget)
            .expect("Populated detach")
            .value,
        ArrayCowOutcome::Detached
    );
    let copied = heap.array_data(selected, budget).expect("Fresh data").value;
    assert_ne!(copied, original_data);
    assert_eq!(
        heap.array_data(original, budget)
            .expect("Original identity unchanged")
            .value,
        original_data
    );
    assert_eq!(
        heap.array_owner_count(copied, budget)
            .expect("Unique copy")
            .value,
        1
    );
    assert_eq!(
        heap.array_owner_count(original_data, budget)
            .expect("One original owner")
            .value,
        1
    );
    let keys = heap.array_keys(copied, budget).expect("Reversed keys");
    for (index, key) in keys.value.into_iter().enumerate() {
        assert!(matches!(key, ArrayKey::Index(value) if value == index as u64));
    }
    for (index, element) in elements.into_iter().enumerate() {
        assert_eq!(
            heap.array_get(copied, ArrayKey::Index(index as u64), &strings, budget)
                .expect("Unchanged payload")
                .value,
            Some(element)
        );
    }
    heap.array_put_unique(selected, ArrayKey::Index(0), number(99), &strings, budget)
        .expect("Mutate copy");
    assert_eq!(
        heap.array_get(original_data, ArrayKey::Index(0), &strings, budget)
            .expect("Original unaffected")
            .value,
        Some(number(1))
    );
    assert_eq!(
        heap.make_array_unique(selected, &strings, budget)
            .expect("Second selection unique")
            .value,
        ArrayCowOutcome::Unchanged
    );
    reclaim_both_copies(&mut heap, &mut strings, original, selected, object);
}

fn reclaim_both_copies(
    heap: &mut ObjectHeap,
    strings: &mut StringHeap,
    original: ArrayValueId,
    selected: ArrayValueId,
    object: ObjectId,
) {
    let budget = HeapWorkBudget::default();
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Both arrays root shared struct")
            .objects,
        0
    );
    assert_eq!(
        heap.reclaim_strings(strings, &[], budget)
            .expect("Both string tags remain rooted")
            .strings,
        0
    );
    heap.release_array_value(original, budget)
        .expect("Release original");
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Collect only original")
            .objects,
        1
    );
    assert_eq!(heap.field_count(object), Ok(0));
    heap.release_array_value(selected, budget)
        .expect("Release copy");
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect("Collect copy and shared struct")
            .objects,
        2
    );
    assert_eq!(
        heap.reclaim_strings(strings, &[], budget)
            .expect("Collect copied payload")
            .strings,
        1
    );
}

#[test]
fn populated_copy_preserves_complete_binary_keys_and_exact_nan_and_vector_bits()
-> Result<(), &'static str> {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    let scalar = KernelValue::Number(LegacyNumber::Float(f32::from_bits(0x7fa1_2345)));
    let vector = KernelValue::Vector(LegacyVector::from_components([
        f32::from_bits(0x7fc0_1234),
        -0.0,
        f32::INFINITY,
    ]));
    heap.array_put_unique(
        value,
        ArrayKey::Bytes(&[0, 255, 1, 0]),
        scalar,
        &strings,
        budget,
    )
    .expect("Binary key/NaN value");
    heap.array_put_unique(value, ArrayKey::Index(u64::MAX), vector, &strings, budget)
        .expect("Wide key/vector value");
    let alias = heap.copy_array_value(value, budget).expect("Alias").value;
    heap.make_array_unique(alias, &strings, budget)
        .expect("Copy");
    let data = heap.array_data(alias, budget).expect("Copy data").value;
    let Some(KernelValue::Number(LegacyNumber::Float(number))) = heap
        .array_get(data, ArrayKey::Bytes(&[0, 255, 1, 0]), &strings, budget)
        .expect("Scalar copy")
        .value
    else {
        return Err("Expected copied float");
    };
    assert_eq!(number.to_bits(), 0x7fa1_2345);
    let Some(KernelValue::Vector(vector)) = heap
        .array_get(data, ArrayKey::Index(u64::MAX), &strings, budget)
        .expect("Vector copy")
        .value
    else {
        return Err("Expected copied vector");
    };
    assert_eq!(
        vector.components().map(f32::to_bits),
        [0x7fc0_1234, 0x8000_0000, 0x7f80_0000]
    );
    assert!(matches!(
        heap.array_keys(data, budget)
            .expect("Typed copy keys")
            .value
            .as_slice(),
        [ArrayKey::Bytes([0, 255, 1, 0]), ArrayKey::Index(u64::MAX)]
    ));
    Ok(())
}
