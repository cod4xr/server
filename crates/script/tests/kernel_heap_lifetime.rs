// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests inject foreign/stale arguments without forging IDs."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelExecutionBudget, KernelHeapContext, KernelInstruction,
    KernelValue,
};
use cod4x_script::heap::HeapWorkBudget;

#[test]
fn foreign_stale_and_unrooted_host_copies_fail_explicitly_with_no_dangling_payload() {
    let code = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let module = kernel_call_support::verify(&[definition(&code, 1, 1)]);
    let mut heap = kernel_heap_support::heap(1);
    let stale = allocate(&mut heap, b"old");
    heap.reclaim(&[], HeapWorkBudget::default())
        .expect("Unrooted host copy");
    let valid = allocate(&mut heap, b"new");
    let mut foreign_heap = kernel_heap_support::heap(1);
    let foreign = allocate(&mut foreign_heap, b"foreign");
    for (id, kind) in [
        (stale, KernelErrorKind::StaleStringReference),
        (foreign, KernelErrorKind::ForeignStringReference),
    ] {
        let error = module
            .execute(0, &[KernelValue::String(id)], KernelCallBudget::default())
            .expect_err("Pure execution rejects string ownership");
        assert_eq!(error.cause.kind, KernelErrorKind::HeapContextRequired);
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        let error = module
            .execute_with_heap(
                0,
                &[KernelValue::String(id)],
                KernelCallBudget::default(),
                &mut context,
            )
            .expect_err("Entry validates handle");
        assert_eq!(error.cause.kind, kind);
        assert_eq!(error.cause.work_steps, 0);
        assert_eq!(context.usage().steps, 1);
        assert_eq!(context.get(valid).expect("No payload mutation"), b"new");
        assert!(!format!("{error:?}").contains("foreign\""));
    }
    let roots = [foreign];
    let mut context = KernelHeapContext::new(&mut heap, &roots, HeapWorkBudget::default());
    assert_eq!(
        module
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("Invalid external root")
            .cause
            .kind,
        KernelErrorKind::ForeignStringReference
    );
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    assert_eq!(
        module
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Missing argument remains Undefined")
            .value,
        KernelValue::Undefined
    );
}

#[test]
fn pure_standalone_execution_also_rejects_runtime_string_arguments() {
    let program = kernel_heap_support::binary(KernelInstruction::Compare(
        cod4x_script::legacy::LegacyComparison::Equal,
    ));
    let mut heap = kernel_heap_support::heap(1);
    let id = allocate(&mut heap, b"private");
    let arguments = [KernelValue::String(id), KernelValue::String(id)];
    assert_eq!(
        program
            .execute_with_arguments(&arguments, KernelExecutionBudget::default())
            .expect_err("No implicit heap")
            .kind,
        KernelErrorKind::HeapContextRequired
    );
    assert_eq!(
        program
            .execute_legacy_script_arguments(&arguments, KernelExecutionBudget::default())
            .expect_err("No implicit legacy heap")
            .kind,
        KernelErrorKind::HeapContextRequired
    );
}
