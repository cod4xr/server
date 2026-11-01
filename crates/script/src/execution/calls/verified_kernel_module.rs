// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::calls::kernel_module_executor;
use crate::execution::calls::kernel_module_verifier;
use crate::execution::calls::{
    KernelCallBudget, KernelCallOutcome, KernelModuleError, KernelModuleUsage,
    KernelModuleVerificationBudget,
};
use crate::execution::kernel_module_identity::KernelModuleIdentity;
use crate::execution::{
    KernelFunctionLayout, KernelFunctionReference, KernelHeapContext, KernelInstruction,
    KernelProgramUsage, KernelValue, VerifiedKernelProgram,
};

/// Borrowed internal function input, without source names, imports or host authority.
#[derive(Clone, Copy)]
pub struct KernelFunctionDefinition<'a> {
    /// Instructions copied only after complete inventory admission.
    pub instructions: &'a [KernelInstruction],
    /// Parameter-prefix and complete local slot counts.
    pub layout: KernelFunctionLayout,
    /// Owned by the caller until proof copies all rows and exact payload bytes.
    pub string_literals: &'a [Vec<u8>],
    /// Exact immutable field-name bytes, independent of runtime string caches.
    pub field_names: &'a [Vec<u8>],
}

/// Owned immutable call/reference inventory. This is not source linking/module activation.
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
///     KernelFunctionDefinition { instructions: &caller, layout: KernelFunctionLayout::default(), string_literals: &[], field_names: &[] },
///     KernelFunctionDefinition { instructions: &callee, layout: KernelFunctionLayout::default(), string_literals: &[], field_names: &[] },
/// ];
/// let module = VerifiedKernelModule::verify(&definitions, KernelModuleVerificationBudget::default())?;
/// assert_eq!(module.execute(0, &[], KernelCallBudget::default())?.value, result);
/// # Ok::<(), KernelModuleError>(())
/// ```
pub struct VerifiedKernelModule {
    pub(super) identity: KernelModuleIdentity,
    pub(super) functions: Vec<VerifiedKernelProgram>,
    pub(super) usage: KernelModuleUsage,
}

impl fmt::Debug for KernelFunctionDefinition<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KernelFunctionDefinition")
            .field("instructions", &self.instructions.len())
            .field("layout", &self.layout)
            .field("string_literals", &self.string_literals.len())
            .finish_non_exhaustive()
    }
}

impl VerifiedKernelModule {
    /// Verifies every function, call target and reachable stack/flow shape.
    /// Supports forward calls and recursion without host recursion.
    /// Empty inventories contain no entry and need no fabricated function code.
    ///
    /// # Errors
    /// Rejects malformed/unresolved operands,
    /// invalid flow, global/per-function budget or identity exhaustion, and allocation failure.
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

    /// Returns one function's proof accounting without exposing mutable code/constants.
    #[must_use]
    pub fn function_usage(&self, function: u64) -> Option<KernelProgramUsage> {
        self.functions
            .get(usize::try_from(function).ok()?)
            .map(VerifiedKernelProgram::usage)
    }

    /// Returns an opaque copied reference to one checked function in this module.
    /// The reference grants no host authority and does not keep the module alive.
    #[must_use]
    pub fn function_reference(&self, function: u64) -> Option<KernelFunctionReference> {
        let index = usize::try_from(function).ok()?;
        self.functions.get(index)?;
        Some(KernelFunctionReference::new(self.identity, index))
    }

    /// Invokes an entry with a source-parameter-ordered primitive argument prefix.
    /// Missing parameters receive Undefined; excess arguments fail. All frames own
    /// copied private locals/stacks, sharing global fuel/setup/live-payload budgets.
    /// Call-site excess arguments fail at invocation, after caller arguments run
    /// and before callee allocation/body execution; skipped calls do not fail.
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
        kernel_module_executor::execute(self, entry, arguments, budget, None)
    }

    /// Invokes with caller-owned strings, complete external roots and global heap work.
    /// Legacy argument-prefix binding and all existing call budgets still apply.
    ///
    /// # Errors
    /// Rejects stale/foreign arguments and located runtime/resource failures. Failed
    /// execution can retain allocations or completed reclamation. Caller IDs must be
    /// explicitly rooted to survive overwrites; results stay in the borrowed heap.
    pub fn execute_with_heap(
        &self,
        entry: u64,
        arguments: &[KernelValue],
        budget: KernelCallBudget,
        heap: &mut KernelHeapContext<'_>,
    ) -> Result<KernelCallOutcome, KernelModuleError> {
        kernel_module_executor::execute(self, entry, arguments, budget, Some(heap))
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
