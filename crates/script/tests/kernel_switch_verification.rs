// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Immutable labels, including unused rows, must share all exact verification scopes."
)]
use crate::kernel_call_support;
use cod4x_script::execution::{
    KernelInstruction, KernelModuleVerificationBudget, KernelSwitchComparison, KernelSwitchKey,
    KernelVerificationBudget, VerifiedKernelModule, VerifiedKernelProgram,
};

#[test]
fn unused_label_rows_payload_and_operands_share_exact_global_and_function_verification_admission() {
    let code = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::RequireSwitchKey,
        KernelInstruction::SwitchCompare {
            key: KernelSwitchKey::StringLabel(0),
            comparison: KernelSwitchComparison::Equal,
        },
        KernelInstruction::Return,
    ];
    let labels = [b"needle".to_vec(), b"unused".to_vec(), Vec::new()];
    let original = labels.clone();
    let mut definition = kernel_call_support::definition(&code, 1, 1);
    definition.switch_labels = &labels;
    let usage =
        VerifiedKernelModule::verify(&[definition], KernelModuleVerificationBudget::default())
            .expect("Baseline")
            .usage();
    assert_eq!((usage.switch_labels, usage.string_literals), (3, 0));
    assert_eq!(
        usage.retained_switch_bytes,
        (3 * size_of::<Vec<u8>>() + 12) as u64
    );
    let exact = KernelModuleVerificationBudget {
        max_functions: 1,
        max_instructions: 4,
        max_steps: usage.verification_steps,
        max_metadata_bytes: usage.verification_metadata_bytes,
        max_function_stack_values: 1,
        max_function_local_values: 1,
    };
    assert_eq!(
        VerifiedKernelModule::verify(&[definition], exact)
            .expect("Exact aggregate scopes")
            .usage(),
        usage
    );
    for maximum in 0..usage.verification_steps {
        let error = VerifiedKernelModule::verify(
            &[definition],
            KernelModuleVerificationBudget {
                max_steps: maximum,
                ..exact
            },
        )
        .expect_err("Every shorter row scan/copy/operand/flow prefix");
        assert!(error.cause.work_steps <= maximum);
        assert!(!format!("{error:?}").contains("needle"));
    }
    for resource in 0..5 {
        let mut short = exact;
        match resource {
            0 => short.max_functions -= 1,
            1 => short.max_instructions -= 1,
            2 => short.max_metadata_bytes -= 1,
            3 => short.max_function_stack_values -= 1,
            _ => short.max_function_local_values -= 1,
        }
        VerifiedKernelModule::verify(&[definition], short).expect_err("One-less aggregate scope");
    }
    let local =
        VerifiedKernelProgram::verify_definition(definition, KernelVerificationBudget::default())
            .expect("Standalone baseline")
            .usage();
    let exact = KernelVerificationBudget {
        max_instructions: 4,
        max_steps: local.verification_steps,
        max_stack_values: 1,
        max_local_values: 1,
        max_metadata_bytes: local.verification_metadata_bytes,
    };
    assert_eq!(
        VerifiedKernelProgram::verify_definition(definition, exact)
            .expect("Exact function scopes")
            .usage(),
        local
    );
    for maximum in 0..local.verification_steps {
        let error = VerifiedKernelProgram::verify_definition(
            definition,
            KernelVerificationBudget {
                max_steps: maximum,
                ..exact
            },
        )
        .expect_err("Every shorter standalone proof prefix");
        assert!(error.work_steps <= maximum);
    }
    for resource in 0..4 {
        let mut short = exact;
        match resource {
            0 => short.max_instructions -= 1,
            1 => short.max_metadata_bytes -= 1,
            2 => short.max_stack_values -= 1,
            _ => short.max_local_values -= 1,
        }
        VerifiedKernelProgram::verify_definition(definition, short)
            .expect_err("One-less function scope");
    }
    assert_eq!(labels, original);
}
