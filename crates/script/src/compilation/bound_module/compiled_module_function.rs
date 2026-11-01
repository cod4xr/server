// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::binding::{LegacyFunctionInclusion, LegacyFunctionVisibility};
use crate::compilation::FunctionCompilationUsage;

/// Retained declaration metadata in original source order, including disabled code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledModuleDeclaration {
    /// Original complete function declaration byte span; caller associates the source.
    pub span: Range<u64>,
    /// Source parameter count, including metadata for inactive declarations.
    pub parameters: u64,
    /// Ordinary/developer-only visibility from the supplied validated binding.
    pub visibility: LegacyFunctionVisibility,
    /// Active/disabled state from the supplied validated binding.
    pub inclusion: LegacyFunctionInclusion,
    /// Active module-local index, or None for explicitly disabled definitions.
    pub function_index: Option<u64>,
}

/// Owned source maps/accounting for one active function, without names/source borrows.
pub struct CompiledModuleFunction {
    pub(super) declaration: usize,
    pub(super) spans: Vec<Range<u64>>,
    pub(super) usage: FunctionCompilationUsage,
}

impl CompiledModuleFunction {
    /// Returns the original source-order declaration index.
    #[must_use]
    pub fn declaration_index(&self) -> u64 {
        self.declaration as u64
    }

    /// Returns the original byte span for an instruction, without retaining source text.
    #[must_use]
    pub fn instruction_span(&self, instruction: u64) -> Option<Range<u64>> {
        self.spans.get(usize::try_from(instruction).ok()?).cloned()
    }

    /// Returns individual code generation/proof accounting; no fabricated execution occurs.
    #[must_use]
    pub fn usage(&self) -> FunctionCompilationUsage {
        self.usage
    }
}

impl fmt::Debug for CompiledModuleFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledModuleFunction")
            .field("declaration_index", &self.declaration_index())
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
