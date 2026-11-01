// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests complete local operand validation and distinct uninitialized-cell rejection."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::object_heap_support::stores;
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelInstruction,
    KernelModuleVerificationBudget, VerifiedKernelModule,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn writable_local_operands_are_checked_even_when_unreachable() {
    for instruction in [
        KernelInstruction::StoreArrayLocal(9),
        KernelInstruction::ClearArrayLocal(9),
    ] {
        let code = [
            KernelInstruction::Jump(2),
            instruction,
            integer(1),
            KernelInstruction::Return,
        ];
        let error = VerifiedKernelModule::verify(
            &[definition(&code, 0, 1)],
            KernelModuleVerificationBudget::default(),
        )
        .expect_err("Invalid unreachable local operand");
        assert_eq!(
            error.cause.kind,
            KernelErrorKind::InvalidLocal {
                slot: 9,
                local_values: 1
            }
        );
    }
}

#[test]
fn valid_but_uninitialized_local_targets_fail_without_autovivification() {
    let assign = [
        integer(7),
        integer(0),
        KernelInstruction::StoreArrayLocal(0),
        integer(0),
        KernelInstruction::Return,
    ];
    let clear = [
        integer(0),
        KernelInstruction::ClearArrayLocal(0),
        integer(0),
        KernelInstruction::Return,
    ];
    let module =
        kernel_call_support::verify(&[definition(&assign, 0, 1), definition(&clear, 0, 1)]);
    let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    for entry in 0..2 {
        let error = module
            .execute_with_heap(entry, &[], KernelCallBudget::default(), &mut context)
            .expect_err("No implicit creation from an uninitialized cell");
        assert_eq!(
            error.cause.kind,
            KernelErrorKind::UninitializedLocal { slot: 0 }
        );
        let usage = context.object_heap_usage().expect("Objects");
        assert_eq!((usage.live_objects, usage.live_array_values), (0, 0));
    }
}
