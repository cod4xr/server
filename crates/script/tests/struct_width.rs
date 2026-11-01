// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests retain live constructors beyond native parent-pool widths."
)]
use crate::object_heap_support::stores;
use cod4x_script::execution::{
    KernelBuiltin, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn seventy_thousand_simultaneous_structs_remain_distinct_and_live_in_wide_vm_locals() {
    let count = 70_000;
    let mut code = Vec::new();
    for slot in 0..count {
        code.push(KernelInstruction::Builtin {
            function: KernelBuiltin::SpawnStruct,
            arguments: 0,
        });
        code.push(KernelInstruction::StoreLocal(slot));
    }
    code.extend([
        KernelInstruction::LoadLocal(count - 1),
        KernelInstruction::Return,
    ]);
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(0, count as u64).expect("Wide layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Wide constructor proof");
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = program
        .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
        .expect("Wide live constructors");
    assert!(matches!(result.value, KernelValue::Object(_)));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (usage.live_objects, usage.object_slots, usage.live_fields),
        (count as u64, count as u64, 0)
    );
    assert_eq!(context.usage().object_collections, 0);
    assert_eq!(
        objects
            .reclaim(&[], HeapWorkBudget::default())
            .expect("No host roots remain")
            .objects,
        count as u64
    );
}
