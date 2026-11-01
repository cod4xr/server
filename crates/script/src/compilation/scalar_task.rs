// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::binding::LegacyIdentifier;
use crate::compilation::scalar_initialization::ScalarInitializationSnapshot;
use crate::compilation::scalar_loop_transfer::ScalarWhileContext;
use crate::execution::{KernelBranchCondition, KernelInstruction};
use crate::expression::LegacyLogicalOperator;
use crate::syntax::LegacySyntaxNode;

/// Private relocation identity, never a published instruction target or runtime ID.
pub(super) struct ScalarPatchSite(pub(super) usize);

/// Compiler-private loop destination, distinct from an unresolved forward site.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct ScalarLoopHead(pub(super) usize);

pub(super) enum ScalarTask<'tree, 'source> {
    Visit(LegacySyntaxNode<'tree, 'source>),
    Statement {
        syntax: LegacySyntaxNode<'tree, 'source>,
        last: bool,
    },
    FinishAssignment {
        name: LegacyIdentifier<'source>,
        slot: usize,
        span: Range<u64>,
    },
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
    BeginIf {
        consequence: LegacySyntaxNode<'tree, 'source>,
        alternative: Option<LegacySyntaxNode<'tree, 'source>>,
        span: Range<u64>,
    },
    AfterThen {
        false_site: ScalarPatchSite,
        incoming: ScalarInitializationSnapshot,
        alternative: Option<LegacySyntaxNode<'tree, 'source>>,
        span: Range<u64>,
    },
    AfterElse {
        end_site: ScalarPatchSite,
        then_facts: ScalarInitializationSnapshot,
        span: Range<u64>,
    },
    BeginWhileBody {
        head: ScalarLoopHead,
        body: LegacySyntaxNode<'tree, 'source>,
        span: Range<u64>,
    },
    AfterWhileBody {
        head: ScalarLoopHead,
        exit_site: ScalarPatchSite,
        incoming: ScalarInitializationSnapshot,
        outer_loop: Option<ScalarWhileContext>,
        span: Range<u64>,
    },
}

impl ScalarTask<'_, '_> {
    pub(super) fn span(&self) -> Range<u64> {
        match self {
            Self::Visit(node) => node.span(),
            Self::Statement { syntax, .. } => syntax.span(),
            Self::Emit { span, .. }
            | Self::FinishAssignment { span, .. }
            | Self::BeginLogical { span, .. }
            | Self::BeginIf { span, .. }
            | Self::AfterThen { span, .. }
            | Self::AfterElse { span, .. }
            | Self::BeginWhileBody { span, .. }
            | Self::AfterWhileBody { span, .. }
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
    Jump {
        target: Option<usize>,
    },
    Break {
        target: Option<usize>,
        next: Option<ScalarPatchSite>,
    },
}

pub(super) struct PendingScalarInstruction {
    pub(super) kind: PendingScalarKind,
    pub(super) span: Range<u64>,
}
