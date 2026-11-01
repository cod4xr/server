// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers allocate distinct ordinary roots and one borrowed host game location."
)]
use cod4x_script::execution::KernelScriptRoots;
use cod4x_script::heap::{
    ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectId, StringHeap,
    StringHeapBudget,
};

pub struct State {
    pub strings: StringHeap,
    pub objects: ObjectHeap,
    pub level: ObjectId,
    pub receiver: ObjectId,
    pub animation: ObjectId,
    pub game: ArrayValueId,
}

pub fn state(budget: ObjectHeapBudget) -> State {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, budget).expect("Objects");
    let work = HeapWorkBudget::default();
    let level = objects.allocate(work).expect("Level").id;
    let receiver = objects.allocate(work).expect("Current receiver").id;
    let animation = objects.allocate(work).expect("Animation root").id;
    let game = objects.new_array_value(work).expect("Host game").value;
    State {
        strings,
        objects,
        level,
        receiver,
        animation,
        game,
    }
}

impl State {
    pub fn roots(&self) -> KernelScriptRoots {
        KernelScriptRoots::new(self.level, self.receiver, self.game, self.animation)
    }
}
