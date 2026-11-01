// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::execution::VerifiedKernelProgram;

/// Explicit accepted source/validation contract for an owned function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionSourceProfile {
    /// Zero parameters and empty/single-return body, validated by bounded pure execution.
    PureScalarReturn,
    /// Numeric/vector expressions, locals, updates, blocks, branches, loops/transfers and returns.
    /// Publication verifies code without executing fabricated argument values.
    PrimitiveStatements,
}

/// Successful restricted source compiler accounting, excluding allocator overhead/RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionCompilationUsage {
    /// Body/task/literal/name/snapshot/resolution work charged to the compiler.
    pub compiler_steps: u64,
    /// Emitted instructions and source-map entries.
    pub instructions: u64,
    /// Peak live iterative lowering frames.
    pub peak_task_frames: u64,
    /// Peak admitted logical compiler-owned metadata; excludes source/CST and kernel scopes.
    pub peak_metadata_bytes: u64,
    /// Actual verification scans/visits/edges in the separate kernel scope.
    pub verification_steps: u64,
    /// Bound local names, including parameters, in the local-statements profile.
    pub local_bindings: u64,
    /// Identifier bytes charged across declaration/read/initialization visits.
    pub identifier_bytes: u64,
    /// Peak admitted live initialization snapshot flag payload.
    pub peak_control_bytes: u64,
    /// Actual pure validation instructions; None when this profile does not pre-execute.
    pub validation_steps: Option<u64>,
}

/// Owned restricted function code/maps, with no source/tree/name references.
///
/// The explicit profile controls accepted syntax/validation; neither implements
/// full GSC function/module compilation or activation. Callers retain source/module
/// association for these byte spans and resolve module/function names.
pub struct CompiledFunction {
    pub(super) profile: FunctionSourceProfile,
    pub(super) program: VerifiedKernelProgram,
    pub(super) spans: Vec<Range<u64>>,
    pub(super) usage: FunctionCompilationUsage,
}

impl CompiledFunction {
    /// Returns the source/validation contract without source names or values.
    #[must_use]
    pub fn profile(&self) -> FunctionSourceProfile {
        self.profile
    }
    /// Borrows the immutable primitive proof; invocation admits arguments/locals/stack/fuel.
    #[must_use]
    pub fn program(&self) -> &VerifiedKernelProgram {
        &self.program
    }

    /// Maps an instruction to its original source span without storing source text.
    #[must_use]
    pub fn instruction_span(&self, instruction: u64) -> Option<Range<u64>> {
        self.spans.get(usize::try_from(instruction).ok()?).cloned()
    }

    /// Returns successful compiler/kernel accounting, without constants or source.
    #[must_use]
    pub fn usage(&self) -> FunctionCompilationUsage {
        self.usage
    }
}

impl fmt::Debug for CompiledFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledFunction")
            .field("profile", &self.profile)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
