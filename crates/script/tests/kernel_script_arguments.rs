// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit legacy script prefix binding initializes missing parameters only.
#![allow(
    clippy::expect_used,
    reason = "Tests supply finite valid scalar layouts and parameter-ordered arguments."
)]

use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn every_supplied_prefix_fills_missing_parameters_without_initializing_other_locals() {
    let layout = KernelFunctionLayout::new(3, 5).expect("Layout");
    let arguments = [7, 11, 13].map(|value| KernelValue::Number(LegacyNumber::Integer(value)));
    for supplied in 0..=3 {
        for slot in 0..5 {
            let code = [
                KernelInstruction::LoadLocal(slot),
                KernelInstruction::Return,
            ];
            let program = VerifiedKernelProgram::verify_function(
                &code,
                layout,
                KernelVerificationBudget::default(),
            )
            .expect("Layout proof");
            let result = program.execute_legacy_script_arguments(
                &arguments[..supplied],
                KernelExecutionBudget::default(),
            );
            if slot < 3 {
                let expected = if slot < supplied {
                    arguments[slot]
                } else {
                    KernelValue::Undefined
                };
                let outcome = result.expect("Initialized parameter");
                assert_eq!(outcome.value, expected);
                assert_eq!((outcome.steps, outcome.local_values), (2, 5));
            } else {
                let error = result.expect_err("Nonparameter remains uninitialized");
                assert_eq!(
                    error.kind,
                    KernelErrorKind::UninitializedLocal { slot: slot as u64 }
                );
                assert_eq!((error.work_steps, error.admitted_local_values), (1, 5));
            }
            if supplied < 3 {
                assert_eq!(
                    program
                        .execute_with_arguments(
                            &arguments[..supplied],
                            KernelExecutionBudget::default()
                        )
                        .expect_err("Exact entry point remains exact")
                        .kind,
                    KernelErrorKind::ArgumentCount {
                        expected: 3,
                        actual: supplied as u64
                    }
                );
            }
        }
    }
    let code = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let program =
        VerifiedKernelProgram::verify_function(&code, layout, KernelVerificationBudget::default())
            .expect("Proof");
    let error = program
        .execute_legacy_script_arguments(
            &[KernelValue::Undefined; 4],
            KernelExecutionBudget::default(),
        )
        .expect_err("Excess values");
    assert_eq!(
        error.kind,
        KernelErrorKind::ArgumentCount {
            expected: 3,
            actual: 4
        }
    );
    assert_eq!((error.work_steps, error.admitted_local_values), (0, 0));
    assert_eq!(
        arguments,
        [7, 11, 13].map(|value| KernelValue::Number(LegacyNumber::Integer(value)))
    );
}

#[test]
fn supplied_float_bits_and_fresh_missing_suffixes_preserve_caller_values() -> Result<(), String> {
    let code = [KernelInstruction::LoadLocal(0), KernelInstruction::Return];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(2, 2).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Proof");
    for bits in [0x8000_0000, 0x7fc1_2345, 1, 0x7f80_0000] {
        let arguments = [KernelValue::Number(LegacyNumber::Float(f32::from_bits(
            bits,
        )))];
        let outcome = program
            .execute_legacy_script_arguments(&arguments, KernelExecutionBudget::default())
            .expect("Copied prefix");
        let KernelValue::Number(LegacyNumber::Float(value)) = outcome.value else {
            return Err("Expected copied float".into());
        };
        assert_eq!(value.to_bits(), bits);
        let KernelValue::Number(LegacyNumber::Float(value)) = arguments[0] else {
            return Err("Expected original float".into());
        };
        assert_eq!(value.to_bits(), bits);
        assert_eq!(
            program
                .execute_legacy_script_arguments(&[], KernelExecutionBudget::default())
                .expect("Fresh defaults")
                .value,
            KernelValue::Undefined
        );
    }
    Ok(())
}

#[test]
fn seventy_thousand_missing_parameter_slots_do_not_need_native_short_indices() {
    let code = [
        KernelInstruction::LoadLocal(69_999),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(70_000, 70_001).expect("Wide layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Wide proof");
    let outcome = program
        .execute_legacy_script_arguments(&[], KernelExecutionBudget::default())
        .expect("Wide missing prefix");
    assert_eq!(outcome.value, KernelValue::Undefined);
    assert_eq!((outcome.steps, outcome.local_values), (2, 70_001));
    assert_eq!(
        outcome.local_bytes,
        70_001 * size_of::<Option<KernelValue>>() as u64
    );
}
