// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Private tests force scratch-set collisions and a same-capacity index promotion."
)]
use crate::execution::KernelValue;
use crate::heap::objects::arrays::copy::copy_occupancy::CopyOccupancy;
use crate::heap::objects::field_index::Bucket;
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_slot::ObjectState;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, StringHeap,
    StringHeapBudget,
};

#[test]
fn sparse_copy_occupancy_meters_forced_collisions_and_reuses_its_allocation() {
    let mut work = ObjectWork::new(HeapWorkBudget::default());
    let mut set = CopyOccupancy::new(8, &mut work).expect("Scratch set");
    for position in [0, 8, 16, 24] {
        set.insert(position, &mut work).expect("Same start bucket");
    }
    assert_eq!(
        set.insert(8, &mut work),
        Err(ObjectHeapErrorKind::InvalidState)
    );
    for maximum in 0..=4 {
        let mut limited = ObjectWork::new(HeapWorkBudget {
            max_steps: maximum,
            max_scratch_bytes: 0,
        });
        let result = set.contains(24, &mut limited);
        if maximum < 4 {
            assert!(result.is_err());
            assert!(limited.usage().steps <= maximum);
        } else {
            assert_eq!(result, Ok(true));
        }
    }
    assert_eq!(set.contains(32, &mut work), Ok(false));
    set.clear(&mut work).expect("Reuse set allocation");
    assert_eq!(set.contains(24, &mut work), Ok(false));
    assert_eq!(
        CopyOccupancy::capacity(usize::MAX),
        Err(ObjectHeapErrorKind::SizeOverflow)
    );
}

#[test]
fn actual_new_empty_bucket_promotes_a_copy_to_same_capacity_rebuild_before_publication()
-> Result<(), &'static str> {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut heap = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Heap");
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Array").value;
    heap.array_put_unique(
        value,
        ArrayKey::Index(0),
        KernelValue::Undefined,
        &strings,
        budget,
    )
    .expect("Entry");
    let data = heap.array_data(value, budget).expect("Data").value;
    let ObjectState::Occupied {
        first: Some(slot), ..
    } = heap.objects[data.object.slot].state
    else {
        return Err("Expected field");
    };
    let original_bucket = heap
        .index
        .start(FieldKey::Array(ArrayKey::Index(0)).hash(data.object.slot))
        .expect("Existing bucket");
    let new_start = heap
        .index
        .start(FieldKey::Array(ArrayKey::Index(0)).hash(heap.objects.len()))
        .expect("Future owner bucket");
    let new_bucket = if new_start == original_bucket {
        (new_start + 1) & 7
    } else {
        new_start
    };
    heap.index.buckets.fill(Bucket::Empty);
    heap.index.buckets[original_bucket] = Bucket::Occupied(slot);
    let mut deleted = 0;
    for index in 0..8 {
        if index != original_bucket && index != new_bucket && deleted < 3 {
            heap.index.buckets[index] = Bucket::Deleted;
            deleted += 1;
        }
    }
    heap.index.filled = 4;
    let selected = heap.copy_array_value(value, budget).expect("Alias").value;
    heap.make_array_unique(selected, &strings, budget)
        .expect("Promote after actual fill preflight");
    assert_eq!(heap.index.buckets.len(), 8);
    assert_eq!((heap.index.live, heap.index.filled), (2, 2));
    let copied = heap
        .array_data(selected, budget)
        .expect("Copied data")
        .value;
    assert_eq!(
        heap.array_get(copied, ArrayKey::Index(0), &strings, budget)
            .expect("Copied lookup")
            .value,
        Some(KernelValue::Undefined)
    );
    assert_eq!(
        heap.array_get(data, ArrayKey::Index(0), &strings, budget)
            .expect("Old lookup")
            .value,
        Some(KernelValue::Undefined)
    );
    Ok(())
}
