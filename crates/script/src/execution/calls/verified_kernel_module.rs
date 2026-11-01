// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::calls::kernel_module_executor;
use crate::execution::calls::kernel_module_verifier;
use crate::execution::calls::{
    KernelCallBudget, KernelCallOutcome, KernelModuleError, KernelModuleUsage,
    KernelModuleVerificationBudget,
};
use crate::execution::{
    KernelFunctionLayout, KernelInstruction, KernelValue, VerifiedKernelProgram,
};

/// Borrowed internal function input, without source names, imports or host authority.
#[derive(Clone, Copy, Debug)]
pub struct KernelFunctionDefinition<'a> {
    /// Instructions copied only after complete inventory admission.
    pub instructions: &'a [KernelInstruction],
    /// Parameter-prefix and complete local slot counts.
    pub layout: KernelFunctionLayout,
}

/// Owned immutable direct-call inventory. This is not source linking/module activation.
///
/// ```
/// use cod4x_script::execution::{
///     KernelCallBudget, KernelFunctionDefinition, KernelFunctionLayout,
///     KernelInstruction, KernelModuleError, KernelModuleVerificationBudget,
///     KernelValue, VerifiedKernelModule,
/// };
/// use cod4x_script::legacy::LegacyNumber;
///
/// let caller = [KernelInstruction::Call { function: 1, arguments: 0 }, KernelInstruction::Return];
/// let result = KernelValue::Number(LegacyNumber::Integer(42));
/// let callee = [KernelInstruction::Constant(result), KernelInstruction::Return];
/// let definitions = [
///     KernelFunctionDefinition { instructions: &caller, layout: KernelFunctionLayout::default() },
///     KernelFunctionDefinition { instructions: &callee, layout: KernelFunctionLayout::default() },
/// ];
/// let module = VerifiedKernelModule::verify(&definitions, KernelModuleVerificationBudget::default())?;
/// assert_eq!(module.execute(0, &[], KernelCallBudget::default())?.value, result);
/// # Ok::<(), KernelModuleError>(())
/// ```
pub struct VerifiedKernelModule {
    pub(super) functions: Vec<VerifiedKernelProgram>,
    pub(super) usage: KernelModuleUsage,
}

impl VerifiedKernelModule {
    /// Verifies every function, call operand/arity and reachable stack/flow shape.
    /// Supports forward calls and recursion without host recursion.
    ///
    /// # Errors
    /// Rejects empty inventories, malformed/unresolved operands, excess arguments,
    /// invalid flow, global/per-function budget exhaustion and allocation failure.
    /// No partial module escapes; borrowed input remains unchanged.
    pub fn verify(
        definitions: &[KernelFunctionDefinition<'_>],
        budget: KernelModuleVerificationBudget,
    ) -> Result<Self, KernelModuleError> {
        kernel_module_verifier::verify(definitions, budget)
    }

    /// Returns complete successful inventory/proof accounting without code constants.
    #[must_use]
    pub fn usage(&self) -> KernelModuleUsage {
        self.usage
    }

    /// Invokes an entry with a source-parameter-ordered primitive argument prefix.
    /// Missing parameters receive Undefined; excess arguments fail. All frames own
    /// copied private locals/stacks, sharing global fuel/setup/live-payload budgets.
    /// This is not source linking, native call ABI, scheduling or host authority.
    ///
    /// # Errors
    /// Returns located entry/arity/admission/allocation/type/numeric/state failures
    /// with global accounting. No partial result or external mutation escapes.
    pub fn execute(
        &self,
        entry: u64,
        arguments: &[KernelValue],
        budget: KernelCallBudget,
    ) -> Result<KernelCallOutcome, KernelModuleError> {
        kernel_module_executor::execute(self, entry, arguments, budget)
    }
}

impl fmt::Debug for VerifiedKernelModule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedKernelModule")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
