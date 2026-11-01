// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers build admitted synthetic string execution inputs."
)]
use cod4x_script::execution::{
    KernelFunctionLayout, KernelInstruction, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget, StringId};

pub fn heap(maximum: u64) -> StringHeap {
    StringHeap::new(StringHeapBudget {
        max_live_strings: maximum,
        max_slots: maximum,
        ..StringHeapBudget::default()
    })
    .expect("Heap")
}

pub fn allocate(heap: &mut StringHeap, bytes: &[u8]) -> StringId {
    heap.allocate(bytes, HeapWorkBudget::default())
        .expect("Owned input")
        .id
}

pub fn binary(instruction: KernelInstruction) -> VerifiedKernelProgram {
    VerifiedKernelProgram::verify_function(
        &[
            KernelInstruction::LoadLocal(0),
            KernelInstruction::LoadLocal(1),
            instruction,
            KernelInstruction::Return,
        ],
        KernelFunctionLayout::new(2, 2).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Binary program")
}
