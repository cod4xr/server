// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::expression::legacy_syntax_profile;
use crate::expression::{LegacyExpressionError, LegacyPrimaryKind, LegacyPrimarySyntax};
use crate::lexical::LegacyLexeme;
use crate::syntax::LegacySyntaxNode;

/// Borrowed original function/path spelling without interning, normalization, or IDs.
#[derive(Clone, Copy, Debug)]
pub struct LegacyFunctionNameSyntax<'tree, 'source> {
    syntax: LegacySyntaxNode<'tree, 'source>,
    module: Option<LegacyLexeme<'source>>,
    name: LegacyLexeme<'source>,
}

impl<'tree, 'source> LegacyFunctionNameSyntax<'tree, 'source> {
    /// Classifies a named callee/reference production or function-reference primary.
    ///
    /// Reads a constant number of children without allocation, case folding,
    /// module resolution, I/O, binding, or execution. Module tokens remain virtual
    /// source spelling, never permission to open a host path.
    ///
    /// # Errors
    /// Rejects unrelated nodes and mismatched pinned child/name profiles. Source
    /// state remains unchanged and no partial view is returned.
    pub fn from_node(
        syntax: LegacySyntaxNode<'tree, 'source>,
    ) -> Result<Self, LegacyExpressionError> {
        let node = if matches!(legacy_syntax_profile::production(syntax), Some(30..=33)) {
            syntax
        } else {
            let primary = match LegacyPrimarySyntax::from_node(syntax) {
                Ok(primary) => primary,
                Err(LegacyExpressionError::NotExpression | LegacyExpressionError::NotPrimary) => {
                    return Err(LegacyExpressionError::NotFunctionName);
                }
                Err(error) => return Err(error),
            };
            let LegacyPrimaryKind::FunctionReference { syntax } = primary.kind() else {
                return Err(LegacyExpressionError::NotFunctionName);
            };
            syntax
        };
        let child = |index, count| legacy_syntax_profile::child(node, index, count);
        let token = |index, count| {
            child(index, count)?
                .lexeme()
                .ok_or(LegacyExpressionError::ProfileMismatch)
        };
        let (module, name) = match legacy_syntax_profile::production(node) {
            Some(30 | 32) => {
                let path = child(0, 3)?;
                if !matches!(legacy_syntax_profile::production(path), Some(28 | 29)) {
                    return Err(LegacyExpressionError::ProfileMismatch);
                }
                let module = legacy_syntax_profile::child(path, 0, 1)?
                    .lexeme()
                    .ok_or(LegacyExpressionError::ProfileMismatch)?;
                (Some(module), token(2, 3)?)
            }
            Some(31) => (None, token(0, 1)?),
            Some(33) => (None, token(1, 2)?),
            _ => return Err(LegacyExpressionError::NotFunctionName),
        };
        Ok(Self {
            syntax,
            module,
            name,
        })
    }

    /// Returns exact optional virtual module/path spelling, without resolving it.
    #[must_use]
    pub fn module(self) -> Option<LegacyLexeme<'source>> {
        self.module
    }

    /// Returns the exact original identifier; binding owns legacy name identity.
    #[must_use]
    pub fn name(self) -> LegacyLexeme<'source> {
        self.name
    }

    /// Returns the original supplied node, preserving any primary wrapper.
    #[must_use]
    pub fn syntax(self) -> LegacySyntaxNode<'tree, 'source> {
        self.syntax
    }

    /// Returns the original supplied node's half-open source byte span.
    #[must_use]
    pub fn span(self) -> Range<u64> {
        self.syntax.span()
    }
}
