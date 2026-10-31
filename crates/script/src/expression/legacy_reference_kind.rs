// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::lexical::LegacyLexeme;
use crate::syntax::LegacySyntaxNode;

/// Immediate reference role, without binding, mutability/type checks, or evaluation.
#[derive(Clone, Copy, Debug)]
pub enum LegacyReferenceKind<'tree, 'source> {
    /// Unresolved identifier, before scope and canonical-name policy.
    Name {
        /// Exact original identifier token.
        name: LegacyLexeme<'source>,
    },
    /// Dollar-prefixed identifier; source form does not grant runtime authority.
    DollarName {
        /// Exact identifier following the dollar terminal.
        name: LegacyLexeme<'source>,
    },
    /// Ordinary named field, without resolving its object or canonical field key.
    Field {
        /// Original primitive object operand.
        object: LegacySyntaxNode<'tree, 'source>,
        /// Original identifier after the dot, preserving case.
        name: LegacyLexeme<'source>,
    },
    /// Indexed reference, without deciding collection/index types or evaluating them.
    Index {
        /// Original primitive collection/object operand.
        object: LegacySyntaxNode<'tree, 'source>,
        /// Original index expression, left unevaluated.
        index: LegacySyntaxNode<'tree, 'source>,
    },
    /// Reserved .self production, distinct from an ordinary .Self identifier field.
    SelfMember {
        /// Original primitive object; no runtime operation or writable slot selected.
        object: LegacySyntaxNode<'tree, 'source>,
    },
}
