// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests constructor retry roots and complete failed work prefixes with cache, owner, caller and mark scratch."
)]
use crate::array_read_support::number;
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelHeapUsage, KernelModuleError, KernelScriptRoots,
    KernelValue,
};
use cod4x_script::heap::{
    ArrayId, ArrayKey, ArrayValueId, HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapBudget,
    ObjectHeapErrorKind, ObjectId, StringHeap, StringHeapBudget, StringId,
};

struct State {
    strings: StringHeap,
    objects: ObjectHeap,
    host: ArrayValueId,
    data: ArrayId,
    shared: ObjectId,
    world: ObjectId,
    held: StringId,
    world_string: StringId,
    garbage_string: StringId,
    garbage: ObjectId,
}

fn state() -> State {
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let held = strings
        .allocate(b"retained", work)
        .expect("Host payload")
        .id;
    let world_string = strings
        .allocate(b"world retained", work)
        .expect("World-only payload")
        .id;
    let garbage_string = strings
        .allocate(b"discarded", work)
        .expect("Garbage string")
        .id;
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 5,
            max_object_slots: 5,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Paired pressure heap");
    let host = objects.new_array_value(work).expect("Host source").value;
    let data = objects.array_data(host, work).expect("Host data").value;
    let shared = objects.allocate(work).expect("Shared struct").id;
    objects
        .put(
            shared,
            b"payload",
            KernelValue::String(held),
            &strings,
            work,
        )
        .expect("Shared graph");
    for (key, value) in [
        (ArrayKey::Bytes(b"object"), KernelValue::Object(shared)),
        (ArrayKey::Bytes(b"string"), KernelValue::String(held)),
    ] {
        objects
            .array_put_unique(host, key, value, &strings, work)
            .expect("Host entry");
    }
    let world = objects.allocate(work).expect("World-only ordinary root").id;
    objects
        .put(
            world,
            b"payload",
            KernelValue::String(world_string),
            &strings,
            work,
        )
        .expect("World-only string graph");
    let garbage = objects.allocate(work).expect("Garbage object").id;
    State {
        strings,
        objects,
        host,
        data,
        shared,
        world,
        held,
        world_string,
        garbage_string,
        garbage,
    }
}

fn run(
    module: &CompiledModule,
    budget: HeapWorkBudget,
) -> (Result<KernelValue, KernelModuleError>, KernelHeapUsage) {
    let State {
        mut strings,
        mut objects,
        host,
        data,
        shared,
        world,
        held,
        world_string,
        garbage_string,
        garbage,
    } = state();
    let work = HeapWorkBudget::default();
    let roots = KernelScriptRoots::new(world, world, host, world);
    let mut context = KernelHeapContext::new(&mut strings, &[], budget)
        .with_objects(&mut objects, &[])
        .with_script_roots(roots);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Array(host)],
            KernelCallBudget::default(),
            &mut context,
        )
        .map(|outcome| outcome.value);
    let usage = context.usage();
    assert!(usage.steps <= budget.max_steps);
    assert!(usage.peak_scratch_bytes <= budget.max_scratch_bytes);
    assert_eq!(context.get(held), Ok(b"retained".as_slice()));
    assert_eq!(context.get(world_string), Ok(b"world retained".as_slice()));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    if result.is_ok() {
        assert_eq!((usage.object_collections, usage.collections), (1, 1));
        assert_eq!(
            context.get(garbage_string),
            Err(HeapErrorKind::StaleReference)
        );
        assert_eq!(
            context.heap_usage().live_strings,
            4,
            "Host/world payloads and both cached keys survive"
        );
    }
    drop(context);
    if result.is_ok() {
        assert_eq!(
            objects.field_count(garbage),
            Err(ObjectHeapErrorKind::StaleReference)
        );
    }
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Host owner")
            .value,
        1
    );
    assert_eq!(
        objects
            .array_len(data, work)
            .expect("Host entries unchanged")
            .value,
        2
    );
    assert_eq!(objects.field_count(shared), Ok(1));
    assert_eq!(objects.field_count(world), Ok(1));
    (result, usage)
}

#[test]
fn private_branch_alias_cleanup_preserves_caller_world_cached_keys_and_read_result_graphs() {
    let module = module_compile_support::compile(
        b"main(host){if(1){alias=host;}o=host[\"object\"];s=host[\"string\"];host=undefined;
          empty=make();temp=make();return empty.size+o.payload.size+s.size;}
          make(){return [];}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Scoped alias before caller pressure");
    let (result, usage) = run(&module, HeapWorkBudget::default());
    assert_eq!(result.expect("Complete graph roots"), number(16));
    for maximum in 0..usage.steps {
        run(
            &module,
            HeapWorkBudget {
                max_steps: maximum,
                max_scratch_bytes: usage.peak_scratch_bytes,
            },
        )
        .0
        .expect_err("Every shorter scoped pressure prefix");
    }
}

#[test]
fn independent_empty_results_and_cached_keys_survive_callee_pressure_and_every_shorter_heap_scope()
{
    let module = module_compile_support::compile(
        b"main(host){o=host[\"object\"];s=host[\"string\"];host=undefined;
          empty=make();temp=make();return empty.size+o.payload.size+s.size;}
          make(){return [];}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Constructor pressure after read operands release");
    let (result, usage) = run(&module, HeapWorkBudget::default());
    assert_eq!(result.expect("One paired retry"), number(16));
    let exact = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.peak_scratch_bytes,
    };
    assert_eq!(
        run(&module, exact).0.expect("Exact combined scope"),
        number(16)
    );
    for maximum in 0..usage.steps {
        run(
            &module,
            HeapWorkBudget {
                max_steps: maximum,
                ..exact
            },
        )
        .0
        .expect_err("Every shorter heap-work prefix");
    }
    run(
        &module,
        HeapWorkBudget {
            max_scratch_bytes: exact.max_scratch_bytes - 1,
            ..exact
        },
    )
    .0
    .expect_err("One less overlapping cache/owner/caller/mark scratch");
}
