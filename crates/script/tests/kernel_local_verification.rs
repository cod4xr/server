// SPDX-License-Identifier: AGPL-3.0-or-later
//! Layout/local operands are admitted and checked even in unreachable instructions.
#![allow(
    clippy::expect_used,
    reason = "Tests own valid finite layouts and deliberate invalid operands."
)]

use cod4x_script::execution::{
    KernelErrorKind, KernelFunctionLayout, KernelInstruction, KernelResource,
    KernelVerificationBudget, VerifiedKernelProgram,
};

use crate::kernel_support;

#[test]
fn invalid_parameter_prefix_and_all_local_operands_fail_before_code_publication() {
    let error = KernelFunctionLayout::new(2, 1).expect_err("Invalid prefix");
    assert_eq!(
        error.kind,
        KernelErrorKind::InvalidFunctionLayout {
            parameters: 2,
            local_values: 1
        }
    );
    assert_eq!(error.instruction, None);
    let layout = KernelFunctionLayout::new(0, 5).expect("Layout");
    for instruction in [
        KernelInstruction::LoadLocal(5),
        KernelInstruction::StoreLocal(5),
        KernelInstruction::LoadLocal(usize::MAX),
    ] {
        let code = [
            kernel_support::integer(7),
            KernelInstruction::Return,
            instruction,
        ];
        let error = VerifiedKernelProgram::verify_function(
            &code,
            layout,
            KernelVerificationBudget::default(),
        )
        .expect_err("Unreachable local still checked");
        assert!(matches!(
            error.kind,
            KernelErrorKind::InvalidLocal {
                local_values: 5,
                ..
            }
        ));
        assert_eq!(error.instruction, Some(2));
        assert_eq!((error.work_steps, error.admitted_local_values), (3, 5));
    }
    let code = [
        kernel_support::integer(7),
        KernelInstruction::Return,
        KernelInstruction::StoreLocal(4),
    ];
    VerifiedKernelProgram::verify_function(&code, layout, KernelVerificationBudget::default())
        .expect("Unreachable stack shape need not execute; local operand is in range");
    assert!(matches!(
        VerifiedKernelProgram::verify(
            &[KernelInstruction::LoadLocal(0), KernelInstruction::Return],
            KernelVerificationBudget::default()
        )
        .expect_err("Zero-layout convenience")
        .kind,
        KernelErrorKind::InvalidLocal {
            slot: 0,
            local_values: 0
        }
    ));
    assert!(matches!(
        VerifiedKernelProgram::verify_function(
            &[
                KernelInstruction::StoreLocal(0),
                kernel_support::integer(0),
                KernelInstruction::Return
            ],
            layout,
            KernelVerificationBudget::default()
        )
        .expect_err("Local store consumes stack")
        .kind,
        KernelErrorKind::StackUnderflow { .. }
    ));
}

#[test]
fn layout_count_admission_is_independent_of_code_and_stack_storage() {
    let layout = KernelFunctionLayout::new(0, 65).expect("Beyond native local count");
    let code = [kernel_support::integer(7), KernelInstruction::Return];
    let exact = KernelVerificationBudget {
        max_local_values: 65,
        ..KernelVerificationBudget::default()
    };
    let proof = VerifiedKernelProgram::verify_function(&code, layout, exact).expect("Exact locals");
    assert_eq!(proof.usage().local_values, 65);
    let error = VerifiedKernelProgram::verify_function(
        &code,
        layout,
        KernelVerificationBudget {
            max_local_values: 64,
            ..exact
        },
    )
    .expect_err("One less local count");
    assert_eq!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::LocalValues,
            requested: 65,
            maximum: 64
        }
    );
    assert_eq!((error.work_steps, error.admitted_local_values), (0, 0));
}
