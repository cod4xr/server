// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests check every failed field-write work prefix and complete pending operand roots."
)]
use crate::kernel_call_support::definition;
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelInstruction, KernelResource,
    KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectId, StringHeap,
    StringId,
};

struct State {
    strings: StringHeap,
    objects: ObjectHeap,
    root: ObjectId,
    value: ObjectId,
    garbage: ObjectId,
    held: StringId,
}

fn state(budget: ObjectHeapBudget) -> State {
    let (mut strings, mut objects) = stores(budget);
    let held = strings
        .allocate(b"held bytes", HeapWorkBudget::default())
        .expect("Held payload")
        .id;
    let root = allocate(&mut objects);
    let value = allocate(&mut objects);
    let garbage = allocate(&mut objects);
    objects
        .put(
            root,
            b"held",
            KernelValue::String(held),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Root graph");
    objects
        .put(
            garbage,
            b"garbage!",
            number(1),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Unreachable field");
    if budget.max_index_buckets == 8 {
        for key in [b"secondG!", b"thirdG!!"] {
            objects
                .put(garbage, key, number(1), &strings, HeapWorkBudget::default())
                .expect("Fill modeled index to growth boundary");
        }
    }
    State {
        strings,
        objects,
        root,
        value,
        garbage,
        held,
    }
}

fn program() -> VerifiedKernelProgram {
    let code = [
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::StoreLocal(1),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::StoreLocal(0),
        KernelInstruction::StoreField(0),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let names = [b"newfield".to_vec()];
    let mut function = definition(&code, 2, 2);
    function.field_names = &names;
    VerifiedKernelProgram::verify_definition(function, KernelVerificationBudget::default())
        .expect("Operand-only roots")
}

#[test]
fn field_live_key_slot_index_and_metadata_pressure_each_reclaim_with_both_operands_rooted() {
    let sample = state(ObjectHeapBudget::default());
    let metadata = sample.objects.usage().metadata_bytes;
    let program = program();
    for budget in [
        ObjectHeapBudget {
            max_live_fields: 2,
            ..ObjectHeapBudget::default()
        },
        ObjectHeapBudget {
            max_key_bytes: 12,
            ..ObjectHeapBudget::default()
        },
        ObjectHeapBudget {
            max_field_slots: 2,
            ..ObjectHeapBudget::default()
        },
        ObjectHeapBudget {
            max_index_buckets: 8,
            ..ObjectHeapBudget::default()
        },
        ObjectHeapBudget {
            max_metadata_bytes: metadata,
            ..ObjectHeapBudget::default()
        },
    ] {
        let mut state = state(budget);
        let mut context =
            KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
                .with_objects(&mut state.objects, &[]);
        program
            .execute_with_heap(
                &[
                    KernelValue::Object(state.root),
                    KernelValue::Object(state.value),
                ],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect("Field write after reclaim");
        assert_eq!(
            (
                context.usage().object_collections,
                context.usage().collections
            ),
            (1, 1)
        );
        assert_eq!(
            context.get(state.held).expect("Receiver's nested payload"),
            b"held bytes"
        );
        assert_eq!(state.objects.field_count(state.value), Ok(0));
        assert_eq!(
            state.objects.field_count(state.garbage),
            Err(ObjectHeapErrorKind::StaleReference)
        );
        assert_eq!(
            state
                .objects
                .get(
                    state.root,
                    b"newfield",
                    &state.strings,
                    HeapWorkBudget::default()
                )
                .expect("Pending RHS survived")
                .value,
            Some(KernelValue::Object(state.value))
        );
        assert_eq!(
            (
                state.objects.usage().live_objects,
                state.objects.usage().live_fields
            ),
            (2, 2)
        );
    }
}

#[test]
fn all_insufficient_field_work_prefixes_and_scratch_fail_without_losing_reachable_state() {
    let program = program();
    let object_budget = ObjectHeapBudget {
        max_live_fields: 2,
        ..ObjectHeapBudget::default()
    };
    let mut measured = state(object_budget);
    let mut context = KernelHeapContext::new(&mut measured.strings, &[], HeapWorkBudget::default())
        .with_objects(&mut measured.objects, &[]);
    program
        .execute_with_heap(
            &[
                KernelValue::Object(measured.root),
                KernelValue::Object(measured.value),
            ],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect("Measured field pressure");
    let usage = context.usage();
    assert!(usage.peak_scratch_bytes > 0);
    let budgets = (0..usage.steps)
        .map(|max_steps| HeapWorkBudget {
            max_steps,
            ..HeapWorkBudget::default()
        })
        .chain([HeapWorkBudget {
            max_scratch_bytes: usage.peak_scratch_bytes - 1,
            ..HeapWorkBudget::default()
        }]);
    for budget in budgets {
        let mut state = state(object_budget);
        let mut context = KernelHeapContext::new(&mut state.strings, &[], budget)
            .with_objects(&mut state.objects, &[]);
        let error = program
            .execute_with_heap(
                &[
                    KernelValue::Object(state.root),
                    KernelValue::Object(state.value),
                ],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect_err("Incomplete field work/scratch");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded {resource: KernelResource::HeapWorkSteps | KernelResource::HeapScratchBytes, requested, maximum} if requested > maximum)
        );
        assert!(context.usage().steps <= budget.max_steps);
        assert!(context.usage().peak_scratch_bytes <= budget.max_scratch_bytes);
        assert_eq!(
            context
                .get(state.held)
                .expect("Nested bytes survive every failure"),
            b"held bytes"
        );
        assert_eq!(state.objects.field_count(state.root), Ok(1));
        assert_eq!(state.objects.field_count(state.value), Ok(0));
    }
    let mut state = state(object_budget);
    let mut context = KernelHeapContext::new(
        &mut state.strings,
        &[],
        HeapWorkBudget {
            max_steps: usage.steps,
            max_scratch_bytes: usage.peak_scratch_bytes,
        },
    )
    .with_objects(&mut state.objects, &[]);
    program
        .execute_with_heap(
            &[
                KernelValue::Object(state.root),
                KernelValue::Object(state.value),
            ],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect("Exact field work/scratch");
    assert_eq!(context.usage(), usage);
}
