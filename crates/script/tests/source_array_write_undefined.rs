// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Literal and computed Undefined have different observable allocation paths."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn only_unwrapped_literal_undefined_clears_without_admitting_a_transient_child() {
    for rhs in ["undefined", "(undefined)", "missing()"] {
        for budget in [
            ObjectHeapBudget {
                max_live_fields: 0,
                ..ObjectHeapBudget::default()
            },
            ObjectHeapBudget {
                max_key_bytes: 0,
                ..ObjectHeapBudget::default()
            },
            ObjectHeapBudget {
                max_field_slots: 0,
                ..ObjectHeapBudget::default()
            },
        ] {
            let source = format!(
                "main(){{a=[];a[\"PrivateKey\"]={rhs};return a.size;}}missing(){{return;}}"
            );
            let mut state = State::new(budget);
            let result = state
                .run(
                    &compile(source.as_bytes()),
                    &[],
                    KernelCallBudget::default(),
                    HeapWorkBudget::default(),
                )
                .0;
            if rhs == "undefined" {
                assert_eq!(result.expect("No transient child").value, number(0));
            } else {
                result.expect_err("Computed lvalue needs transient storage");
            }
            assert_eq!(
                (
                    state.objects.usage().live_array_values,
                    state.objects.usage().live_fields
                ),
                (0, 0)
            );
        }
    }
}

#[test]
fn literal_clear_rejects_undefined_target_while_grouped_and_called_values_autovivify() {
    for rhs in ["undefined", "(undefined)", "missing()"] {
        let source = format!("main(){{a=undefined;a[0]={rhs};return a.size;}}missing(){{return;}}");
        let mut state = State::new(ObjectHeapBudget::default());
        let result = state
            .run(
                &compile(source.as_bytes()),
                &[],
                KernelCallBudget::default(),
                HeapWorkBudget::default(),
            )
            .0;
        if rhs == "undefined" {
            assert_eq!(
                result.expect_err("No literal autovivification").cause.kind,
                KernelErrorKind::ExpectedArrayTarget
            );
            assert_eq!(state.objects.usage().live_arrays, 0);
        } else {
            assert_eq!(result.expect("Computed autovivification").value, number(0));
            assert_eq!(
                (
                    state.objects.usage().live_arrays,
                    state.objects.usage().field_slots
                ),
                (1, 1)
            );
        }
        assert_eq!(state.objects.usage().live_array_values, 0);
    }
}

#[test]
fn literal_and_computed_removal_preserve_remaining_entries_and_final_size() {
    for rhs in ["undefined", "(undefined)", "missing()"] {
        let source = format!(
            "main(){{a=[];a[0]=9;a[1]=4;a[0]={rhs};return a[1]+a.size+isdefined(a[0]);}}missing(){{return;}}"
        );
        let mut state = State::new(ObjectHeapBudget::default());
        assert_eq!(
            state
                .run(
                    &compile(source.as_bytes()),
                    &[],
                    KernelCallBudget::default(),
                    HeapWorkBudget::default()
                )
                .0
                .expect("Source removal")
                .value,
            number(5)
        );
        assert_eq!(state.objects.usage().live_array_values, 0);
    }
}
