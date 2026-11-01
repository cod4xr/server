// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests assemble finite valid typed modules and deliberate reference failures."
)]

use std::collections::HashSet;

use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyComparison, LegacyNumber};

#[test]
fn references_copy_through_locals_arguments_returns_and_indirect_calls() {
    let root = [
        integer(7),
        KernelInstruction::FunctionReference(1),
        KernelInstruction::StoreLocal(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::CallIndirect { arguments: 1 },
        KernelInstruction::Return,
    ];
    let identity = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let factory = [
        KernelInstruction::FunctionReference(1),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[
        definition(&root, 0, 1),
        definition(&identity, 1, 1),
        definition(&factory, 0, 0),
    ]);
    assert_eq!(
        module
            .execute(0, &[], KernelCallBudget::default())
            .expect("Indirect call")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
    let reference = module.function_reference(1).expect("Checked reference");
    let value = KernelValue::Function(reference);
    assert_eq!(
        module
            .execute(2, &[], KernelCallBudget::default())
            .expect("Returned reference")
            .value,
        value
    );
    assert_eq!(
        module
            .execute(1, &[value], KernelCallBudget::default())
            .expect("Copied reference argument")
            .value,
        value
    );
    assert_eq!(
        format!("{value:?} {reference:?}"),
        "KernelValue::Function(..) KernelFunctionReference(..)"
    );
    assert_eq!(module.function_reference(u64::MAX), None);
}

#[test]
fn same_index_in_foreign_or_recreated_modules_never_resolves_or_compares_equal() {
    let invoke = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::CallIndirect { arguments: 0 },
        KernelInstruction::Return,
    ];
    let leaf = [integer(8), KernelInstruction::Return];
    let old = kernel_call_support::verify(&[definition(&invoke, 1, 1), definition(&leaf, 0, 0)]);
    let reference = old.function_reference(1).expect("Old owner");
    assert_eq!(
        old.execute(
            0,
            &[KernelValue::Function(reference)],
            KernelCallBudget::default()
        )
        .expect("Original owner")
        .value,
        KernelValue::Number(LegacyNumber::Integer(8))
    );
    drop(old);
    let new = kernel_call_support::verify(&[definition(&invoke, 1, 1), definition(&leaf, 0, 0)]);
    let replacement = new.function_reference(1).expect("New owner");
    assert_ne!(reference, replacement);
    let error = new
        .execute(
            0,
            &[KernelValue::Function(reference)],
            KernelCallBudget::default(),
        )
        .expect_err("Dropped owner cannot resolve in new module");
    assert_eq!(error.cause.kind, KernelErrorKind::ForeignFunctionReference);
    assert_eq!(error.cause.instruction, Some(1));
    assert_eq!(error.cause.work_steps, 2);
    assert_eq!(error.peak_frames, 1);
    let compare = VerifiedKernelProgram::verify(
        &[
            KernelInstruction::Constant(KernelValue::Function(reference)),
            KernelInstruction::Constant(KernelValue::Function(replacement)),
            KernelInstruction::Compare(LegacyComparison::Equal),
            KernelInstruction::Return,
        ],
        KernelVerificationBudget::default(),
    )
    .expect("Opaque data may be compared without dereferencing");
    assert_eq!(
        compare
            .execute(KernelExecutionBudget::default())
            .expect("Distinct identity")
            .value,
        KernelValue::Number(LegacyNumber::Integer(0))
    );
}

#[test]
fn independent_parallel_module_construction_mints_unique_opaque_identities() {
    let references = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let code = [integer(1), KernelInstruction::Return];
                    (0..32)
                        .map(|_| {
                            kernel_call_support::verify(&[definition(&code, 0, 0)])
                                .function_reference(0)
                                .expect("Opaque reference")
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().expect("Worker"))
            .collect::<HashSet<_>>()
    });
    assert_eq!(references.len(), 256);
}
