// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::binding::{CanonicalScriptPath, LegacyDeveloperMode};
use crate::compilation::{
    CompiledModule, CompiledModuleDeclaration, CompiledModuleFunction, ModuleCompilationUsage,
};
use crate::execution::VerifiedKernelModule;

/// Retained input-order module boundaries in the shared verified image.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledScriptModule {
    /// Global declaration range, including disabled definitions.
    pub declarations: Range<u64>,
    /// Global active executable function range.
    pub functions: Range<u64>,
    /// Supplied local binding's developer-script mode.
    pub developer_mode: LegacyDeveloperMode,
}

/// Complete logical package accounting, excluding caller AST/symbols/allocator/RSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScriptPackageUsage {
    /// Retained named modules, including empty/all-disabled modules.
    pub modules: u64,
    /// Owned canonical virtual-name payload bytes.
    pub canonical_module_bytes: u64,
    /// Namespace/code generation/proof counters under shared compiler admission.
    pub compilation: ModuleCompilationUsage,
    /// All retained names/module boundaries/maps, excluding immutable kernel code.
    pub retained_metadata_bytes: u64,
}

/// Source location without source contents, private names or native addresses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptPackageLocation {
    /// Input-order module index.
    pub module: u64,
    /// Original byte span within that module's source.
    pub span: Range<u64>,
}

/// Owned closed script package with one verified function-reference identity.
/// This does not load files, resolve includes, activate game state or provide host services.
pub struct CompiledScriptPackage {
    pub(super) image: CompiledModule,
    pub(super) names: Vec<CanonicalScriptPath>,
    pub(super) modules: Vec<CompiledScriptModule>,
    pub(super) usage: ScriptPackageUsage,
}

impl CompiledScriptPackage {
    /// Borrows the shared immutable image; normal global invocation budgets apply.
    #[must_use]
    pub fn program(&self) -> &VerifiedKernelModule {
        self.image.program()
    }

    /// Borrows input-order module ranges without exposing names in Debug/logs.
    #[must_use]
    pub fn modules(&self) -> &[CompiledScriptModule] {
        &self.modules
    }

    /// Explicitly borrows one canonical virtual name, without filesystem authority.
    #[must_use]
    pub fn module_name(&self, module: u64) -> Option<&[u8]> {
        self.names
            .get(usize::try_from(module).ok()?)
            .map(CanonicalScriptPath::as_bytes)
    }

    /// Borrows global source-order declaration metadata across input modules.
    #[must_use]
    pub fn declarations(&self) -> &[CompiledModuleDeclaration] {
        self.image.declarations()
    }

    /// Borrows active global function maps, whose declaration indices are image-global.
    #[must_use]
    pub fn functions(&self) -> &[CompiledModuleFunction] {
        self.image.functions()
    }

    /// Maps an original module/local declaration pair to an active image function.
    #[must_use]
    pub fn function_for_declaration(&self, module: u64, declaration: u64) -> Option<u64> {
        let range = &self
            .modules
            .get(usize::try_from(module).ok()?)?
            .declarations;
        let index = range.start.checked_add(declaration)?;
        if index >= range.end {
            return None;
        }
        self.image.function_for_declaration(index)
    }

    /// Locates an instruction using numeric source module and original byte span.
    #[must_use]
    pub fn instruction_location(
        &self,
        function: u64,
        instruction: u64,
    ) -> Option<ScriptPackageLocation> {
        let span = self.image.instruction_span(function, instruction)?;
        let (mut low, mut high) = (0, self.modules.len());
        while low < high {
            let middle = low + (high - low) / 2;
            if self.modules[middle].functions.start <= function {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        let module = low.checked_sub(1)?;
        self.modules[module]
            .functions
            .contains(&function)
            .then_some(ScriptPackageLocation {
                module: module as u64,
                span,
            })
    }

    /// Returns admitted namespace/compiler and separately scoped kernel proof usage.
    #[must_use]
    pub fn usage(&self) -> ScriptPackageUsage {
        self.usage
    }
}

impl fmt::Debug for CompiledScriptPackage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledScriptPackage")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
