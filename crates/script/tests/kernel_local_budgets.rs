// SPDX-License-Identifier: AGPL-3.0-or-later
//! Local storage admission, fuel, and checked huge-layout failures publish no partial result.
#![allow(
    clippy::expect_used,
    reason = "Tests own finite programs and explicit admitted layout boundaries."
)]

use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelInstruction,
    KernelResource, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};

use crate::kernel_support;

#[test]
fn local_count_bytes_stack_and_fuel_remain_separately_admitted_and_accounted() {
    let layout = KernelFunctionLayout::new(1, 2).expect("Layout");
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::StoreLocal(1),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Return,
    ];
    let program =
        VerifiedKernelProgram::verify_function(&code, layout, KernelVerificationBudget::default())
            .expect("Proof");
    let arguments = [KernelValue::Number(LegacyNumber::Integer(7))];
    let exact = KernelExecutionBudget {
        max_steps: 4,
        max_stack_values: 1,
        max_stack_bytes: size_of::<KernelValue>() as u64,
        max_local_values: 2,
        max_local_bytes: 2 * size_of::<Option<KernelValue>>() as u64,
    };
    let outcome = program
        .execute_with_arguments(&arguments, exact)
        .expect("Exact invocation");
    assert_eq!(
        (outcome.local_values, outcome.local_bytes),
        (2, exact.max_local_bytes)
    );
    for (resource, budget, admitted, work) in [
        (
            KernelResource::LocalValues,
            KernelExecutionBudget {
                max_local_values: 1,
                ..exact
            },
            0,
            0,
        ),
        (
            KernelResource::ExecutionLocalBytes,
            KernelExecutionBudget {
                max_local_bytes: exact.max_local_bytes - 1,
                ..exact
            },
            0,
            0,
        ),
        (
            KernelResource::StackValues,
            KernelExecutionBudget {
                max_stack_values: 0,
                ..exact
            },
            2,
            0,
        ),
        (
            KernelResource::ExecutionStackBytes,
            KernelExecutionBudget {
                max_stack_bytes: 0,
                ..exact
            },
            2,
            0,
        ),
        (
            KernelResource::ExecutionSteps,
            KernelExecutionBudget {
                max_steps: 3,
                ..exact
            },
            2,
            3,
        ),
        (
            KernelResource::ExecutionSteps,
            KernelExecutionBudget {
                max_steps: 0,
                ..exact
            },
            0,
            0,
        ),
    ] {
        let error = program
            .execute_with_arguments(&arguments, budget)
            .expect_err("Admission/fuel limit");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert_eq!(error.admitted_local_values, admitted);
        assert_eq!(error.work_steps, work);
        assert_eq!(arguments[0], KernelValue::Number(LegacyNumber::Integer(7)));
    }
}

#[test]
fn numeric_failure_after_local_initialization_keeps_work_and_original_arguments() {
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::StoreLocal(1),
        KernelInstruction::LoadLocal(1),
        kernel_support::integer(0),
        KernelInstruction::Divide,
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 2).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Numeric proof");
    let arguments = [KernelValue::Number(LegacyNumber::Integer(7))];
    let error = program
        .execute_with_arguments(&arguments, KernelExecutionBudget::default())
        .expect_err("Located numeric error");
    assert_eq!(
        error.kind,
        KernelErrorKind::Numeric(LegacyNumberError::DivisionByZero)
    );
    assert_eq!(error.instruction, Some(4));
    assert_eq!(
        (
            error.work_steps,
            error.peak_stack_values,
            error.admitted_local_values
        ),
        (5, 2, 2)
    );
    assert_eq!(arguments[0], KernelValue::Number(LegacyNumber::Integer(7)));
}

#[test]
fn huge_layout_rejects_host_size_or_byte_overflow_and_zero_fuel_without_allocating_slots() {
    match KernelFunctionLayout::new(0, u64::MAX) {
        Err(error) => {
            assert!(usize::try_from(u64::MAX).is_err());
            assert_eq!(error.kind, KernelErrorKind::SizeOverflow);
        }
        Ok(layout) => {
            let code = [kernel_support::integer(0), KernelInstruction::Return];
            let program = VerifiedKernelProgram::verify_function(
                &code,
                layout,
                KernelVerificationBudget {
                    max_local_values: u64::MAX,
                    ..KernelVerificationBudget::default()
                },
            )
            .expect("Layout proof allocates only small code/flow slots");
            let budget = KernelExecutionBudget {
                max_local_values: u64::MAX,
                max_local_bytes: u64::MAX,
                ..KernelExecutionBudget::default()
            };
            let error = program
                .execute(budget)
                .expect_err("Checked local bytes before allocation");
            assert_eq!(error.kind, KernelErrorKind::SizeOverflow);
            assert_eq!((error.work_steps, error.admitted_local_values), (0, 0));
            let zero = program
                .execute(KernelExecutionBudget {
                    max_steps: 0,
                    ..budget
                })
                .expect_err("Zero fuel before local processing");
            assert!(matches!(
                zero.kind,
                KernelErrorKind::BudgetExceeded {
                    resource: KernelResource::ExecutionSteps,
                    requested: 1,
                    maximum: 0
                }
            ));
            assert_eq!((zero.work_steps, zero.admitted_local_values), (0, 0));
        }
    }
}
