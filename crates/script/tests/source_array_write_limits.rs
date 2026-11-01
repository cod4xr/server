// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Source writes share exact budgets and exhausted cleanup while host data remains unchanged."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::compilation::CompiledModule;
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelHeapUsage, KernelModuleError, KernelValue,
};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget, ObjectHeapUsage};

#[derive(Clone, Copy, Default)]
struct Scope {
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
    objects: ObjectHeapBudget,
}

fn run(
    module: &CompiledModule,
    scope: Scope,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
    ObjectHeapUsage,
) {
    let work = HeapWorkBudget::default();
    let mut state = State::new(scope.objects);
    let host = state.objects.new_array_value(work).expect("Host").value;
    let data = state
        .objects
        .array_data(host, work)
        .expect("Host data")
        .value;
    for key in 0..3 {
        state
            .objects
            .array_put_unique(host, ArrayKey::Index(key), number(7), &state.strings, work)
            .expect("Host fields");
    }
    let (result, heap) = state.run(
        module,
        &[KernelValue::Array(host), number(3)],
        scope.calls,
        scope.heap,
    );
    let usage = state.objects.usage();
    assert_eq!(usage.live_array_values, 1);
    assert_eq!(
        state
            .objects
            .array_owner_count(data, work)
            .expect("Host only")
            .value,
        1
    );
    let keys = state.objects.array_keys(data, work).expect("Host order");
    assert!(matches!(
        keys.value.as_slice(),
        [ArrayKey::Index(2), ArrayKey::Index(1), ArrayKey::Index(0)]
    ));
    for key in 0..3 {
        assert_eq!(
            state
                .objects
                .array_get(data, ArrayKey::Index(key), &state.strings, work)
                .expect("Host payload unchanged")
                .value,
            Some(number(7))
        );
    }
    (result, heap, usage)
}

#[test]
fn source_cow_transient_children_calls_and_cleanup_admit_exactly_and_reject_every_shorter_prefix() {
    let module = compile(
        b"main(a,k){a[k]=99;discard=relay(a);return a.size;}
        relay(a){a[\"Transient\"]=(undefined);return a;}",
    );
    let (result, heap, objects) = run(&module, Scope::default());
    let result = result.expect("Baseline");
    assert_eq!(result.value, number(4));
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
            max_key_bytes: objects.key_bytes + 10,
            max_index_buckets: objects.index_buckets,
            max_metadata_bytes: objects.metadata_bytes,
            ..ObjectHeapBudget::default()
        },
    };
    assert_eq!(run(&module, exact).0.expect("All exact resources"), result);
    for maximum in 0..heap.steps {
        run(
            &module,
            Scope {
                heap: HeapWorkBudget {
                    max_steps: maximum,
                    ..exact.heap
                },
                ..exact
            },
        )
        .0
        .expect_err("Every shorter COW/child/publication/cleanup work prefix");
    }
    for maximum in 0..usage.steps {
        run(
            &module,
            Scope {
                calls: KernelCallBudget {
                    max_steps: maximum,
                    ..exact.calls
                },
                ..exact
            },
        )
        .0
        .expect_err("Every shorter source instruction prefix");
    }
    for maximum in 0..usage.setup_work {
        run(
            &module,
            Scope {
                calls: KernelCallBudget {
                    max_setup_work: maximum,
                    ..exact.calls
                },
                ..exact
            },
        )
        .0
        .expect_err("Every shorter call setup prefix");
    }
    for resource in 0..16 {
        let short = shorter(exact, resource);
        run(&module, short)
            .0
            .expect_err("One-less frame/storage/transient/scratch scope");
    }
}

fn shorter(scope: Scope, resource: u8) -> Scope {
    let mut short = scope;
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
    short
}
