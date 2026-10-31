// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::execution::{KernelBranchCondition, KernelInstruction};
use crate::expression::LegacyLogicalOperator;
use crate::syntax::LegacySyntaxNode;

/// Private relocation identity, never a published instruction target or runtime ID.
pub(super) struct ScalarPatchSite(pub(super) usize);

pub(super) enum ScalarTask<'tree, 'source> {
    Visit(LegacySyntaxNode<'tree, 'source>),
    Emit {
        instruction: KernelInstruction,
        span: Range<u64>,
    },
    BeginLogical {
        operator: LegacyLogicalOperator,
        right: LegacySyntaxNode<'tree, 'source>,
        span: Range<u64>,
    },
    EndLogical {
        site: ScalarPatchSite,
        span: Range<u64>,
    },
}

impl ScalarTask<'_, '_> {
    pub(super) fn span(&self) -> Range<u64> {
        match self {
            Self::Visit(node) => node.span(),
            Self::Emit { span, .. }
            | Self::BeginLogical { span, .. }
            | Self::EndLogical { span, .. } => span.clone(),
        }
    }
}

pub(super) enum PendingScalarKind {
    Ready(KernelInstruction),
    Branch {
        condition: KernelBranchCondition,
        target: Option<usize>,
    },
}

pub(super) struct PendingScalarInstruction {
    pub(super) kind: PendingScalarKind,
    pub(super) span: Range<u64>,
}
