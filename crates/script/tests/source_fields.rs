// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute source field identity, deletion, values and original evaluation order."
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
fn source_field_aliases_case_missing_and_literal_or_computed_undefined_deletion_preserve_struct_identity()
 {
    let mut source = b"main(){obj=spawnstruct();alias=obj;alias.COUNT=4;obj.x=8;obj.x=undefined;obj.y=optional();return obj.count+isdefined(obj.x)+isdefined(obj.y)+obj.size;}optional(value){return value;}".to_vec();
    let module = module_compile_support::compile(
        &source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Source fields");
    source.fill(0);
    assert_eq!(module.usage().string_literals, 0);
    assert!(module.usage().field_names > 0);
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let outcome = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Shared fields");
    assert_eq!(outcome.value, KernelValue::Number(LegacyNumber::Integer(5)));
    assert_eq!(context.heap_usage().live_strings, 0);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("One shared struct")
            .live_objects,
        1
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Deletes are absent")
            .live_fields,
        1
    );
}

#[test]
fn source_field_rhs_runs_before_receiver_and_retains_old_aliases_across_nested_replacement() {
    let source = include_bytes!("../../../examples/gsc/field_demo.gsc");
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("RHS-before-receiver fields");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Original receiver order")
            .value,
        KernelValue::Number(LegacyNumber::Integer(71))
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Three independent shared objects")
            .live_fields,
        3
    );
}

#[test]
fn source_fields_preserve_plain_localized_vector_and_function_values_through_calls() {
    for (source, expected) in [
        (b"main(){o=spawnstruct();o.plain=\"Rust\";o.loc=&\"MENU\";o.vec=(1,2,3);return o.vec[2]+o.plain.size+isstring(o.plain)+isdefined(o.loc);}".as_slice(), LegacyNumber::Float(9.0)),
        (b"main(){o=spawnstruct();o.callback=::calculate;return [[o.callback]](3);}calculate(x){return x+4;}", LegacyNumber::Integer(7)),
    ] {
        let module = module_compile_support::compile(source, LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("Typed field values");
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default()).with_objects(&mut objects, &[]);
        assert_eq!(module.program().execute_with_heap(0, &[], KernelCallBudget::default(), &mut context).expect("Copied field contents").value, KernelValue::Number(expected));
    }
}
