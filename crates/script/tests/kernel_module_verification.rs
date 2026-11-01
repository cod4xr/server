// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests assemble complete valid and deliberately malformed internal inventories."
)]

use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelInstruction, KernelModuleVerificationBudget,
    KernelResource, KernelValue, KernelVerificationBudget, VerifiedKernelModule,
    VerifiedKernelProgram,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn all_call_targets_need_complete_inventory_and_reachable_calls_need_stack_operands() {
    let leaf = [integer(1), KernelInstruction::Return];
    let code = [
        integer(0),
        KernelInstruction::Return,
        KernelInstruction::Call {
            function: usize::MAX,
            arguments: usize::MAX,
        },
    ];
    let error = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
        .expect_err("Standalone proof cannot resolve unreachable call");
    assert_eq!(error.kind, KernelErrorKind::UnlinkedCall);
    assert_eq!(error.instruction, Some(2));
    let error = VerifiedKernelModule::verify(
        &[definition(&code, 0, 0), definition(&leaf, 0, 0)],
        KernelModuleVerificationBudget::default(),
    )
    .expect_err("Unreachable target still invalid");
    assert_eq!(error.function, Some(0));
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::InvalidFunction {
            function: usize::MAX as u64
        }
    );
    let code = [
        integer(0),
        KernelInstruction::Return,
        KernelInstruction::Call {
            function: 1,
            arguments: usize::MAX,
        },
    ];
    let module = VerifiedKernelModule::verify(
        &[definition(&code, 0, 0), definition(&leaf, 0, 0)],
        KernelModuleVerificationBudget::default(),
    )
    .expect("Unreachable excess count is not a source-signature admission error");
    assert_eq!(
        module
            .execute(0, &[], KernelCallBudget::default())
            .expect("Unreachable call does not run")
            .value,
        KernelValue::Number(LegacyNumber::Integer(0))
    );
    let code = [
        KernelInstruction::Call {
            function: 1,
            arguments: 2,
        },
        KernelInstruction::Return,
    ];
    let error = VerifiedKernelModule::verify(
        &[definition(&code, 0, 0), definition(&leaf, 2, 2)],
        KernelModuleVerificationBudget::default(),
    )
    .expect_err("Valid target/arity cannot hide stack underflow");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::StackUnderflow {
            required: 2,
            available: 0
        }
    );
    assert_eq!(
        VerifiedKernelModule::verify(&[], KernelModuleVerificationBudget::default())
            .expect("Empty inventory retains no fabricated functions")
            .usage()
            .functions,
        0
    );
    let error = VerifiedKernelModule::verify(
        &[definition(&leaf, 0, 0), definition(&[], 0, 0)],
        KernelModuleVerificationBudget::default(),
    )
    .expect_err("A later malformed function prevents complete publication");
    assert_eq!(error.function, Some(1));
    assert_eq!(error.cause.kind, KernelErrorKind::EmptyProgram);
    assert_eq!(
        kernel_call_support::verify(&[definition(&leaf, 0, 0)])
            .usage()
            .functions,
        1
    );
}

#[test]
fn module_function_instruction_work_and_overlapping_storage_limits_are_global() -> Result<(), String>
{
    let code = [integer(1), KernelInstruction::Return];
    let definitions = [definition(&code, 0, 0); 2];
    let proof = kernel_call_support::verify(&definitions);
    let usage = proof.usage();
    let exact = KernelModuleVerificationBudget {
        max_functions: 2,
        max_instructions: 4,
        max_steps: usage.verification_steps,
        max_metadata_bytes: usage.verification_metadata_bytes,
        max_function_stack_values: 1,
        max_function_local_values: 0,
    };
    assert_eq!(
        VerifiedKernelModule::verify(&definitions, exact)
            .expect("Exact module proof admission")
            .usage(),
        usage
    );
    for (resource, budget) in [
        (
            KernelResource::Functions,
            KernelModuleVerificationBudget {
                max_functions: 1,
                ..exact
            },
        ),
        (
            KernelResource::Instructions,
            KernelModuleVerificationBudget {
                max_instructions: 3,
                ..exact
            },
        ),
        (
            KernelResource::VerificationSteps,
            KernelModuleVerificationBudget {
                max_steps: exact.max_steps - 1,
                ..exact
            },
        ),
        (
            KernelResource::VerificationMetadataBytes,
            KernelModuleVerificationBudget {
                max_metadata_bytes: exact.max_metadata_bytes - 1,
                ..exact
            },
        ),
        (
            KernelResource::StackValues,
            KernelModuleVerificationBudget {
                max_function_stack_values: 0,
                ..exact
            },
        ),
    ] {
        let error = VerifiedKernelModule::verify(&definitions, budget)
            .expect_err("One less module resource");
        let KernelErrorKind::BudgetExceeded {
            resource: actual,
            requested,
            maximum,
        } = error.cause.kind
        else {
            return Err("Expected fixed admission failure".into());
        };
        assert_eq!(actual, resource);
        assert_eq!(requested, maximum + 1);
        assert!(error.cause.work_steps <= budget.max_steps);
        if resource == KernelResource::Instructions {
            assert_eq!(error.cause.work_steps, 2);
        }
    }
    let definitions = [definition(&code, 0, 70_000)];
    assert_eq!(
        kernel_call_support::verify(&definitions)
            .execute(0, &[], KernelCallBudget::default())
            .expect("Wide local admission")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
    let error = VerifiedKernelModule::verify(
        &definitions,
        KernelModuleVerificationBudget {
            max_function_local_values: 69_999,
            ..KernelModuleVerificationBudget::default()
        },
    )
    .expect_err("Individual local budget remains explicit");
    assert!(matches!(
        error.cause.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::LocalValues,
            ..
        }
    ));
    Ok(())
}

#[test]
fn wide_function_targets_keep_the_complete_inventory_past_sixteen_bit_indices() {
    let empty = [
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let marker = [integer(42), KernelInstruction::Return];
    let root = [
        KernelInstruction::Call {
            function: 65_999,
            arguments: 0,
        },
        KernelInstruction::Return,
    ];
    let mut definitions = vec![definition(&empty, 0, 0); 66_000];
    definitions[0] = definition(&root, 0, 0);
    definitions[65_999] = definition(&marker, 0, 0);
    let module = kernel_call_support::verify(&definitions);
    assert_eq!(module.usage().functions, 66_000);
    assert_eq!(module.usage().instructions, 132_000);
    for entry in [0, 65_999] {
        assert_eq!(
            module
                .execute(entry, &[], KernelCallBudget::default())
                .expect("Wide target selects final marker")
                .value,
            KernelValue::Number(LegacyNumber::Integer(42))
        );
    }
}
