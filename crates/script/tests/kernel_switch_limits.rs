// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Every runtime key comparison prefix must respect shared budgets without allocating label strings."
)]
use crate::kernel_switch_support;
use cod4x_script::execution::{
    KernelCallBudget, KernelCallOutcome, KernelHeapContext, KernelHeapUsage, KernelModuleError,
    KernelSwitchComparison, KernelSwitchKey, KernelValue, VerifiedKernelModule,
};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};

fn run(
    module: &VerifiedKernelModule,
    calls: KernelCallBudget,
    heap: HeapWorkBudget,
) -> (
    Result<KernelCallOutcome, KernelModuleError>,
    KernelHeapUsage,
) {
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 1,
        max_slots: 1,
        ..StringHeapBudget::default()
    })
    .expect("Input only");
    let id = strings
        .allocate(b"needle", HeapWorkBudget::default())
        .expect("Input")
        .id;
    let mut context = KernelHeapContext::new(&mut strings, &[], heap);
    let result = module.execute_with_heap(0, &[KernelValue::String(id)], calls, &mut context);
    let usage = context.usage();
    assert!(usage.steps <= heap.max_steps);
    assert!(usage.peak_scratch_bytes <= heap.max_scratch_bytes);
    assert_eq!(
        (
            context.heap_usage().live_strings,
            context.heap_usage().slots
        ),
        (1, 1)
    );
    (result, usage)
}

#[test]
fn key_validation_comparison_and_frame_setup_accept_exactly_and_fail_every_shorter_prefix() {
    let module = kernel_switch_support::module(
        KernelSwitchKey::StringLabel(0),
        KernelSwitchComparison::Equal,
        &[b"needle".to_vec()],
    );
    let (result, heap) = run(
        &module,
        KernelCallBudget::default(),
        HeapWorkBudget::default(),
    );
    let result = result.expect("Baseline");
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
    assert_eq!(
        heap.max_scratch_bytes, 0,
        "Case rows occupy no runtime cache slots"
    );
    assert_eq!(
        run(&module, calls, heap).0.expect("All exact scopes"),
        result
    );
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
    for maximum in 0..usage.steps {
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
    for maximum in 0..usage.setup_work {
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
    for resource in 0..6 {
        let mut short = calls;
        match resource {
            0 => short.max_frames -= 1,
            1 => short.max_frame_bytes -= 1,
            2 => short.max_stack_values -= 1,
            3 => short.max_stack_bytes -= 1,
            4 => short.max_local_values -= 1,
            _ => short.max_local_bytes -= 1,
        }
        run(&module, short, heap)
            .0
            .expect_err("One-less frame/operand/local resource");
    }
}
