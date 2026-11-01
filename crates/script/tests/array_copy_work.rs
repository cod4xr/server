// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust failed work/scratch prefixes for empty, patched, rebuilt and reused-slot copies."
)]
use crate::array_copy_support::{graph, unchanged};
use cod4x_script::heap::{
    ArrayCowOutcome, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind, ObjectResource,
};

#[test]
fn populated_copy_preflights_every_failed_work_prefix_and_exact_simultaneous_scratch() {
    for (width, spare) in [(0, 0), (2, 0), (4, 0), (2, 16), (1, 7)] {
        let mut sample = graph(width, spare, ObjectHeapBudget::default());
        let usage = sample
            .heap
            .make_array_unique(sample.selected, &sample.strings, HeapWorkBudget::default())
            .expect("Measured copy")
            .usage;
        for maximum in 0..=usage.steps {
            let mut state = graph(width, spare, ObjectHeapBudget::default());
            let before = state.heap.usage();
            let result = state.heap.make_array_unique(
                state.selected,
                &state.strings,
                HeapWorkBudget {
                    max_steps: maximum,
                    max_scratch_bytes: usage.scratch_bytes,
                },
            );
            if maximum < usage.steps {
                let error = result.expect_err("Every incomplete prefix");
                assert!(matches!(
                    error.kind,
                    ObjectHeapErrorKind::BudgetExceeded {
                        resource: ObjectResource::WorkSteps,
                        ..
                    }
                ));
                assert!(error.usage.steps <= maximum);
                unchanged(&state, before);
            } else {
                let result = result.expect("Exact work/scratch");
                assert_eq!(result.value, ArrayCowOutcome::Detached);
                assert_eq!(result.usage, usage);
            }
        }
        if usage.scratch_bytes > 0 {
            let mut state = graph(width, spare, ObjectHeapBudget::default());
            let before = state.heap.usage();
            assert!(matches!(
                state
                    .heap
                    .make_array_unique(
                        state.selected,
                        &state.strings,
                        HeapWorkBudget {
                            max_scratch_bytes: usage.scratch_bytes - 1,
                            ..HeapWorkBudget::default()
                        }
                    )
                    .expect_err("One less simultaneous scratch")
                    .kind,
                ObjectHeapErrorKind::BudgetExceeded {
                    resource: ObjectResource::ScratchBytes,
                    ..
                }
            ));
            unchanged(&state, before);
        }
    }
}
