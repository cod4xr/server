// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Enabled and discarded regions must release every VM owner on every exhausted runtime prefix."
)]
use crate::{
    array_read_support::number, source_developer_limits::SOURCE, source_developer_support::compile,
    source_scope_support,
};
use cod4x_script::binding::LegacyDeveloperMode::{Disabled, Enabled};
use cod4x_script::execution::KernelCallBudget;
use cod4x_script::heap::HeapWorkBudget;

#[test]
fn every_developer_execution_setup_heap_and_scratch_prefix_releases_all_vm_owners() {
    for mode in [Disabled, Enabled] {
        let module = compile(SOURCE, mode);
        let run = |calls, heap| source_scope_support::run(&module, 32, calls, heap);
        let baseline = run(KernelCallBudget::default(), HeapWorkBudget::default());
        let result = baseline.result.expect("Baseline");
        assert_eq!(result.value, number(1));
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
            max_steps: baseline.heap.steps,
            max_scratch_bytes: baseline.heap.peak_scratch_bytes,
        };
        assert_eq!(run(calls, heap).result.expect("All exact"), result);
        for maximum in 0..calls.max_steps {
            run(
                KernelCallBudget {
                    max_steps: maximum,
                    ..calls
                },
                heap,
            )
            .result
            .expect_err("Every shorter instruction prefix");
        }
        for maximum in 0..calls.max_setup_work {
            run(
                KernelCallBudget {
                    max_setup_work: maximum,
                    ..calls
                },
                heap,
            )
            .result
            .expect_err("Every shorter setup prefix");
        }
        for maximum in 0..heap.max_steps {
            run(
                calls,
                HeapWorkBudget {
                    max_steps: maximum,
                    ..heap
                },
            )
            .result
            .expect_err("Every shorter heap prefix");
        }
        for resource in 0..7 {
            let mut short = calls;
            let mut short_heap = heap;
            match resource {
                0 => short.max_frames -= 1,
                1 => short.max_frame_bytes -= 1,
                2 => short.max_stack_values -= 1,
                3 => short.max_stack_bytes -= 1,
                4 => short.max_local_values -= 1,
                5 => short.max_local_bytes -= 1,
                _ => short_heap.max_scratch_bytes -= 1,
            }
            run(short, short_heap)
                .result
                .expect_err("One less frame/value/byte/scratch resource");
        }
    }
}
