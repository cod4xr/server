// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::binding::LegacyIdentifier;
use crate::compilation::function_for::{FunctionForEnd, FunctionForPlan};
use crate::compilation::function_initialization::FunctionInitializationSnapshot;
use crate::compilation::function_loop_transfer::FunctionLoopContext;
use crate::execution::{KernelBranchCondition, KernelInstruction};
use crate::expression::LegacyLogicalOperator;
use crate::syntax::LegacySyntaxNode;

/// Private relocation identity, never a published instruction target or runtime ID.
pub(super) struct FunctionPatchSite(pub(super) usize);

/// Compiler-private loop destination, distinct from an unresolved forward site.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct FunctionLoopHead(pub(super) usize);

pub(super) enum FunctionTask<'tree, 'source> {
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
        site: FunctionPatchSite,
        span: Range<u64>,
    },
    BeginIf {
        consequence: LegacySyntaxNode<'tree, 'source>,
        alternative: Option<LegacySyntaxNode<'tree, 'source>>,
        span: Range<u64>,
    },
    AfterThen {
        false_site: FunctionPatchSite,
        incoming: FunctionInitializationSnapshot,
        alternative: Option<LegacySyntaxNode<'tree, 'source>>,
        span: Range<u64>,
    },
    AfterElse {
        end_site: FunctionPatchSite,
        then_facts: FunctionInitializationSnapshot,
        span: Range<u64>,
    },
    BeginWhileBody {
        head: FunctionLoopHead,
        body: LegacySyntaxNode<'tree, 'source>,
        span: Range<u64>,
    },
    AfterWhileBody {
        head: FunctionLoopHead,
        exit_site: FunctionPatchSite,
        incoming: FunctionInitializationSnapshot,
        outer_loop: Option<FunctionLoopContext>,
        span: Range<u64>,
    },
    AfterForInit(FunctionForPlan<'tree, 'source>),
    BeginForBody {
        head: FunctionLoopHead,
        plan: FunctionForPlan<'tree, 'source>,
    },
    AfterForBody {
        end: FunctionForEnd,
        update: Option<LegacySyntaxNode<'tree, 'source>>,
    },
    AfterForUpdate(FunctionForEnd),
}

impl FunctionTask<'_, '_> {
    pub(super) fn span(&self) -> Range<u64> {
        match self {
            Self::Visit(node) => node.span(),
            Self::Statement { syntax, .. } => syntax.span(),
            Self::AfterForInit(plan) | Self::BeginForBody { plan, .. } => plan.span.clone(),
            Self::AfterForBody { end, .. } | Self::AfterForUpdate(end) => end.span.clone(),
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

pub(super) enum PendingFunctionKind {
    Ready(KernelInstruction),
    Branch {
        condition: KernelBranchCondition,
        target: Option<usize>,
    },
    Jump {
        target: Option<usize>,
    },
    Transfer {
        target: Option<usize>,
        next: Option<FunctionPatchSite>,
    },
}

pub(super) struct PendingFunctionInstruction {
    pub(super) kind: PendingFunctionKind,
    pub(super) span: Range<u64>,
}
