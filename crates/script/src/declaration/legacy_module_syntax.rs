// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::declaration::declaration_lowering;
use crate::declaration::{DeclarationBudget, DeclarationError, LegacyModuleItem};
use crate::lexical::LegacyLexeme;
use crate::syntax::LegacySyntaxTree;

/// Successful logical declaration extraction counts, excluding tree/source costs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyModuleUsage {
    /// Top-level function/directive/developer items.
    pub items: u64,
    /// Include directives, including duplicates.
    pub includes: u64,
    /// Parameters across all function declarations, including duplicates.
    pub parameters: u64,
    /// Admitted node visits, including tree scan and parameter-list lowering.
    pub steps: u64,
    /// Logical retained item/include/parameter storage, not allocator overhead/RSS.
    pub metadata_bytes: u64,
}

/// Complete typed top-level syntax for one legacy Script tree.
///
/// This owns bounded declaration metadata, borrows names and structural bodies,
/// and does not resolve files, intern/fold names, check duplicates/developer balance,
/// decode directive strings, lower body expressions, compile, or run scripts.
pub struct LegacyModuleSyntax<'tree, 'source> {
    pub(super) includes: Vec<LegacyLexeme<'source>>,
    pub(super) items: Vec<LegacyModuleItem<'tree, 'source>>,
    pub(super) usage: LegacyModuleUsage,
}

impl<'tree, 'source> LegacyModuleSyntax<'tree, 'source> {
    /// Extracts typed declarations from an immutable source-grounded Script tree.
    ///
    /// Node visits/storage are admitted before fallible metadata growth. Source
    /// spelling/order/duplicates remain unchanged; no native string pool is used.
    ///
    /// # Errors
    /// Rejects Fragment entry, invariant failures, exhausted budgets, size overflow,
    /// or allocation failure. Returns no partial module and mutates no input/tree.
    pub fn lower(
        tree: &'tree LegacySyntaxTree<'source>,
        budget: DeclarationBudget,
    ) -> Result<Self, DeclarationError> {
        declaration_lowering::lower(tree, budget)
    }

    /// Borrows ordered include-path lexemes without normalizing or opening files.
    #[must_use]
    pub fn includes(&self) -> &[LegacyLexeme<'source>] {
        &self.includes
    }

    /// Borrows all top-level items in original declaration order.
    #[must_use]
    pub fn items(&self) -> &[LegacyModuleItem<'tree, 'source>] {
        &self.items
    }

    /// Returns successful logical storage/work accounting, excluding tree/source.
    #[must_use]
    pub fn usage(&self) -> LegacyModuleUsage {
        self.usage
    }
}

impl fmt::Debug for LegacyModuleSyntax<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyModuleSyntax")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
