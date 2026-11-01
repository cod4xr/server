// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests preserve host-managed array data and shared struct/string graphs through paired pressure collection."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    ArrayKey, HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind,
    StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn array_builtin_operands_and_local_aliases_survive_paired_object_string_pressure() {
    let module = module_compile_support::compile(
        b"main(a){copy=a;temp=spawnstruct(a);return isdefined(copy);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Managed operands around allocation");
    let budget = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let held = strings
        .allocate(b"held by array's shared struct", budget)
        .expect("Held bytes")
        .id;
    let discarded = strings
        .allocate(b"unreachable string", budget)
        .expect("Garbage bytes")
        .id;
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 3,
            max_object_slots: 3,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Pressure store");
    let host = objects.new_array_value(budget).expect("Host").value;
    let data = objects.array_data(host, budget).expect("Data").value;
    let child = objects.allocate(budget).expect("Shared struct").id;
    objects
        .put(child, b"held", KernelValue::String(held), &strings, budget)
        .expect("Nested string");
    objects
        .array_put_unique(
            host,
            ArrayKey::Index(0),
            KernelValue::Object(child),
            &strings,
            budget,
        )
        .expect("Array graph");
    let garbage = objects.allocate(budget).expect("Garbage struct").id;
    let mut context =
        KernelHeapContext::new(&mut strings, &[], budget).with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Array(host)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Retry with managed operands");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (1, 1)
    );
    assert_eq!(
        context.get(held),
        Ok(b"held by array's shared struct".as_slice())
    );
    assert_eq!(context.get(discarded), Err(HeapErrorKind::StaleReference));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    drop(context);
    assert_eq!(
        objects.field_count(garbage),
        Err(ObjectHeapErrorKind::StaleReference)
    );
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(0), &strings, budget)
            .expect("Retained graph")
            .value,
        Some(KernelValue::Object(child))
    );
    assert_eq!(
        objects
            .array_owner_count(data, budget)
            .expect("Only host owner")
            .value,
        1
    );
}
