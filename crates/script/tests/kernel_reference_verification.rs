// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests intentionally assemble malformed reference instructions."
)]

use crate::kernel_call_support::{definition, integer};
use cod4x_script::execution::{
    KernelErrorKind, KernelInstruction, KernelModuleVerificationBudget, KernelVerificationBudget,
    VerifiedKernelModule, VerifiedKernelProgram,
};

#[test]
fn reference_operands_and_indirect_sizes_are_verified_even_when_unreachable() {
    for (instruction, standalone, linked) in [
        (
            KernelInstruction::FunctionReference(1),
            KernelErrorKind::UnlinkedFunctionReference,
            KernelErrorKind::InvalidFunction { function: 1 },
        ),
        (
            KernelInstruction::CallIndirect {
                arguments: usize::MAX,
            },
            KernelErrorKind::UnlinkedCall,
            KernelErrorKind::SizeOverflow,
        ),
    ] {
        let code = [integer(0), KernelInstruction::Return, instruction];
        let error = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect_err("Standalone cannot resolve module operations");
        assert_eq!(error.kind, standalone);
        assert_eq!(error.instruction, Some(2));
        let error = VerifiedKernelModule::verify(
            &[definition(&code, 0, 0)],
            KernelModuleVerificationBudget::default(),
        )
        .expect_err("Unreachable operands still checked");
        assert_eq!(error.cause.kind, linked);
        assert_eq!(error.cause.instruction, Some(2));
    }
}

#[test]
fn indirect_stack_effect_includes_callee_and_all_supplied_arguments() {
    for (arguments, required, available) in [(0, 1, 0), (1, 2, 1)] {
        let mut code = Vec::new();
        if arguments == 1 {
            code.push(KernelInstruction::FunctionReference(0));
        }
        code.extend([
            KernelInstruction::CallIndirect { arguments },
            KernelInstruction::Return,
        ]);
        let error = VerifiedKernelModule::verify(
            &[definition(&code, 0, 0)],
            KernelModuleVerificationBudget::default(),
        )
        .expect_err("Missing operands");
        assert_eq!(
            error.cause.kind,
            KernelErrorKind::StackUnderflow {
                required,
                available
            }
        );
    }
}
