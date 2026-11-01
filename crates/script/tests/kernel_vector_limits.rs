// SPDX-License-Identifier: AGPL-3.0-or-later
//! Vector payload sizes are included in proof/runtime storage before allocation.
#![allow(
    clippy::expect_used,
    reason = "Tests use finite programs and explicit complete budget boundaries."
)]

use crate::kernel_support;
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelInstruction,
    KernelResource, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn vector_constructor_underflow_exact_stack_bytes_and_fuel_are_checked() {
    for count in 0..3 {
        let mut code = vec![kernel_support::integer(1); count];
        code.extend([KernelInstruction::MakeVector, KernelInstruction::Return]);
        let error = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect_err("Three components required");
        assert_eq!(
            error.kind,
            KernelErrorKind::StackUnderflow {
                required: 3,
                available: count as u64
            }
        );
    }
    let code = [
        kernel_support::integer(1),
        kernel_support::integer(2),
        kernel_support::integer(3),
        KernelInstruction::MakeVector,
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
        .expect("Construction proof");
    assert_eq!(program.usage().max_stack_values, 3);
    let exact = KernelExecutionBudget {
        max_steps: 5,
        max_stack_values: 3,
        max_stack_bytes: 3 * size_of::<KernelValue>() as u64,
        max_local_values: 0,
        max_local_bytes: 0,
    };
    let outcome = program.execute(exact).expect("Exact vector storage/fuel");
    assert_eq!(
        outcome.value,
        KernelValue::Vector(LegacyVector::from_numbers([
            LegacyNumber::Integer(3),
            LegacyNumber::Integer(2),
            LegacyNumber::Integer(1)
        ]))
    );
    for (resource, budget, work) in [
        (
            KernelResource::ExecutionStackBytes,
            KernelExecutionBudget {
                max_stack_bytes: exact.max_stack_bytes - 1,
                ..exact
            },
            0,
        ),
        (
            KernelResource::StackValues,
            KernelExecutionBudget {
                max_stack_values: 2,
                ..exact
            },
            0,
        ),
        (
            KernelResource::ExecutionSteps,
            KernelExecutionBudget {
                max_steps: 4,
                ..exact
            },
            4,
        ),
    ] {
        let error = program
            .execute(budget)
            .expect_err("One less vector invocation resource");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded { resource: actual, .. } if actual == resource)
        );
        assert_eq!(error.work_steps, work);
    }
}

#[test]
fn vector_local_byte_admission_and_wide_argument_layouts_use_actual_inline_sizes() {
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::StoreLocal(1),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 2).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Proof");
    let arg = KernelValue::Vector(LegacyVector::from_components([1.0, 2.0, 3.0]));
    let exact = KernelExecutionBudget {
        max_steps: 4,
        max_stack_values: 1,
        max_stack_bytes: size_of::<KernelValue>() as u64,
        max_local_values: 2,
        max_local_bytes: 2 * size_of::<Option<KernelValue>>() as u64,
    };
    let outcome = program
        .execute_with_arguments(&[arg], exact)
        .expect("Exact local bytes");
    assert_eq!(outcome.value, arg);
    assert_eq!(outcome.local_bytes, exact.max_local_bytes);
    let error = program
        .execute_with_arguments(
            &[arg],
            KernelExecutionBudget {
                max_local_bytes: exact.max_local_bytes - 1,
                ..exact
            },
        )
        .expect_err("One less local byte");
    assert!(matches!(
        error.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionLocalBytes,
            ..
        }
    ));
    assert_eq!((error.work_steps, error.admitted_local_values), (0, 0));
    let wide = VerifiedKernelProgram::verify_function(
        &[
            KernelInstruction::LoadLocal(69_999),
            KernelInstruction::Return,
        ],
        KernelFunctionLayout::new(70_000, 70_000).expect("Wide layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Wide vector proof");
    let arguments = vec![arg; 70_000];
    let outcome = wide
        .execute_legacy_script_arguments(&arguments, KernelExecutionBudget::default())
        .expect("Wide vector invocation");
    assert_eq!(outcome.value, arg);
    assert_eq!(
        outcome.local_bytes,
        70_000 * size_of::<Option<KernelValue>>() as u64
    );
    assert_eq!(arguments[69_999], arg);
}
