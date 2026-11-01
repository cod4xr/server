// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::binding::LegacyModuleSymbols;
use crate::lexical::LegacyLexeme;

/// Borrowed caller-supplied virtual name and validated local function symbols.
/// Includes default to empty; opt-in directive slices resolve only supplied modules.
/// No file access, automatic dependency loading or runtime activation occurs. Compilation
/// owns its final names/code/maps independently of these source borrows.
#[derive(Clone, Copy)]
pub struct ScriptModuleDefinition<'symbols, 'module, 'tree, 'source> {
    pub(super) name: &'source [u8],
    pub(super) symbols: &'symbols LegacyModuleSymbols<'module, 'tree, 'source>,
    pub(super) includes: &'module [LegacyLexeme<'source>],
}

impl<'symbols, 'module, 'tree, 'source> ScriptModuleDefinition<'symbols, 'module, 'tree, 'source> {
    /// Supplies a closed-catalog identity and validated local definitions.
    /// Name validation/ownership and aggregate admission occur during compilation.
    #[must_use]
    pub fn new(
        name: &'source [u8],
        symbols: &'symbols LegacyModuleSymbols<'module, 'tree, 'source>,
    ) -> Self {
        Self {
            name,
            symbols,
            includes: &[],
        }
    }

    /// Supplies validated include-path lexemes from this module's declaration view.
    /// Resolution stays within the explicit catalog; this grants no host file access.
    #[must_use]
    pub fn with_includes(mut self, includes: &'module [LegacyLexeme<'source>]) -> Self {
        self.includes = includes;
        self
    }
}

impl fmt::Debug for ScriptModuleDefinition<'_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScriptModuleDefinition")
            .field("name_bytes", &self.name.len())
            .field("symbols", &self.symbols.usage())
            .field("includes", &self.includes.len())
            .finish_non_exhaustive()
    }
}
