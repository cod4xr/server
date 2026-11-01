// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Private tests corrupt internal key encoding to check collection preflight."
)]
use crate::execution::KernelValue;
use crate::heap::objects::object_slot::{FieldSlot, ObjectState};
use crate::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, StringHeap,
    StringHeapBudget,
};

#[test]
fn malformed_array_field_encoding_rejects_collection_before_any_sweep() -> Result<(), &'static str>
{
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut heap = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Heap");
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    heap.array_put_unique(
        value,
        ArrayKey::Index(3),
        KernelValue::Undefined,
        &strings,
        budget,
    )
    .expect("Raw entry");
    let ObjectState::Occupied {
        first: Some(index), ..
    } = heap.objects[data.object.slot].state
    else {
        return Err("Expected populated data");
    };
    let FieldSlot::Occupied(field) = &mut heap.fields[index] else {
        return Err("Expected field");
    };
    field.key[0] = 2;
    heap.release_array_value(value, budget)
        .expect("Make malformed data garbage");
    let before = heap.usage();
    assert_eq!(
        heap.reclaim(&[], budget)
            .expect_err("Preflight internal key invariant")
            .kind,
        ObjectHeapErrorKind::InvalidState
    );
    assert_eq!(heap.usage(), before);
    assert_eq!(
        heap.array_len(data, budget)
            .expect("No partial data sweep")
            .value,
        1
    );
    assert_eq!(
        heap.array_keys(data, budget)
            .expect_err("No partial key output")
            .kind,
        ObjectHeapErrorKind::InvalidState
    );
    Ok(())
}
