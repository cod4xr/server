// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests all shared mutation/call failure prefixes, transient child quotas and exact staged resources against host aliases."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelHeapContext, KernelHeapUsage, KernelInstruction,
    KernelModuleError, KernelValue, VerifiedKernelModule,
};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget, ObjectHeapUsage};

#[derive(Clone, Copy, Default)]
struct Scope {
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
    objects: ObjectHeapBudget,
}

fn module() -> VerifiedKernelModule {
    let entry = [
        integer(99),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::StoreArrayLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Call {
            function: 1,
            arguments: 1,
        },
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Size,
        KernelInstruction::Return,
    ];
    let callee = [
        KernelInstruction::Constant(KernelValue::Undefined),
        integer(77),
        KernelInstruction::StoreArrayLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Return,
    ];
    kernel_call_support::verify(&[definition(&entry, 2, 2), definition(&callee, 1, 1)])
}

fn run(
    module: &VerifiedKernelModule,
    scope: Scope,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
    ObjectHeapUsage,
) {
    let work = HeapWorkBudget::default();
    let (mut strings, mut objects) = stores(scope.objects);
    let host = objects.new_array_value(work).expect("Host").value;
    let data = objects.array_data(host, work).expect("Data").value;
    for key in 0..3 {
        objects
            .array_put_unique(
                host,
                ArrayKey::Index(key),
                number(i32::try_from(key).expect("Key")),
                &strings,
                work,
            )
            .expect("Retained host payload");
    }
    let mut context =
        KernelHeapContext::new(&mut strings, &[], scope.heap).with_objects(&mut objects, &[]);
    let result = module.execute_with_heap(
        0,
        &[KernelValue::Array(host), number(3)],
        scope.calls,
        &mut context,
    );
    let heap = context.usage();
    let usage = context.object_heap_usage().expect("Objects");
    assert!(heap.steps <= scope.heap.max_steps);
    assert!(heap.peak_scratch_bytes <= scope.heap.max_scratch_bytes);
    assert_eq!(usage.live_array_values, 1);
    drop(context);
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
            .expect("Unchanged host count")
            .value,
        3
    );
    let keys = objects.array_keys(data, work).expect("Host sibling order");
    assert!(matches!(
        keys.value.as_slice(),
        [ArrayKey::Index(2), ArrayKey::Index(1), ArrayKey::Index(0)]
    ));
    for key in 0..3 {
        assert_eq!(
            objects
                .array_get(data, ArrayKey::Index(key), &strings, work)
                .expect("Unchanged payload")
                .value,
            Some(number(i32::try_from(key).expect("Key")))
        );
    }
    (result, heap, usage)
}

#[test]
fn copied_typed_writes_admit_exact_transient_storage_and_cleanup_every_failed_work_and_call_prefix()
{
    let module = module();
    let (result, heap, objects) = run(&module, Scope::default());
    let result = result.expect("Baseline");
    assert_eq!(result.value, number(4));
    assert_eq!(objects.live_fields, 11);
    assert_eq!(
        objects.field_slots, 12,
        "Missing computed Undefined creates then removes a real child"
    );
    let usage = result.usage;
    let exact = Scope {
        calls: KernelCallBudget {
            max_steps: usage.steps,
            max_setup_work: usage.setup_work,
            max_frames: usage.peak_frames,
            max_frame_bytes: usage.peak_frame_bytes,
            max_stack_values: usage.peak_stack_values,
            max_stack_bytes: usage.peak_stack_bytes,
            max_local_values: usage.peak_local_values,
            max_local_bytes: usage.peak_local_bytes,
        },
        heap: HeapWorkBudget {
            max_steps: heap.steps,
            max_scratch_bytes: heap.peak_scratch_bytes,
        },
        objects: ObjectHeapBudget {
            max_live_objects: objects.live_objects,
            max_object_slots: objects.object_slots,
            max_array_values: objects.array_value_slots,
            max_array_value_slots: objects.array_value_slots,
            max_live_fields: objects.live_fields + 1,
            max_field_slots: objects.field_slots,
            max_key_bytes: objects.key_bytes + 9,
            max_index_buckets: objects.index_buckets,
            max_metadata_bytes: objects.metadata_bytes,
            ..ObjectHeapBudget::default()
        },
    };
    assert_eq!(run(&module, exact).0.expect("Every exact scope"), result);
    for maximum in 0..heap.steps {
        let mut short = exact;
        short.heap.max_steps = maximum;
        run(&module, short)
            .0
            .expect_err("Every shorter heap/COW/child/publication prefix");
    }
    for maximum in 0..usage.steps {
        let mut short = exact;
        short.calls.max_steps = maximum;
        run(&module, short)
            .0
            .expect_err("Every shorter instruction prefix");
    }
    for maximum in 0..usage.setup_work {
        let mut short = exact;
        short.calls.max_setup_work = maximum;
        run(&module, short)
            .0
            .expect_err("Every shorter setup prefix");
    }
    for resource in 0..16 {
        let mut short = exact;
        match resource {
            0 => short.calls.max_frames -= 1,
            1 => short.calls.max_frame_bytes -= 1,
            2 => short.calls.max_stack_values -= 1,
            3 => short.calls.max_stack_bytes -= 1,
            4 => short.calls.max_local_values -= 1,
            5 => short.calls.max_local_bytes -= 1,
            6 => short.heap.max_scratch_bytes -= 1,
            7 => short.objects.max_live_objects -= 1,
            8 => short.objects.max_object_slots -= 1,
            9 => short.objects.max_array_values -= 1,
            10 => short.objects.max_array_value_slots -= 1,
            11 => short.objects.max_live_fields -= 1,
            12 => short.objects.max_field_slots -= 1,
            13 => short.objects.max_key_bytes -= 1,
            14 => short.objects.max_index_buckets -= 1,
            _ => short.objects.max_metadata_bytes -= 1,
        }
        run(&module, short)
            .0
            .expect_err("One-less transient/storage/frame/scratch scope");
    }
}
