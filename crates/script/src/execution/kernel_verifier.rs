// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_flow_state::KernelFlowState;
use crate::execution::kernel_verification_ledger::KernelVerificationLedger;
use crate::execution::{
    KernelError, KernelErrorKind, KernelInstruction, KernelVerificationBudget,
    VerifiedKernelProgram,
};

pub(super) fn verify(
    input: &[KernelInstruction],
    budget: KernelVerificationBudget,
) -> Result<VerifiedKernelProgram, KernelError> {
    if input.is_empty() {
        return Err(kernel_error::general(KernelErrorKind::EmptyProgram));
    }
    let mut ledger = KernelVerificationLedger::new(input.len(), budget)?;
    verify_flow(input, &mut ledger).map_err(|error| {
        error.with_usage(
            ledger.usage.verification_steps,
            ledger.usage.max_stack_values,
        )
    })
}

fn verify_flow(
    input: &[KernelInstruction],
    ledger: &mut KernelVerificationLedger,
) -> Result<VerifiedKernelProgram, KernelError> {
    for (pc, instruction) in input.iter().copied().enumerate() {
        ledger.visit(pc)?;
        if let KernelInstruction::Jump(target) | KernelInstruction::Branch { target, .. } =
            instruction
            && target >= input.len()
        {
            return Err(kernel_error::at(
                pc,
                KernelErrorKind::InvalidTarget {
                    target: target as u64,
                },
            ));
        }
    }
    let mut instructions = Vec::new();
    instructions
        .try_reserve_exact(input.len())
        .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))?;
    instructions.extend_from_slice(input);
    let mut flow = KernelFlowState::new(input.len())?;
    while let Some((pc, height)) = flow.next_pending()? {
        ledger.visit(pc)?;
        let instruction = instructions
            .get(pc)
            .copied()
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?;
        let next_height = stack_height(instruction, height, pc)?;
        ledger.admit_stack(next_height, pc)?;
        match instruction {
            KernelInstruction::Return => {}
            KernelInstruction::Jump(target) => {
                edge(&mut flow, ledger, pc, target, next_height)?;
            }
            KernelInstruction::Branch { target, .. } => {
                edge(&mut flow, ledger, pc, target, next_height)?;
                fallthrough(&mut flow, ledger, pc, next_height, input.len())?;
            }
            _ => fallthrough(&mut flow, ledger, pc, next_height, input.len())?,
        }
    }
    Ok(VerifiedKernelProgram {
        instructions,
        usage: ledger.usage,
    })
}

fn stack_height(
    instruction: KernelInstruction,
    height: usize,
    pc: usize,
) -> Result<usize, KernelError> {
    if matches!(instruction, KernelInstruction::Return) && height != 1 {
        return Err(kernel_error::at(
            pc,
            KernelErrorKind::InvalidReturnStack {
                values: height as u64,
            },
        ));
    }
    let (pops, pushes) = instruction.stack_effect();
    let after = height.checked_sub(pops).ok_or_else(|| {
        kernel_error::at(
            pc,
            KernelErrorKind::StackUnderflow {
                required: pops as u64,
                available: height as u64,
            },
        )
    })?;
    after
        .checked_add(pushes)
        .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::SizeOverflow))
}

fn edge(
    flow: &mut KernelFlowState,
    ledger: &mut KernelVerificationLedger,
    pc: usize,
    target: usize,
    height: usize,
) -> Result<(), KernelError> {
    ledger.visit(pc)?;
    flow.bind(pc, target, height)
}

fn fallthrough(
    flow: &mut KernelFlowState,
    ledger: &mut KernelVerificationLedger,
    pc: usize,
    height: usize,
    count: usize,
) -> Result<(), KernelError> {
    let next = pc
        .checked_add(1)
        .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::SizeOverflow))?;
    if next >= count {
        return Err(kernel_error::at(pc, KernelErrorKind::FallthroughPastEnd));
    }
    edge(flow, ledger, pc, next, height)
}
