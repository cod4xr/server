// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::kernel_executor;
use crate::execution::kernel_locals::KernelArgumentBinding;
use crate::execution::kernel_verifier;
use crate::execution::{
    KernelError, KernelExecutionBudget, KernelFunctionLayout, KernelInstruction, KernelValue,
    KernelVerificationBudget,
};

/// Successful verification accounting, excluding allocator rounding and process RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelProgramUsage {
    /// Declared parameter-prefix count; invocation entry points select binding policy.
    pub parameters: u64,
    /// Complete layout slots, including parameters and uninitialized slots.
    pub local_values: u64,
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
    /// Admitted complete local slots for this invocation, not touched/initialized count.
    pub local_values: u64,
    /// Logical local initialized-state/value bytes, excluding allocator rounding/RSS.
    pub local_bytes: u64,
}

/// Independently owned immutable control-flow/stack proof for the scalar profile.
///
/// Construction stays inside this private execution boundary. No public mutation,
/// source/host ownership, runtime IDs, native bytecode, or serialized ABI is supplied.
pub struct VerifiedKernelProgram {
    pub(super) layout: KernelFunctionLayout,
    pub(super) instructions: Vec<KernelInstruction>,
    pub(super) usage: KernelProgramUsage,
}

impl VerifiedKernelProgram {
    /// Admits, copies, and iteratively verifies all operands/reachable stack flow.
    /// This convenience uses a zero-parameter, zero-local layout.
    ///
    /// # Errors
    /// Rejects malformed flow, invalid stack/return shapes, resource/size failures,
    /// and fallible allocation failures. No partial proof escapes or input changes.
    pub fn verify(
        instructions: &[KernelInstruction],
        budget: KernelVerificationBudget,
    ) -> Result<Self, KernelError> {
        Self::verify_function(instructions, KernelFunctionLayout::default(), budget)
    }

    /// Verifies every branch/local operand and reachable stack flow under a layout.
    ///
    /// Local initialization is checked at runtime; this proof does not establish
    /// source definite assignment, name binding, or native argument semantics.
    ///
    /// # Errors
    /// Rejects invalid operands/flow/stack shapes, resource/size failure, and fallible
    /// allocation failure without changing input or publishing a partial proof.
    pub fn verify_function(
        instructions: &[KernelInstruction],
        layout: KernelFunctionLayout,
        budget: KernelVerificationBudget,
    ) -> Result<Self, KernelError> {
        kernel_verifier::verify(instructions, layout, budget)
    }

    /// Executes only this immutable proof under separately admitted stack/fuel budgets.
    /// Supplies no arguments while retaining this program's local layout.
    ///
    /// # Errors
    /// Returns located resource/numeric/type/state failures without a partial result
    /// or external mutation, including an arity error for a parameterized layout.
    /// This is not a source compiler, full GSC VM, or scheduler.
    pub fn execute(&self, budget: KernelExecutionBudget) -> Result<KernelOutcome, KernelError> {
        self.execute_with_arguments(&[], budget)
    }

    /// Copies exact internal arguments into fresh private parameter/local storage.
    ///
    /// Nonparameter locals start uninitialized; stored Undefined is initialized.
    /// Count/byte admission bounds initialization independently of instruction fuel.
    /// Arguments/other invocations remain unchanged on success or failure.
    ///
    /// # Errors
    /// Rejects wrong exact arity, exhausted local/stack/fuel admission, allocation
    /// failure, uninitialized loads, and numeric/type/state errors. No partial result
    /// escapes. Legacy script-argument adaptation uses its explicit entry point.
    pub fn execute_with_arguments(
        &self,
        arguments: &[KernelValue],
        budget: KernelExecutionBudget,
    ) -> Result<KernelOutcome, KernelError> {
        kernel_executor::execute(self, arguments, budget, KernelArgumentBinding::Exact)
    }

    /// Invokes the scalar legacy script profile with an already parameter-ordered prefix.
    ///
    /// Missing parameter slots receive initialized Undefined; supplied values are
    /// copied, and nonparameter locals remain uninitialized. Excess values fail.
    /// Direct local initialization uses existing count/byte/stack/fuel admission
    /// without a temporary padded argument array. Caller state remains unchanged.
    /// This is not builtin validation, native stack/push order, frames or unwind parity.
    ///
    /// # Errors
    /// Rejects excess arguments, exhausted admission/fuel, allocation failure and
    /// located runtime type/numeric/uninitialized/state errors. No partial result
    /// or external mutation escapes; native markers/pointers are never accepted.
    pub fn execute_legacy_script_arguments(
        &self,
        arguments: &[KernelValue],
        budget: KernelExecutionBudget,
    ) -> Result<KernelOutcome, KernelError> {
        kernel_executor::execute(self, arguments, budget, KernelArgumentBinding::LegacyScript)
    }

    /// Returns immutable parameter/local counts without names or runtime state.
    #[must_use]
    pub fn layout(&self) -> KernelFunctionLayout {
        self.layout
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
