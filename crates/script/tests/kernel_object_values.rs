// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests admit host references only through object-aware contexts."
)]
use crate::kernel_call_support::{self, definition};
use crate::object_heap_support::{allocate, stores};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout,
    KernelHeapContext, KernelInstruction, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};

#[test]
fn checked_host_object_references_travel_through_program_locals_and_direct_indirect_calls() {
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let id = allocate(&mut objects);
    let identity = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let caller = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::FunctionReference(1),
        KernelInstruction::CallIndirect { arguments: 1 },
        KernelInstruction::Return,
    ];
    let direct = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Call {
            function: 0,
            arguments: 1,
        },
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[
        definition(&caller, 1, 1),
        definition(&identity, 1, 1),
        definition(&direct, 1, 1),
    ]);
    let program = VerifiedKernelProgram::verify_function(
        &identity,
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Proof");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let value = KernelValue::Object(id);
    assert_eq!(
        program
            .execute_with_heap(&[value], KernelExecutionBudget::default(), &mut context)
            .expect("Standalone transfer")
            .value,
        value
    );
    assert_eq!(
        module
            .execute_with_heap(2, &[value], KernelCallBudget::default(), &mut context)
            .expect("Direct plus indirect transfer")
            .value,
        value
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Object storage")
            .live_objects,
        1
    );
    assert_eq!(
        (
            context.usage().collections,
            context.usage().object_collections
        ),
        (0, 0)
    );
}

#[test]
fn object_context_rejects_foreign_stale_external_and_wrong_string_binding_before_frame_activation()
{
    let code = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let module = kernel_call_support::verify(&[definition(&code, 1, 1)]);
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let live = allocate(&mut objects);
    let dead = allocate(&mut objects);
    objects
        .reclaim(&[live], HeapWorkBudget::default())
        .expect("Create stale handle");
    let (mut other_strings, mut foreign) = stores(ObjectHeapBudget::default());
    let foreign_id = allocate(&mut foreign);
    for (id, expected) in [
        (dead, ObjectHeapErrorKind::StaleReference),
        (foreign_id, ObjectHeapErrorKind::ForeignReference),
    ] {
        for external in [false, true] {
            let roots = [id];
            let before = objects.usage();
            let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
                .with_objects(&mut objects, if external { &roots } else { &[] });
            let value = KernelValue::Object(if external { live } else { id });
            let error = module
                .execute_with_heap(0, &[value], KernelCallBudget::default(), &mut context)
                .expect_err("Invalid object inventory");
            assert_eq!(error.cause.kind, KernelErrorKind::ObjectHeap(expected));
            // Admission is counted before reference validation, without executing a frame.
            assert_eq!(error.peak_frames, 1);
            assert_eq!(error.cause.work_steps, 0);
            assert_eq!(error.cause.instruction, None);
            assert_eq!(context.object_heap_usage(), Some(before));
        }
    }
    let mut context = KernelHeapContext::new(&mut other_strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let error = module
        .execute_with_heap(
            0,
            &[KernelValue::Object(live)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("Wrong associated string owner");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::ForeignStringHeap)
    );
    assert_eq!(error.peak_frames, 1);
    assert_eq!(error.cause.work_steps, 0);
    assert_eq!(error.cause.instruction, None);
}
