// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers assemble valid finite primitive function definitions."
)]

use cod4x_script::execution::{
    KernelBranchCondition, KernelFunctionDefinition, KernelFunctionLayout, KernelInstruction,
    KernelModuleVerificationBudget, KernelValue, VerifiedKernelModule,
};
use cod4x_script::legacy::{LegacyArithmetic, LegacyComparison, LegacyNumber};

pub fn integer(value: i32) -> KernelInstruction {
    KernelInstruction::Constant(KernelValue::Number(LegacyNumber::Integer(value)))
}

pub fn definition(
    code: &[KernelInstruction],
    parameters: u64,
    locals: u64,
) -> KernelFunctionDefinition<'_> {
    KernelFunctionDefinition {
        instructions: code,
        string_literals: &[],
        field_names: &[],
        layout: KernelFunctionLayout::new(parameters, locals).expect("Valid layout"),
    }
}

pub fn verify(definitions: &[KernelFunctionDefinition<'_>]) -> VerifiedKernelModule {
    VerifiedKernelModule::verify(definitions, KernelModuleVerificationBudget::default())
        .expect("Valid module")
}

pub fn countdown(target: usize, base: i32) -> Vec<KernelInstruction> {
    vec![
        KernelInstruction::LoadLocal(0),
        integer(0),
        KernelInstruction::Compare(LegacyComparison::LessOrEqual),
        KernelInstruction::Branch {
            condition: KernelBranchCondition::IfTrue,
            target: 11,
        },
        KernelInstruction::LoadLocal(0),
        integer(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Subtract),
        KernelInstruction::Call {
            function: target,
            arguments: 1,
        },
        integer(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
        integer(base),
        KernelInstruction::Return,
    ]
}
