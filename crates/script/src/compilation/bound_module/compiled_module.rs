// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::compilation::bound_module::{CompiledModuleDeclaration, CompiledModuleFunction};
use crate::execution::{KernelModuleUsage, VerifiedKernelModule};

/// Successful aggregate source-module accounting; excludes allocator rounding/RSS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ModuleCompilationUsage {
    /// Complete active plain/localized literal row count.
    pub string_literals: u64,
    /// Logical complete staged literal rows/payload before separate proof copying.
    pub staged_string_bytes: u64,
    /// Complete active immutable field-name row count.
    pub field_names: u64,
    /// Logical staged field-name rows/payload before separate proof copying.
    pub staged_field_bytes: u64,
    /// Complete decoded switch-label rows, independent of runtime literal caches.
    pub switch_labels: u64,
    /// Logical staged switch-label rows/payload before separate kernel proof copying.
    pub staged_switch_bytes: u64,
    /// All retained declaration records, including disabled developer code.
    pub declarations: u64,
    /// Active executable functions.
    pub functions: u64,
    /// Admitted inventory/name/record visits and all function compiler work.
    pub compiler_steps: u64,
    /// Complete active instruction count.
    pub instructions: u64,
    /// Peak staged prefix plus current temporary code, including discarded developer regions.
    pub peak_instructions: u64,
    /// Peak admitted compiler tables/payload/current-function overlap.
    pub peak_metadata_bytes: u64,
    /// Retained declaration/function-map records and span payload, excluding kernel code.
    pub retained_metadata_bytes: u64,
    /// Separately admitted complete immutable kernel proof.
    pub verification: KernelModuleUsage,
}

/// Owned bound primitive source code/maps; not import linking or engine activation.
///
/// Callers retain module/source names and select entries by original declaration
/// index. Disabled declarations remain explicit metadata. No source/tree/name or
/// symbol-index borrow escapes; private values never appear in Debug output.
pub struct CompiledModule {
    pub(super) program: VerifiedKernelModule,
    pub(super) declarations: Vec<CompiledModuleDeclaration>,
    pub(super) functions: Vec<CompiledModuleFunction>,
    pub(super) usage: ModuleCompilationUsage,
}

impl CompiledModule {
    /// Borrows complete immutable executable code; invocation admits global runtime budgets.
    #[must_use]
    pub fn program(&self) -> &VerifiedKernelModule {
        &self.program
    }

    /// Borrows all source-order declaration metadata without silently dropping disabled code.
    #[must_use]
    pub fn declarations(&self) -> &[CompiledModuleDeclaration] {
        &self.declarations
    }

    /// Borrows active maps/accounting in module-local executable function order.
    #[must_use]
    pub fn functions(&self) -> &[CompiledModuleFunction] {
        &self.functions
    }

    /// Maps an original declaration index to its active executable index, if included.
    #[must_use]
    pub fn function_for_declaration(&self, declaration: u64) -> Option<u64> {
        self.declarations
            .get(usize::try_from(declaration).ok()?)?
            .function_index
    }

    /// Maps an executable function/instruction pair to its original source byte span.
    #[must_use]
    pub fn instruction_span(&self, function: u64, instruction: u64) -> Option<Range<u64>> {
        self.functions
            .get(usize::try_from(function).ok()?)?
            .instruction_span(instruction)
    }

    /// Returns aggregate compiler admission and separate kernel proof accounting.
    #[must_use]
    pub fn usage(&self) -> ModuleCompilationUsage {
        self.usage
    }
}

impl fmt::Debug for CompiledModule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledModule")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
