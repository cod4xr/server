// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Every failed entry/copy/guard prefix must release VM array owners through prepaid cleanup."
)]
use crate::kernel_switch_support;
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelHeapUsage, KernelModuleError,
    KernelSwitchComparison, KernelSwitchKey, KernelValue, VerifiedKernelModule,
};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

fn run(
    module: &VerifiedKernelModule,
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
) -> (KernelModuleError, KernelHeapUsage) {
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Host data").value;
    let payload = KernelValue::Number(LegacyNumber::Integer(77));
    objects
        .array_put_unique(host, ArrayKey::Index(0), payload, &strings, work)
        .expect("Host payload");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], heap).with_objects(&mut objects, &[]);
    let error = module
        .execute_with_heap(0, &[KernelValue::Array(host)], calls, &mut context)
        .expect_err("Array key rejects");
    let usage = context.usage();
    assert!(usage.steps <= heap.max_steps);
    assert!(usage.peak_scratch_bytes <= heap.max_scratch_bytes);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    drop(context);
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Only host remains")
            .value,
        1
    );
    assert_eq!(
        objects
            .array_get(data, ArrayKey::Index(0), &strings, work)
            .expect("Payload unchanged")
            .value,
        Some(payload)
    );
    assert_eq!(objects.usage().live_arrays, 1);
    (error, usage)
}

#[test]
fn rejected_array_keys_unwind_every_shorter_heap_instruction_and_setup_prefix() {
    let module = kernel_switch_support::module(
        KernelSwitchKey::Integer(0),
        KernelSwitchComparison::Equal,
        &[],
    );
    let (error, usage) = run(
        &module,
        KernelCallBudget::default(),
        HeapWorkBudget::default(),
    );
    assert_eq!(error.cause.kind, KernelErrorKind::ExpectedSwitchKey);
    let heap = HeapWorkBudget {
        max_steps: usage.steps,
        max_scratch_bytes: usage.peak_scratch_bytes,
    };
    assert_eq!(
        run(&module, KernelCallBudget::default(), heap).0.cause.kind,
        KernelErrorKind::ExpectedSwitchKey
    );
    for maximum in 0..usage.steps {
        run(
            &module,
            KernelCallBudget::default(),
            HeapWorkBudget {
                max_steps: maximum,
                ..heap
            },
        );
    }
    for maximum in 0..error.cause.work_steps {
        run(
            &module,
            KernelCallBudget {
                max_steps: maximum,
                ..KernelCallBudget::default()
            },
            heap,
        );
    }
    for maximum in 0..error.setup_work {
        run(
            &module,
            KernelCallBudget {
                max_setup_work: maximum,
                ..KernelCallBudget::default()
            },
            heap,
        );
    }
    let short = run(
        &module,
        KernelCallBudget::default(),
        HeapWorkBudget {
            max_scratch_bytes: heap.max_scratch_bytes - 1,
            ..heap
        },
    )
    .0;
    assert!(matches!(
        short.cause.kind,
        KernelErrorKind::BudgetExceeded { .. }
    ));
}
