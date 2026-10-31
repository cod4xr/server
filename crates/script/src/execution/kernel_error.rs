// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::legacy::LegacyNumberError;

/// Program/execution resource, independent of native script capacity constants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelResource {
    /// Input code instructions.
    Instructions,
    /// Verification scans/visits/edges.
    VerificationSteps,
    /// Logical code plus temporary state/worklist slots.
    VerificationMetadataBytes,
    /// Maximum reachable value stack depth.
    StackValues,
    /// Executed instructions for this invocation.
    ExecutionSteps,
    /// Logical reserved runtime stack bytes.
    ExecutionStackBytes,
}

/// Fixed kernel failure category; never contains constants, source, or private names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelErrorKind {
    /// There is no entry instruction.
    EmptyProgram,
    /// A branch/jump operand points outside this program.
    InvalidTarget {
        /// Invalid instruction index.
        target: u64,
    },
    /// Reachable ordinary execution would run past the final instruction.
    FallthroughPastEnd,
    /// A reachable instruction cannot consume its required stack operands.
    StackUnderflow {
        /// Required live values.
        required: u64,
        /// Available live values.
        available: u64,
    },
    /// Two control-flow paths disagree on a destination's live stack height.
    StackJoin {
        /// Destination instruction.
        target: u64,
        /// First established height.
        expected: u64,
        /// Conflicting height.
        actual: u64,
    },
    /// A return requires exactly one live result value.
    InvalidReturnStack {
        /// Actual live result stack height.
        values: u64,
    },
    /// A configured count/work/storage budget was exhausted.
    BudgetExceeded {
        /// Resource being admitted.
        resource: KernelResource,
        /// Logical requested count or bytes.
        requested: u64,
        /// Configured admission budget.
        maximum: u64,
    },
    /// Checked count/storage/address arithmetic overflowed.
    SizeOverflow,
    /// Fallible complete code/state/stack allocation failed.
    AllocationFailed,
    /// Undefined is not a numeric operand in this limited instruction profile.
    ExpectedNumber,
    /// Defined legacy numeric operation failure, without operand values.
    Numeric(LegacyNumberError),
    /// Private execution state disagreed with an immutable verified proof.
    InvalidExecutionState,
}

/// Typed verification/execution error at an optional zero-based instruction index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelError {
    /// Instruction location; None for whole-program/storage admission failures.
    pub instruction: Option<u64>,
    /// Fixed category/resource counts without operand contents.
    pub kind: KernelErrorKind,
    /// Charged verification visits/edges or execution instructions, including a
    /// failing operation when admitted; rejected fuel is never charged.
    pub work_steps: u64,
    /// Admitted verifier maximum or actual execution peak before failure.
    pub peak_stack_values: u64,
}

impl KernelError {
    pub(super) fn with_usage(mut self, work_steps: u64, peak_stack_values: u64) -> Self {
        self.work_steps = work_steps;
        self.peak_stack_values = peak_stack_values;
        self
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "scalar kernel error at {:?} after {} work steps: {:?}",
            self.instruction, self.work_steps, self.kind
        )
    }
}

impl Error for KernelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            KernelErrorKind::Numeric(error) => Some(error),
            _ => None,
        }
    }
}

pub(super) fn at(pc: usize, kind: KernelErrorKind) -> KernelError {
    KernelError {
        instruction: Some(pc as u64),
        kind,
        work_steps: 0,
        peak_stack_values: 0,
    }
}

pub(super) fn general(kind: KernelErrorKind) -> KernelError {
    KernelError {
        instruction: None,
        kind,
        work_steps: 0,
        peak_stack_values: 0,
    }
}

pub(super) fn check(
    resource: KernelResource,
    requested: u64,
    maximum: u64,
    instruction: Option<u64>,
) -> Result<(), KernelError> {
    if requested > maximum {
        return Err(KernelError {
            instruction,
            work_steps: 0,
            peak_stack_values: 0,
            kind: KernelErrorKind::BudgetExceeded {
                resource,
                requested,
                maximum,
            },
        });
    }
    Ok(())
}
