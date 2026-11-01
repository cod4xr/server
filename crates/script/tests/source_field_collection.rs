// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests retain shared cyclic/deep source graphs across bounded pressure collection."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn hundred_thousand_source_field_updates_reclaim_cycles_with_bounded_object_and_field_slots() {
    let source = b"main(){root=spawnstruct();root.loop=root;i=0;while(i<100000){garbage=spawnstruct();garbage.loop=garbage;root.value=i;i++;}return root.value;}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Cyclic source update loop");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 4,
            max_object_slots: 4,
            max_live_fields: 5,
            max_field_slots: 5,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget {
                max_steps: 5_000_000,
                ..KernelCallBudget::default()
            },
            &mut context,
        )
        .expect("Bounded cyclic churn");
    assert_eq!(
        result.value,
        KernelValue::Number(LegacyNumber::Integer(99_999))
    );
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(usage.object_slots, 4);
    assert!(usage.field_slots <= 5);
    assert!(context.usage().object_collections > 30_000);
    assert_eq!(
        context.usage().object_collections,
        context.usage().collections
    );
    assert_eq!(context.heap_usage().live_strings, 0);
}

#[test]
fn twelve_thousand_source_linked_structs_retain_graphs_and_final_object_roots_iteratively()
-> Result<(), String> {
    let source = b"main(){root=spawnstruct();cursor=root;i=0;while(i<12000){cursor.child=spawnstruct();cursor=cursor.child;i++;}cursor.value=7;return root;}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Deep source graph");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Flat graph construction");
    let KernelValue::Object(root) = result.value else {
        return Err("Expected graph root".into());
    };
    assert_eq!(
        objects
            .reclaim(&[root], HeapWorkBudget::default())
            .expect("Returned result is an explicit host root")
            .objects,
        0
    );
    let mut cursor = root;
    for _ in 0..12000 {
        let value = objects
            .get(cursor, b"child", &strings, HeapWorkBudget::default())
            .expect("Deep edge")
            .value;
        let Some(KernelValue::Object(child)) = value else {
            return Err("Missing linked object".into());
        };
        cursor = child;
    }
    assert_eq!(
        objects
            .get(cursor, b"value", &strings, HeapWorkBudget::default())
            .expect("Tail value")
            .value,
        Some(KernelValue::Number(LegacyNumber::Integer(7)))
    );
    assert_eq!(
        objects
            .reclaim(&[], HeapWorkBudget::default())
            .expect("Flat full graph release")
            .objects,
        12_001
    );
    Ok(())
}
