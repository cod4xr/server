// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute real bound GSC with checked host object arguments."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

#[test]
fn compiled_gsc_calls_and_loops_transport_host_objects_and_root_their_strings_under_pressure() {
    let mut source = b"main(obj){for(i=0;i<1000;i++){pressure(\"x\");}return forward(obj);}pressure(s){return s+s;}forward(x){return x;}".to_vec();
    let module = module_compile_support::compile(
        &source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Bound GSC calls and loops");
    source.fill(0);
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 3,
        max_slots: 3,
        ..StringHeapBudget::default()
    })
    .expect("Strings");
    let held = strings
        .allocate(b"PrivateNestedPayload", work)
        .expect("Nested string")
        .id;
    let garbage = strings.allocate(b"old", work).expect("Pressure").id;
    strings.allocate(b"other", work).expect("Full pool");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let root = objects.allocate(work).expect("Host object").id;
    objects
        .put(
            root,
            b"value",
            KernelValue::LocalizedString(held),
            &strings,
            work,
        )
        .expect("Nested localized value");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Object(root)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Real compiled object argument transport");
    assert_eq!(result.value, KernelValue::Object(root));
    assert!(context.usage().collections > 900);
    assert_eq!(
        context.usage().object_collections,
        context.usage().collections
    );
    assert_eq!(
        context.get(held).expect("Nested root"),
        b"PrivateNestedPayload"
    );
    assert_eq!(context.get(garbage), Err(HeapErrorKind::StaleReference));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Object store")
            .live_objects,
        1
    );
    assert!(!format!("{module:?} {result:?} {context:?}").contains("PrivateNestedPayload"));
}
