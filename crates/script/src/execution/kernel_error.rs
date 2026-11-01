// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use crate::legacy::{LegacyNumberError, LegacyVectorError};

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
    /// Complete local count, including parameters/uninitialized slots.
    LocalValues,
    /// Logical runtime initialized-state/value slots.
    ExecutionLocalBytes,
}

/// Fixed kernel failure category; never contains constants, source, or private names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelErrorKind {
    /// There is no entry instruction.
    EmptyProgram,
    /// Parameter-prefix count exceeds total local slots.
    InvalidFunctionLayout {
        /// Requested parameter count.
        parameters: u64,
        /// Requested total local slots.
        local_values: u64,
    },
    /// A local operand is outside the immutable function layout.
    InvalidLocal {
        /// Rejected local index.
        slot: u64,
        /// Complete admitted local count.
        local_values: u64,
    },
    /// Argument count does not satisfy the selected primitive invocation profile.
    ArgumentCount {
        /// Layout parameters: exact internal count or legacy script maximum.
        expected: u64,
        /// Supplied primitive arguments.
        actual: u64,
    },
    /// A local has never been initialized on the selected path.
    UninitializedLocal {
        /// Local index, without its source name or value.
        slot: u64,
    },
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
    /// This operation requires a number; Undefined/vector data is unsupported here.
    ExpectedNumber,
    /// This primitive pair/comparison has incompatible legacy value types.
    UnmatchingTypes,
    /// The current primitive profile requires a vector for an indexed object.
    ExpectedIndexable,
    /// Defined legacy numeric operation failure, without operand values.
    Numeric(LegacyNumberError),
    /// Defined legacy vector-domain operation failure, without coordinate data.
    Vector(LegacyVectorError),
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
    /// Complete local count admitted before this failure, not allocated bytes/RSS.
    pub admitted_local_values: u64,
}

impl KernelError {
    pub(super) fn with_locals(mut self, count: u64) -> Self {
        self.admitted_local_values = count;
        self
    }
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
            "primitive kernel error at {:?} after {} work steps: {:?}",
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
        admitted_local_values: 0,
    }
}

pub(super) fn general(kind: KernelErrorKind) -> KernelError {
    KernelError {
        instruction: None,
        kind,
        work_steps: 0,
        peak_stack_values: 0,
        admitted_local_values: 0,
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
            admitted_local_values: 0,
            kind: KernelErrorKind::BudgetExceeded {
                resource,
                requested,
                maximum,
            },
        });
    }
    Ok(())
}
