// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::binding::LegacyModuleSymbols;

/// Borrowed caller-supplied virtual name and validated local function symbols.
/// No file access, include expansion or runtime activation occurs. Compilation
/// owns its final names/code/maps independently of these source borrows.
#[derive(Clone, Copy)]
pub struct ScriptModuleDefinition<'symbols, 'module, 'tree, 'source> {
    pub(super) name: &'source [u8],
    pub(super) symbols: &'symbols LegacyModuleSymbols<'module, 'tree, 'source>,
}

impl<'symbols, 'module, 'tree, 'source> ScriptModuleDefinition<'symbols, 'module, 'tree, 'source> {
    /// Supplies a closed-catalog identity and validated local definitions.
    /// Name validation/ownership and aggregate admission occur during compilation.
    #[must_use]
    pub fn new(
        name: &'source [u8],
        symbols: &'symbols LegacyModuleSymbols<'module, 'tree, 'source>,
    ) -> Self {
        Self { name, symbols }
    }
}

impl fmt::Debug for ScriptModuleDefinition<'_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScriptModuleDefinition")
            .field("name_bytes", &self.name.len())
            .field("symbols", &self.symbols.usage())
            .finish_non_exhaustive()
    }
}
