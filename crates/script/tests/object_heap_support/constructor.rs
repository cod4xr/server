// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers create finite rooted and unreachable constructor-pressure graphs."
)]
use crate::object_heap_support::allocate;
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectId, StringHeap, StringHeapBudget, StringId,
};

pub struct Pressure {
    pub strings: StringHeap,
    pub objects: ObjectHeap,
    pub root: ObjectId,
    pub garbage: ObjectId,
    pub held: StringId,
    pub discarded: StringId,
}

pub fn pressure() -> Pressure {
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let held = strings
        .allocate(b"held", work)
        .expect("Nested live value")
        .id;
    let discarded = strings
        .allocate(b"discarded", work)
        .expect("Nested garbage value")
        .id;
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 2,
            max_object_slots: 2,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Objects");
    let root = allocate(&mut objects);
    let garbage = allocate(&mut objects);
    for (id, key, value) in [
        (root, b"held".as_slice(), KernelValue::String(held)),
        (garbage, b"self", KernelValue::Object(garbage)),
        (garbage, b"value", KernelValue::LocalizedString(discarded)),
    ] {
        objects
            .put(id, key, value, &strings, work)
            .expect("Graph field");
    }
    Pressure {
        strings,
        objects,
        root,
        garbage,
        held,
        discarded,
    }
}
