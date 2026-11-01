// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests root read results and shared graphs through allocation pressure and exhausted cleanup budgets."
)]
use crate::array_read_support::number;
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelHeapUsage};
use cod4x_script::execution::{KernelModuleError, KernelValue};
use cod4x_script::heap::{
    ArrayKey, HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind,
    StringHeap, StringHeapBudget,
};

fn run(
    module: &CompiledModule,
    budget: HeapWorkBudget,
) -> (Result<KernelValue, KernelModuleError>, KernelHeapUsage) {
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let held = strings.allocate(b"retained", work).expect("Held string").id;
    let garbage_string = strings
        .allocate(b"unreachable", work)
        .expect("Garbage string")
        .id;
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 3,
            max_object_slots: 3,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Pressure heap");
    let host = objects.new_array_value(work).expect("Host array").value;
    let data = objects.array_data(host, work).expect("Data").value;
    let shared = objects.allocate(work).expect("Shared object").id;
    objects
        .put(
            shared,
            b"payload",
            KernelValue::String(held),
            &strings,
            work,
        )
        .expect("Shared graph");
    objects
        .array_put_unique(
            host,
            ArrayKey::Index(0),
            KernelValue::Object(shared),
            &strings,
            work,
        )
        .expect("Shared result");
    objects
        .array_put_unique(
            host,
            ArrayKey::Index(1),
            KernelValue::String(held),
            &strings,
            work,
        )
        .expect("String result");
    let garbage = objects.allocate(work).expect("Unreachable object").id;
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
        .map(|outcome| outcome.value);
    let usage = context.usage();
    assert!(usage.steps <= budget.max_steps);
    assert!(usage.peak_scratch_bytes <= budget.max_scratch_bytes);
    assert_eq!(context.get(held), Ok(b"retained".as_slice()));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    if result.is_ok() {
        assert_eq!((usage.collections, usage.object_collections), (1, 1));
        assert_eq!(
            context.get(garbage_string),
            Err(HeapErrorKind::StaleReference)
        );
    }
    drop(context);
    if result.is_ok() {
        assert_eq!(
            objects.field_count(garbage),
            Err(ObjectHeapErrorKind::StaleReference)
        );
    }
    assert_eq!(objects.field_count(shared), Ok(1));
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Host owner survives")
            .value,
        1
    );
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(0), &strings, work)
            .expect("Unchanged shared entry")
            .value,
        Some(KernelValue::Object(shared))
    );
    (result, usage)
}

#[test]
fn source_read_results_survive_callee_pressure_with_exact_and_every_shorter_heap_scope() {
    let module = module_compile_support::compile(
        b"main(a){o=a[0];s=a[1];a=undefined;temp=allocate();return o.payload.size+s.size;}
          allocate(){return spawnstruct();}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Read operands consumed before callee pressure");
    let (result, usage) = run(&module, HeapWorkBudget::default());
    assert_eq!(result.expect("Pressure retry"), number(16));
    let exact = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.peak_scratch_bytes,
    };
    assert_eq!(
        run(&module, exact).0.expect("Exact combined scratch"),
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
    .expect_err("One less combined registry/cache/root/mark scratch");
}
