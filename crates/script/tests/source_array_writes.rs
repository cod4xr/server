// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile and execute real local array-building source."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn indexed_for_initializer_and_update_share_the_continue_destination_without_receiver_aliases() {
    let module =
        compile(b"main(){a=[];for(a[0]=0;a[0]<3;a[0]=a[0]+1){if(a[0]==1)continue;}return a[0];}");
    let mut state = State::new(ObjectHeapBudget {
        max_live_objects: 1,
        max_object_slots: 1,
        max_live_fields: 1,
        max_field_slots: 1,
        max_array_values: 2,
        max_array_value_slots: 2,
        ..ObjectHeapBudget::default()
    });
    assert_eq!(
        state
            .run(
                &module,
                &[],
                KernelCallBudget::default(),
                HeapWorkBudget::default()
            )
            .0
            .expect("Indexed clauses and continue")
            .value,
        number(3)
    );
    assert_eq!(
        (
            state.objects.usage().live_arrays,
            state.objects.usage().live_array_values
        ),
        (1, 0)
    );
}

#[test]
fn a_missing_parameter_is_initialized_undefined_and_can_autovivify() {
    let module = compile(b"main(a){a[0]=7;return a[0];}");
    let mut state = State::new(ObjectHeapBudget::default());
    assert_eq!(
        state
            .run(
                &module,
                &[],
                KernelCallBudget::default(),
                HeapWorkBudget::default()
            )
            .0
            .expect("Missing suffix argument")
            .value,
        number(7)
    );
    assert_eq!(state.objects.usage().live_array_values, 0);
}

#[test]
fn independent_and_explicitly_undefined_locals_build_and_return_through_indirect_calls() {
    let module = compile(
        b"main(){a=[];a[0]=7;a[\"key\"]=9;b=undefined;b[1]=3;
        f=::change;c=[[f]](a);return a[0]*100+c[0]*10+b[1]+a.size;}
        change(a){a[0]=4;return a;}",
    );
    let mut state = State::new(ObjectHeapBudget::default());
    let result = state
        .run(
            &module,
            &[],
            KernelCallBudget::default(),
            HeapWorkBudget::default(),
        )
        .0
        .expect("Source writes and COW across call");
    assert_eq!(result.value, number(745));
    assert_eq!(state.objects.usage().live_array_values, 0);
}

#[test]
fn array_elements_keep_ordinary_object_identity_runtime_strings_and_checked_function_tokens() {
    let module = compile(
        b"main(){a=[];o=spawnstruct();o.x=3;a[0]=o;a[1]=\"a\"+\"bc\";
        a[2]=::take;b=a;b[0].x=7;f=a[2];return a[0].x+a[1].size+[[f]]();}take(){return 11;}",
    );
    let mut state = State::new(ObjectHeapBudget::default());
    let result = state
        .run(
            &module,
            &[],
            KernelCallBudget::default(),
            HeapWorkBudget::default(),
        )
        .0
        .expect("Checked nonarray payload categories");
    assert_eq!(result.value, number(21));
    assert_eq!(state.objects.usage().live_array_values, 0);
}

#[test]
fn source_array_rhs_values_remain_rejected_without_publishing_nested_owner_edges() {
    let module = compile(b"main(){a=[];a[0]=[];return 0;}");
    let mut state = State::new(ObjectHeapBudget::default());
    let error = state
        .run(
            &module,
            &[],
            KernelCallBudget::default(),
            HeapWorkBudget::default(),
        )
        .0
        .expect_err("No stored/nested Array authority");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::ManagedArrayFieldUnsupported)
    );
    assert_eq!(
        (
            state.objects.usage().live_array_values,
            state.objects.usage().live_fields
        ),
        (0, 0)
    );
}
