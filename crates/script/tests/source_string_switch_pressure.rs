// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Source dispatch must retain runtime keys through last-owner release, callee pressure and exhausted cleanup."
)]
use crate::{array_read_support::number, source_scope_support::compile};
use cod4x_script::compilation::CompiledModule;
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelHeapContext, KernelHeapUsage, KernelModuleError,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

fn run(
    module: &CompiledModule,
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
) {
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 3,
        max_slots: 3,
        ..StringHeapBudget::default()
    })
    .expect("Components and runtime key");
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 1,
            max_object_slots: 1,
            ..ObjectHeapBudget::default()
        },
    )
    .expect("Collection required");
    let mut context =
        KernelHeapContext::new(&mut strings, &[], heap).with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(0, &[], calls, &mut context);
    let usage = context.usage();
    assert!(usage.steps <= heap.max_steps);
    assert!(usage.peak_scratch_bytes <= heap.max_scratch_bytes);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        0
    );
    if let Ok(result) = &result {
        assert_eq!(result.value, number(42));
        assert_eq!((usage.object_collections, usage.collections), (1, 1));
        assert_eq!(
            context.heap_usage().slots,
            3,
            "Labels cannot require a fourth string slot"
        );
        assert_eq!(context.object_heap_usage().expect("Objects").live_arrays, 0);
    }
    (result, usage)
}

#[test]
fn source_stack_only_runtime_keys_survive_collection_and_all_shorter_execution_heap_and_setup_prefixes()
 {
    let module = compile(br#"main(){switch(pick(collect(),make()[0])){case "leftright":return 42;default:return 0;}}
        pick(ignore,k){return k;}collect(){return spawnstruct();}make(){a=[];a[0]="left"+"right";return a;}"#);
    let (result, heap) = run(
        &module,
        KernelCallBudget::default(),
        HeapWorkBudget::default(),
    );
    let result = result.expect("Key rooted while its source array is reclaimed");
    let usage = result.usage;
    let calls = KernelCallBudget {
        max_steps: usage.steps,
        max_setup_work: usage.setup_work,
        max_frames: usage.peak_frames,
        max_frame_bytes: usage.peak_frame_bytes,
        max_stack_values: usage.peak_stack_values,
        max_stack_bytes: usage.peak_stack_bytes,
        max_local_values: usage.peak_local_values,
        max_local_bytes: usage.peak_local_bytes,
    };
    let heap = HeapWorkBudget {
        max_steps: heap.steps,
        max_scratch_bytes: heap.peak_scratch_bytes,
    };
    assert_eq!(run(&module, calls, heap).0.expect("All exact"), result);
    for maximum in 0..heap.max_steps {
        run(
            &module,
            calls,
            HeapWorkBudget {
                max_steps: maximum,
                ..heap
            },
        )
        .0
        .expect_err("Every shorter heap prefix");
    }
    for maximum in 0..calls.max_steps {
        run(
            &module,
            KernelCallBudget {
                max_steps: maximum,
                ..calls
            },
            heap,
        )
        .0
        .expect_err("Every shorter instruction prefix");
    }
    for maximum in 0..calls.max_setup_work {
        run(
            &module,
            KernelCallBudget {
                max_setup_work: maximum,
                ..calls
            },
            heap,
        )
        .0
        .expect_err("Every shorter setup prefix");
    }
    run(
        &module,
        calls,
        HeapWorkBudget {
            max_scratch_bytes: heap.max_scratch_bytes - 1,
            ..heap
        },
    )
    .0
    .expect_err("One less scratch");
}
