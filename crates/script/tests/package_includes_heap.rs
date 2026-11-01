// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests preserve owned included code/maps and complete shared roots through pressure."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::ScriptPackageBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn included_code_names_literals_and_locations_survive_destruction_of_source_storage()
-> Result<(), String> {
    let mut name = b"PrivateRoot".to_vec();
    let mut root = b"#include lib;main(){f=::work;return [[f]]();}".to_vec();
    let mut lib = b"work(){return \"PrivatePayload\";}".to_vec();
    let package = compile(
        &[(&name, &root), (b"lib", &lib)],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Owned included image");
    let original = package.instruction_location(1, 0).expect("Source map");
    name.fill(0);
    root.fill(0);
    lib.fill(0);
    drop((name, root, lib));
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
    let result = package
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("No source lifetime coupling");
    let KernelValue::String(id) = result.value else {
        return Err("Expected included literal".into());
    };
    assert_eq!(
        context.get(id).expect("Owned literal bytes"),
        b"PrivatePayload"
    );
    assert_eq!(package.module_name(0), Some(b"privateroot".as_slice()));
    assert_eq!(package.instruction_location(1, 0), Some(original));
    assert!(!format!("{package:?} {result:?} {context:?}").contains("Private"));
    Ok(())
}

#[test]
fn imported_function_references_share_caller_and_cached_literal_roots_during_collection() {
    let package = compile(&[
        (b"root", b"#include lib;main(){r=spawnstruct();r.text=\"\";r.anchor=&\"MENU\";f=::append;i=0;while(i<1000){[[f]](r);i++;}return r.text.size+isdefined(r.anchor);}"),
        (b"lib", b"append(r){r.text+=\"x\";r.temp=spawnstruct();}"),
    ], Disabled, ScriptPackageBudget::default()).expect("Included heap operations");
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 5,
        max_slots: 5,
        max_payload_bytes: 4096,
        ..StringHeapBudget::default()
    })
    .expect("Small string heap");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 3,
            max_object_slots: 3,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Small object heap");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = package
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Imported callers/callees retain complete roots");
    assert_eq!(
        result.value,
        KernelValue::Number(LegacyNumber::Integer(1001))
    );
    assert!(context.usage().collections > 900);
    assert!(context.usage().object_collections > 900);
}
