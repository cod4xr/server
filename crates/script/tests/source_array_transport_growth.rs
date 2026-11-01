// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust every work prefix across multiple registry growth stages, including pending-owner admission."
)]
use std::fmt::Write;

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelHeapUsage, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

fn execute(module: &CompiledModule, budget: HeapWorkBudget) -> (bool, KernelHeapUsage) {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let host = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let data = objects
        .array_data(host, HeapWorkBudget::default())
        .expect("Data")
        .value;
    let mut context =
        KernelHeapContext::new(&mut strings, &[], budget).with_objects(&mut objects, &[]);
    let result = module.program().execute_with_heap(
        0,
        &[KernelValue::Array(host)],
        KernelCallBudget::default(),
        &mut context,
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1,
        "{result:?}"
    );
    let usage = context.usage();
    assert!(usage.steps <= budget.max_steps);
    assert!(usage.peak_scratch_bytes <= budget.max_scratch_bytes);
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, HeapWorkBudget::default())
            .expect("Host owner")
            .value,
        1
    );
    (result.is_ok(), usage)
}

#[test]
fn every_insufficient_growth_rehash_probe_and_pending_owner_work_prefix_unwinds_without_fuel() {
    let mut source = String::from("main(a){");
    for index in 0..48 {
        write!(source, "p{index}=a;").expect("Source buffer");
    }
    source.push_str("return isdefined(p47);}");
    let module = module_compile_support::compile(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Growth source");
    let (success, baseline) = execute(&module, HeapWorkBudget::default());
    assert!(success);
    let exact = HeapWorkBudget {
        max_steps: baseline.steps,
        max_scratch_bytes: baseline.peak_scratch_bytes,
    };
    assert!(execute(&module, exact).0);
    for maximum in 0..baseline.steps {
        assert!(
            !execute(
                &module,
                HeapWorkBudget {
                    max_steps: maximum,
                    ..exact
                }
            )
            .0,
            "{maximum}"
        );
    }
    assert!(
        !execute(
            &module,
            HeapWorkBudget {
                max_scratch_bytes: exact.max_scratch_bytes - 1,
                ..exact
            }
        )
        .0
    );
}
