// SPDX-License-Identifier: AGPL-3.0-or-later
//! Argument-prefix/local execution preserves initialization and per-invocation ownership.
#![allow(
    clippy::expect_used,
    reason = "Tests supply finite valid typed function layouts/programs."
)]

use cod4x_script::execution::{
    KernelBranchCondition, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyArithmetic, LegacyNumber};

use crate::kernel_support;

#[test]
fn exact_arguments_and_local_stores_execute_without_mutating_caller_storage() {
    let layout = KernelFunctionLayout::new(2, 3).expect("Layout");
    let input = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::StoreLocal(2),
        KernelInstruction::LoadLocal(2),
        KernelInstruction::Return,
    ];
    let program =
        VerifiedKernelProgram::verify_function(&input, layout, KernelVerificationBudget::default())
            .expect("Function proof");
    let arguments = [
        KernelValue::Number(LegacyNumber::Integer(7)),
        KernelValue::Number(LegacyNumber::Integer(3)),
    ];
    let outcome = program
        .execute_with_arguments(&arguments, KernelExecutionBudget::default())
        .expect("Bound invocation");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(10))
    );
    assert_eq!(outcome.steps, 6);
    assert_eq!(outcome.local_values, 3);
    assert_eq!(
        outcome.local_bytes,
        3 * size_of::<Option<KernelValue>>() as u64
    );
    assert_eq!(arguments[0], KernelValue::Number(LegacyNumber::Integer(7)));
    assert_eq!(arguments[1], KernelValue::Number(LegacyNumber::Integer(3)));
    assert_eq!(program.layout(), layout);
    assert_eq!(
        (program.usage().parameters, program.usage().local_values),
        (2, 3)
    );
    for arguments in [&[][..], &arguments[..1], &[KernelValue::Undefined; 3][..]] {
        let error = program
            .execute_with_arguments(arguments, KernelExecutionBudget::default())
            .expect_err("Exact internal arity");
        assert_eq!(
            error.kind,
            KernelErrorKind::ArgumentCount {
                expected: 2,
                actual: arguments.len() as u64
            }
        );
        assert_eq!((error.work_steps, error.admitted_local_values), (0, 0));
    }
    assert!(matches!(
        program
            .execute(KernelExecutionBudget::default())
            .expect_err("No implicit argument binding")
            .kind,
        KernelErrorKind::ArgumentCount {
            expected: 2,
            actual: 0
        }
    ));
}

#[test]
fn stored_undefined_and_argument_float_bits_remain_initialized_values() -> Result<(), String> {
    let layout = KernelFunctionLayout::new(1, 2).expect("Layout");
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::StoreLocal(1),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Return,
    ];
    let program =
        VerifiedKernelProgram::verify_function(&code, layout, KernelVerificationBudget::default())
            .expect("Store/load proof");
    for bits in [0x8000_0000, 0x7fc1_2345, 1, 0x7f80_0000] {
        let arguments = [KernelValue::Number(LegacyNumber::Float(f32::from_bits(
            bits,
        )))];
        let value = program
            .execute_with_arguments(&arguments, KernelExecutionBudget::default())
            .expect("Bit-preserving locals")
            .value;
        let KernelValue::Number(LegacyNumber::Float(value)) = value else {
            return Err("Expected float local".into());
        };
        assert_eq!(value.to_bits(), bits);
    }
    assert_eq!(
        program
            .execute_with_arguments(&[KernelValue::Undefined], KernelExecutionBudget::default())
            .expect("Initialized Undefined")
            .value,
        KernelValue::Undefined
    );
    let uninitialized = VerifiedKernelProgram::verify_function(
        &[KernelInstruction::LoadLocal(0), KernelInstruction::Return],
        KernelFunctionLayout::new(0, 1).expect("Local"),
        KernelVerificationBudget::default(),
    )
    .expect("Stack proof does not assert definite assignment");
    let error = uninitialized
        .execute(KernelExecutionBudget::default())
        .expect_err("Never initialized");
    assert_eq!(error.kind, KernelErrorKind::UninitializedLocal { slot: 0 });
    assert_eq!(error.instruction, Some(0));
    assert_eq!((error.work_steps, error.admitted_local_values), (1, 1));
    Ok(())
}

#[test]
fn selected_path_initialization_and_fresh_invocations_do_not_share_locals() {
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Branch {
            condition: KernelBranchCondition::IfFalse,
            target: 5,
        },
        kernel_support::integer(7),
        KernelInstruction::StoreLocal(1),
        KernelInstruction::Jump(5),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 2).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Branch stack proof");
    for condition in [1, 0, 1] {
        let arguments = [KernelValue::Number(LegacyNumber::Integer(condition))];
        let result = program.execute_with_arguments(&arguments, KernelExecutionBudget::default());
        if condition == 0 {
            let error = result.expect_err("Skipped initialization");
            assert_eq!(error.kind, KernelErrorKind::UninitializedLocal { slot: 1 });
            assert_eq!(error.instruction, Some(5));
            assert_eq!((error.work_steps, error.admitted_local_values), (3, 2));
        } else {
            assert_eq!(
                result.expect("Initialized invocation").value,
                KernelValue::Number(LegacyNumber::Integer(7))
            );
        }
    }
}

#[test]
fn seventy_thousand_arguments_and_local_indices_have_no_native_short_or_local_ceiling() {
    let layout = KernelFunctionLayout::new(70_000, 70_000).expect("Wide layout");
    let mut code = [
        KernelInstruction::LoadLocal(69_999),
        KernelInstruction::Return,
    ];
    let program =
        VerifiedKernelProgram::verify_function(&code, layout, KernelVerificationBudget::default())
            .expect("Wide local proof");
    code.fill(KernelInstruction::LoadLocal(usize::MAX));
    let mut arguments = vec![KernelValue::Undefined; 70_000];
    arguments[69_999] = KernelValue::Number(LegacyNumber::Integer(37));
    let outcome = program
        .execute_with_arguments(&arguments, KernelExecutionBudget::default())
        .expect("Wide parameter binding");
    assert_eq!(
        outcome.value,
        KernelValue::Number(LegacyNumber::Integer(37))
    );
    assert_eq!((outcome.steps, outcome.local_values), (2, 70_000));
    assert!(!format!("{program:?} {outcome:?}").contains("37"));
}
