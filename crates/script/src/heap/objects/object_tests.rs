// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Private tests inject generation exhaustion and forced hash collisions."
)]
use crate::execution::KernelValue;
use crate::heap::objects::field_index::{Bucket, FieldIndex};
use crate::heap::objects::object_slot::{Field, FieldSlot, ObjectState};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectId, ObjectResource,
    StringHeap, StringHeapBudget,
};

#[test]
fn exhausted_object_generation_retires_without_reusing_stale_identity() {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_object_slots: 1,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Objects");
    let original = objects
        .allocate(HeapWorkBudget::default())
        .expect("Original")
        .id;
    objects.objects[original.slot].generation = u64::MAX;
    let maximum = ObjectId {
        generation: u64::MAX,
        ..original
    };
    assert_eq!(objects.field_count(maximum), Ok(0));
    objects
        .reclaim(&[maximum], HeapWorkBudget::default())
        .expect("Maximum rooted generation");
    objects
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Retire maximum generation");
    assert!(matches!(
        objects.objects[original.slot].state,
        ObjectState::Retired
    ));
    assert_eq!(
        objects.field_count(maximum),
        Err(ObjectHeapErrorKind::StaleReference)
    );
    assert_eq!(
        objects.field_count(original),
        Err(ObjectHeapErrorKind::StaleReference)
    );
    let before = objects.usage();
    assert!(matches!(
        objects
            .allocate(HeapWorkBudget::default())
            .expect_err("Retired slot cannot wrap")
            .kind,
        ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::ObjectSlots,
            ..
        }
    ));
    assert_eq!(objects.usage(), before);
}

#[test]
fn forced_equal_hashes_charge_every_probe_and_content_comparison_before_exhaustion() {
    let mut index = FieldIndex {
        buckets: vec![Bucket::Empty; 256],
        ..FieldIndex::default()
    };
    let mut fields = Vec::new();
    for value in 0_u16..128 {
        fields.push(FieldSlot::Occupied(Field {
            owner: 0,
            hash: 0,
            key: value.to_le_bytes().to_vec(),
            value: KernelValue::Undefined,
            previous: None,
            next: None,
        }));
        index
            .insert(usize::from(value), usize::from(value))
            .expect("Forced collision chain");
    }
    let mut work = ObjectWork::new(HeapWorkBudget::default());
    assert_eq!(
        index
            .lookup(&fields, 0, 0, &127_u16.to_le_bytes(), &mut work)
            .expect("Last collision")
            .field,
        Some(127)
    );
    assert_eq!(work.usage().steps, 384);
    let mut work = ObjectWork::new(HeapWorkBudget {
        max_steps: 384,
        ..HeapWorkBudget::default()
    });
    assert!(matches!(
        index
            .lookup(&fields, 0, 0, &128_u16.to_le_bytes(), &mut work)
            .err(),
        Some(ObjectHeapErrorKind::BudgetExceeded {
            resource: ObjectResource::WorkSteps,
            requested: 385,
            maximum: 384
        })
    ));
    assert_eq!(work.usage().steps, 384);
    assert_eq!((index.live, index.filled), (128, 128));
    let mut work = ObjectWork::new(HeapWorkBudget::default());
    assert_eq!(
        index
            .lookup(&fields, 0, 0, &128_u16.to_le_bytes(), &mut work)
            .expect("Complete negative lookup")
            .field,
        None
    );
    assert_eq!(work.usage().steps, 385);
}
