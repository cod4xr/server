// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile ordinary struct size and definedness without inventing entity subtypes."
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
fn compiled_struct_size_and_definedness_work_through_calls_and_evaluated_extra_arguments() {
    let source = b"main(obj){return query(obj);}query(obj){return obj.size + isdefined(obj, (1,2,3), obj.size);} ";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Bound struct queries");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let id = objects
        .allocate(HeapWorkBudget::default())
        .expect("Ordinary struct")
        .id;
    let before = objects.usage();
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Object(id)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Real object query calls");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(2)));
    assert_eq!(context.object_heap_usage(), Some(before));
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (0, 0)
    );
}
