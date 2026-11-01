// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers construct admitted synthetic object-store inputs."
)]
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectId, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

pub mod model;

pub fn stores(budget: ObjectHeapBudget) -> (StringHeap, ObjectHeap) {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("String owner");
    let objects = ObjectHeap::new(&strings, budget).expect("Object owner");
    (strings, objects)
}

pub fn allocate(objects: &mut ObjectHeap) -> ObjectId {
    objects
        .allocate(HeapWorkBudget::default())
        .expect("Object allocation")
        .id
}

pub fn number(value: i32) -> KernelValue {
    KernelValue::Number(LegacyNumber::Integer(value))
}
