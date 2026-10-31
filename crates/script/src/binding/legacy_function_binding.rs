// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::binding::LegacyIdentifier;
use crate::declaration::LegacyFunctionSyntax;

/// Source-level function visibility, independent of a native code-position type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFunctionVisibility {
    /// Definition outside developer regions.
    Ordinary,
    /// Definition inside a developer region; call/link policy remains separate.
    DeveloperOnly,
}

/// Whether a recorded definition participates in this module's active lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFunctionInclusion {
    /// Included by the selected developer-script mode.
    Active,
    /// Retained as metadata but excluded because developer scripts are disabled.
    DeveloperDisabled,
}

/// Scoped local function binding; no runtime ID, native pointer, or bytecode address.
#[derive(Clone, Copy)]
pub struct LegacyFunctionBinding<'module, 'tree, 'source> {
    pub(super) declaration: &'module LegacyFunctionSyntax<'tree, 'source>,
    pub(super) name: LegacyIdentifier<'source>,
    pub(super) visibility: LegacyFunctionVisibility,
    pub(super) inclusion: LegacyFunctionInclusion,
}

impl<'module, 'tree, 'source> LegacyFunctionBinding<'module, 'tree, 'source> {
    /// Borrows the immutable original signature and structural body.
    #[must_use]
    pub fn declaration(self) -> &'module LegacyFunctionSyntax<'tree, 'source> {
        self.declaration
    }

    /// Returns canonical textual identity while retaining the original spelling.
    #[must_use]
    pub fn name(self) -> LegacyIdentifier<'source> {
        self.name
    }

    /// Returns ordinary/developer-only classification without filtering definitions.
    #[must_use]
    pub fn visibility(self) -> LegacyFunctionVisibility {
        self.visibility
    }

    /// Returns active/excluded classification without discarding source metadata.
    #[must_use]
    pub fn inclusion(self) -> LegacyFunctionInclusion {
        self.inclusion
    }
}

impl fmt::Debug for LegacyFunctionBinding<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyFunctionBinding")
            .field("visibility", &self.visibility)
            .field("inclusion", &self.inclusion)
            .field("offset", &self.declaration.name().span().start)
            .finish_non_exhaustive()
    }
}
