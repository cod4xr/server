// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests preserve complete graphs through object-allocation pressure."
)]
use crate::kernel_call_support::{self, definition};
use crate::object_heap_support::constructor;
use crate::object_heap_support::{allocate, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::execution::{
    KernelBuiltin, KernelCallBudget, KernelExecutionBudget, KernelHeapContext, KernelInstruction,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn constructor_pressure_preserves_caller_stack_graphs_and_strings_before_popping_or_returning() {
    let caller = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::StoreLocal(0),
        KernelInstruction::Call {
            function: 1,
            arguments: 0,
        },
        KernelInstruction::Discard,
        KernelInstruction::Return,
    ];
    let worker = [
        KernelInstruction::Builtin {
            function: KernelBuiltin::SpawnStruct,
            arguments: 0,
        },
        KernelInstruction::Return,
    ];
    let module =
        kernel_call_support::verify(&[definition(&caller, 1, 1), definition(&worker, 0, 0)]);
    let mut state = constructor::pressure();
    let mut context = KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
        .with_objects(&mut state.objects, &[]);
    let result = module
        .execute_with_heap(
            0,
            &[KernelValue::Object(state.root)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Object pressure in callee");
    assert_eq!(result.value, KernelValue::Object(state.root));
    assert_eq!(context.get(state.held).expect("Nested root"), b"held");
    assert_eq!(
        context.get(state.discarded),
        Err(HeapErrorKind::StaleReference)
    );
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (1, 1)
    );
    assert_eq!(state.objects.field_count(state.root), Ok(1));
    assert_eq!(
        state.objects.field_count(state.garbage),
        Err(ObjectHeapErrorKind::StaleReference)
    );
}

#[test]
fn live_slot_and_metadata_pressure_each_reclaim_before_constructor_retries() {
    let (_, mut sample) = stores(ObjectHeapBudget::default());
    allocate(&mut sample);
    let record = sample.usage().metadata_bytes;
    let program = VerifiedKernelProgram::verify(
        &[
            KernelInstruction::Builtin {
                function: KernelBuiltin::SpawnStruct,
                arguments: 0,
            },
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect("Constructor");
    for budget in [
        ObjectHeapBudget {
            max_live_objects: 1,
            ..ObjectHeapBudget::default()
        },
        ObjectHeapBudget {
            max_object_slots: 1,
            ..ObjectHeapBudget::default()
        },
        ObjectHeapBudget {
            max_metadata_bytes: record,
            ..ObjectHeapBudget::default()
        },
    ] {
        let (mut strings, mut objects) = stores(budget);
        let old = allocate(&mut objects);
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        let result = program
            .execute_with_heap(&[], KernelExecutionBudget::default(), &mut context)
            .expect("Admitted reuse");
        assert!(matches!(result.value, KernelValue::Object(id) if id != old));
        assert_eq!(
            (
                context.usage().object_collections,
                context.usage().collections
            ),
            (1, 1)
        );
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Reused metadata")
                .object_slots,
            1
        );
        assert_eq!(
            objects.field_count(old),
            Err(ObjectHeapErrorKind::StaleReference)
        );
    }
}
