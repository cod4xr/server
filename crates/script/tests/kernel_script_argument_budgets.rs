// SPDX-License-Identifier: AGPL-3.0-or-later
//! Legacy defaults use existing local/stack/fuel admission without a padded argument array.
#![allow(
    clippy::expect_used,
    reason = "Tests own finite proofs and exact admission boundaries."
)]

use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelInstruction,
    KernelResource, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};

#[test]
fn missing_suffix_binding_keeps_independent_local_stack_and_fuel_limits() {
    let code = [KernelInstruction::LoadLocal(2), KernelInstruction::Return];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(3, 5).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Proof");
    let exact = KernelExecutionBudget {
        max_steps: 2,
        max_stack_values: 1,
        max_stack_bytes: size_of::<KernelValue>() as u64,
        max_local_values: 5,
        max_local_bytes: 5 * size_of::<Option<KernelValue>>() as u64,
    };
    assert_eq!(
        program
            .execute_legacy_script_arguments(&[], exact)
            .expect("Exact defaults")
            .value,
        KernelValue::Undefined
    );
    for (resource, budget, locals, work) in [
        (
            KernelResource::LocalValues,
            KernelExecutionBudget {
                max_local_values: 4,
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
            5,
            0,
        ),
        (
            KernelResource::ExecutionStackBytes,
            KernelExecutionBudget {
                max_stack_bytes: 0,
                ..exact
            },
            5,
            0,
        ),
        (
            KernelResource::ExecutionSteps,
            KernelExecutionBudget {
                max_steps: 1,
                ..exact
            },
            5,
            1,
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
            .execute_legacy_script_arguments(&[], budget)
            .expect_err("Admission boundary");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded { resource: actual, requested, maximum } if actual == resource && requested > maximum)
        );
        assert_eq!(
            (error.admitted_local_values, error.work_steps),
            (locals, work)
        );
    }
}

#[test]
fn huge_missing_prefix_fails_host_address_or_checked_bytes_before_storage() {
    match KernelFunctionLayout::new(u64::MAX, u64::MAX) {
        Err(error) => {
            assert!(usize::try_from(u64::MAX).is_err());
            assert_eq!(error.kind, KernelErrorKind::SizeOverflow);
        }
        Ok(layout) => {
            let code = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
            let program = VerifiedKernelProgram::verify_function(
                &code,
                layout,
                KernelVerificationBudget {
                    max_local_values: u64::MAX,
                    ..KernelVerificationBudget::default()
                },
            )
            .expect("Small code proof with wide layout");
            let error = program
                .execute_legacy_script_arguments(
                    &[],
                    KernelExecutionBudget {
                        max_local_values: u64::MAX,
                        max_local_bytes: u64::MAX,
                        ..KernelExecutionBudget::default()
                    },
                )
                .expect_err("Byte overflow before defaults");
            assert_eq!(error.kind, KernelErrorKind::SizeOverflow);
            assert_eq!((error.work_steps, error.admitted_local_values), (0, 0));
        }
    }
}
