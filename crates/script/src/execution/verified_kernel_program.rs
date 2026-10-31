// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::kernel_executor;
use crate::execution::kernel_verifier;
use crate::execution::{
    KernelError, KernelExecutionBudget, KernelInstruction, KernelValue, KernelVerificationBudget,
};

/// Successful verification accounting, excluding allocator rounding and process RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelProgramUsage {
    /// Complete retained instruction count, including unreachable code.
    pub instructions: u64,
    /// Actual operand scans, reachable-node visits, and propagated edges.
    pub verification_steps: u64,
    /// Maximum reachable stack depth admitted by the verifier.
    pub max_stack_values: u64,
    /// Logical retained instruction bytes, without temporary verifier slots.
    pub retained_code_bytes: u64,
    /// Logical admitted code plus full temporary state/worklist slots.
    pub verification_metadata_bytes: u64,
}

/// Completed pure execution result; Debug omits the returned scalar contents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KernelOutcome {
    /// Returned undefined/numeric data, not a full GSC runtime value.
    pub value: KernelValue,
    /// Executed instructions, including return.
    pub steps: u64,
    /// Actual peak live stack values on the selected execution path.
    pub peak_stack_values: u64,
}

/// Independently owned immutable control-flow/stack proof for the scalar profile.
///
/// Construction stays inside this private execution boundary. No public mutation,
/// source/host ownership, runtime IDs, native bytecode, or serialized ABI is supplied.
pub struct VerifiedKernelProgram {
    pub(super) instructions: Vec<KernelInstruction>,
    pub(super) usage: KernelProgramUsage,
}

impl VerifiedKernelProgram {
    /// Admits, copies, and iteratively verifies all operands/reachable stack flow.
    ///
    /// # Errors
    /// Rejects malformed flow, invalid stack/return shapes, resource/size failures,
    /// and fallible allocation failures. No partial proof escapes or input changes.
    pub fn verify(
        instructions: &[KernelInstruction],
        budget: KernelVerificationBudget,
    ) -> Result<Self, KernelError> {
        kernel_verifier::verify(instructions, budget)
    }

    /// Executes only this immutable proof under separately admitted stack/fuel budgets.
    ///
    /// # Errors
    /// Returns located resource/numeric/type/state failures without a partial result
    /// or external mutation. This is not a source compiler, full GSC VM, or scheduler.
    pub fn execute(&self, budget: KernelExecutionBudget) -> Result<KernelOutcome, KernelError> {
        kernel_executor::execute(self, budget)
    }

    /// Borrows immutable instructions without permitting changes to the verified proof.
    #[must_use]
    pub fn instructions(&self) -> &[KernelInstruction] {
        &self.instructions
    }

    /// Returns successful logical program/verification accounting without constants.
    #[must_use]
    pub fn usage(&self) -> KernelProgramUsage {
        self.usage
    }
}

impl fmt::Debug for VerifiedKernelProgram {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedKernelProgram")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
