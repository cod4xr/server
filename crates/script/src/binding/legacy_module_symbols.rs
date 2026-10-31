// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;
use std::fmt;

use crate::binding::symbol_builder;
use crate::binding::{
    LegacyDeveloperMode, LegacyFunctionBinding, LegacyIdentifier, SymbolBudget, SymbolError,
};
use crate::declaration::LegacyModuleSyntax;

/// Logical successful local symbol accounting; excludes allocator/hash controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacySymbolUsage {
    /// All local function definitions, including developer-only definitions.
    pub functions: u64,
    /// Definitions admitted to lookup by the selected developer-script mode.
    pub active_functions: u64,
    /// Definitions outside developer regions.
    pub ordinary_functions: u64,
    /// Definitions inside developer regions.
    pub developer_functions: u64,
    /// Borrowed function-name bytes, not owned payload memory.
    pub name_bytes: u64,
    /// Largest declared name, including inactive definitions.
    pub largest_name_bytes: u64,
    /// Largest active indexed name, bounding subsequent lookup hashing.
    pub largest_indexed_name_bytes: u64,
    /// Logical item visits plus admitted name bytes.
    pub steps: u64,
    /// Logical function-record and key/value storage; not retained capacity or RSS.
    pub metadata_bytes: u64,
}

/// Complete local function-name index plus validated top-level developer regions.
///
/// Names remain borrowed, with exact legacy ASCII identity. All definitions retain
/// source order/visibility. This resolves local definitions only: imports, call
/// expressions, parameters/locals, builtins, bytecode, and execution remain open.
pub struct LegacyModuleSymbols<'module, 'tree, 'source> {
    pub(super) functions: Vec<LegacyFunctionBinding<'module, 'tree, 'source>>,
    pub(super) names: HashMap<LegacyIdentifier<'source>, usize>,
    pub(super) usage: LegacySymbolUsage,
    pub(super) mode: LegacyDeveloperMode,
}

impl<'module, 'tree, 'source> LegacyModuleSymbols<'module, 'tree, 'source> {
    /// Admits budgets/storage before fallible allocation and local name binding.
    ///
    /// Rejects duplicate active names and invalid developer regions. Inactive
    /// developer declarations remain recorded under the explicit selected mode.
    /// Directive placement
    /// errors follow definition/region checks, mirroring the source's phase order;
    /// native compiler error recovery and complete error precedence are not claimed.
    /// No script/engine state is activated and no definitions are silently filtered.
    ///
    /// # Errors
    /// Rejects name/profile, semantic, size, allocation, and budget failures without
    /// returning a partial index or changing the declaration/tree/source input.
    pub fn bind(
        module: &'module LegacyModuleSyntax<'tree, 'source>,
        mode: LegacyDeveloperMode,
        budget: SymbolBudget,
    ) -> Result<Self, SymbolError> {
        symbol_builder::bind(module, mode, budget)
    }

    /// Borrows local definitions in source order, including developer-only entries.
    #[must_use]
    pub fn functions(&self) -> &[LegacyFunctionBinding<'module, 'tree, 'source>] {
        &self.functions
    }

    /// Finds an active canonical local definition without allocation. Disabled
    /// developer definitions are excluded; retained visibility still needs call/link checks.
    /// Work is linear in the queried name; names larger than every indexed name
    /// return None before hashing. The map uses the standard randomized hasher.
    #[must_use]
    pub fn find(
        &self,
        name: LegacyIdentifier<'_>,
    ) -> Option<LegacyFunctionBinding<'module, 'tree, 'source>> {
        if name.as_bytes().len() as u64 > self.usage.largest_indexed_name_bytes {
            return None;
        }
        self.names
            .get(&name)
            .and_then(|index| self.functions.get(*index))
            .copied()
    }

    /// Returns successful logical resource accounting, excluding source/tree costs.
    #[must_use]
    pub fn usage(&self) -> LegacySymbolUsage {
        self.usage
    }

    /// Returns the explicit developer-script mode used for this index.
    #[must_use]
    pub fn mode(&self) -> LegacyDeveloperMode {
        self.mode
    }
}

impl fmt::Debug for LegacyModuleSymbols<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyModuleSymbols")
            .field("mode", &self.mode)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
