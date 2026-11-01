// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests admit only independent empty data and explicit managed owner transfer through real source."
)]
use crate::array_read_support::number;
use crate::module_compile_support;
use crate::package_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget, ScriptPackageBudget};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    ArrayKey, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap,
    StringHeapBudget,
};

fn compile(source: &[u8]) -> CompiledModule {
    module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Source construction")
}

fn stores() -> (StringHeap, ObjectHeap) {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    (strings, objects)
}

fn returned(value: KernelValue) -> Result<ArrayValueId, String> {
    match value {
        KernelValue::Array(id) => Ok(id),
        _ => Err("Expected transferred host owner".into()),
    }
}

#[test]
fn each_source_construction_is_independent_and_top_level_returns_transfer_one_host_owner()
-> Result<(), String> {
    let module = compile(
        b"main(){return relay([]);}relay(a){unused=a;f=::pick;return [[f]](a);}
        pick(a){return a;}",
    );
    let work = HeapWorkBudget::default();
    let (mut strings, mut objects) = stores();
    let host = objects
        .new_array_value(work)
        .expect("Unrelated host array")
        .value;
    objects
        .array_put_unique(host, ArrayKey::Index(0), number(7), &strings, work)
        .expect("Host data");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let first = returned(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("First invocation")
            .value,
    )?;
    let second = returned(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Reuse immutable code")
            .value,
    )?;
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        3
    );
    drop(context);
    let first_data = objects.array_data(first, work).expect("First data").value;
    let second_data = objects.array_data(second, work).expect("Second data").value;
    assert_ne!(first, second);
    assert_ne!(first_data, second_data);
    for data in [first_data, second_data] {
        assert_eq!(objects.array_len(data, work).expect("Empty").value, 0);
        assert_eq!(
            objects
                .array_owner_count(data, work)
                .expect("Transferred owner only")
                .value,
            1
        );
    }
    for id in [first, second] {
        objects
            .release_array_value(id, work)
            .expect("Explicit host release");
    }
    assert_eq!(
        objects
            .reclaim(&[], work)
            .expect("Only unowned empty data")
            .objects,
        2
    );
    let host_data = objects
        .array_data(host, work)
        .expect("Host untouched")
        .value;
    assert_eq!(
        objects
            .array_get(host_data, ArrayKey::Index(0), &strings, work)
            .expect("Host payload")
            .value,
        Some(number(7))
    );
    Ok(())
}

#[test]
fn empty_size_missing_reads_and_definedness_do_not_create_payload_edges_or_retain_vm_owners() {
    let module = compile(
        b"main(){a=[];b=a;return a.size+isdefined(b[0])
        +isdefined(a[\"missing\"])+isdefined(a);}",
    );
    let (mut strings, mut objects) = stores();
    let work = HeapWorkBudget::default();
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Read independent empty data");
    assert_eq!(result.value, number(1));
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!((usage.live_array_values, usage.live_fields), (0, 0));
    drop(context);
    assert_eq!(
        objects
            .reclaim(&[], work)
            .expect("Discard empty payload")
            .objects,
        1
    );
}

#[test]
fn missing_object_context_fails_only_at_executed_construction_and_strings_only_context_recovers() {
    let module = compile(b"main(){return [];}skipped(){if(0)return [];return 7;}");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect_err("Explicit object context required")
            .cause
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    assert_eq!(
        module
            .program()
            .execute(1, &[], KernelCallBudget::default())
            .expect("Skipped constructor")
            .value,
        number(7)
    );
    let (mut strings, _) = stores();
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
    for _ in 0..32 {
        assert_eq!(
            module
                .program()
                .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
                .expect_err("Strings alone cannot allocate array data")
                .cause
                .kind,
            KernelErrorKind::ObjectExecutionUnsupported
        );
        assert_eq!(
            module
                .program()
                .execute_with_heap(1, &[], KernelCallBudget::default(), &mut context)
                .expect("Context cleanup")
                .value,
            number(7)
        );
    }
}

#[test]
fn imported_and_qualified_indirect_constructors_use_one_package_and_release_dead_callees() {
    let package = package_compile_support::compile(&[
        (b"root", b"#include lib; main(){a=make();f=lib::make;b=[[f]]();return a.size+b.size+isdefined(a);}"),
        (b"lib", b"make(){return [];}")
    ], LegacyDeveloperMode::Disabled, ScriptPackageBudget::default()).expect("Empty array package");
    let (mut strings, mut objects) = stores();
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let entry = package.function_for_declaration(0, 0).expect("Entry");
    assert_eq!(
        package
            .program()
            .execute_with_heap(entry, &[], KernelCallBudget::default(), &mut context)
            .expect("Independent constructors across files")
            .value,
        number(1)
    );
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(
        (
            usage.live_array_values,
            usage.live_fields,
            usage.live_arrays
        ),
        (0, 0, 2)
    );
}
