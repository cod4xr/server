// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::execution::VerifiedKernelProgram;

/// Successful restricted source compiler accounting, excluding allocator overhead/RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScalarCompilationUsage {
    /// Body/task/literal-byte/resolution work charged to the compiler.
    pub compiler_steps: u64,
    /// Emitted instructions and source-map entries.
    pub instructions: u64,
    /// Peak live iterative lowering frames.
    pub peak_task_frames: u64,
    /// Peak admitted logical compiler-owned metadata; excludes source/CST and kernel scopes.
    pub peak_metadata_bytes: u64,
    /// Actual verification scans/visits/edges in the separate kernel scope.
    pub verification_steps: u64,
    /// Actual instructions run during separate pure validation execution.
    pub validation_steps: u64,
}

/// Owned pure scalar function code/maps, with no source/tree/name references.
///
/// The profile accepts no parameters and only empty/single-return bodies. It is
/// not a full GSC function/module compiler or activation mechanism. Callers retain
/// the source/module association for these original byte spans and bind names.
pub struct CompiledScalarFunction {
    pub(super) program: VerifiedKernelProgram,
    pub(super) spans: Vec<Range<u64>>,
    pub(super) usage: ScalarCompilationUsage,
}

impl CompiledScalarFunction {
    /// Borrows the immutable scalar proof; invocation has its own stack/fuel budget.
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
    pub fn usage(&self) -> ScalarCompilationUsage {
        self.usage
    }
}

impl fmt::Debug for CompiledScalarFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledScalarFunction")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
