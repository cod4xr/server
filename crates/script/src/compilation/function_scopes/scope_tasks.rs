// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_condition::ScopeCondition;
use super::{FunctionScopePlan, ScopeAbort, ScopeId, allocation, overflow};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::syntax::LegacySyntaxNode;

#[derive(Clone, Copy, Default)]
pub(super) struct ScopeFlow {
    pub(super) continuation: Option<usize>,
    pub(super) breaking: Option<usize>,
}

pub(super) struct ScopeContext {
    pub(super) parent: ScopeId,
    pub(super) condition: ScopeCondition,
    pub(super) breaks: Vec<ScopeId>,
    pub(super) continues: Vec<ScopeId>,
    pub(super) public_cases: Vec<ScopeId>,
    pub(super) current_case: Option<ScopeId>,
    pub(super) has_default: bool,
    pub(super) is_switch: bool,
    pub(super) exit_abort: ScopeAbort,
}

#[derive(Clone, Copy)]
pub(super) struct ScopeForSource<'tree, 'source> {
    pub(super) condition: Option<LegacySyntaxNode<'tree, 'source>>,
    pub(super) update: Option<LegacySyntaxNode<'tree, 'source>>,
    pub(super) body: LegacySyntaxNode<'tree, 'source>,
}

#[derive(Clone, Copy)]
pub(super) struct ScopeForWork {
    pub(super) parent: ScopeId,
    pub(super) body: ScopeId,
    pub(super) update: ScopeId,
    pub(super) context: usize,
}

pub(super) enum ScopeTaskKind<'tree, 'source> {
    DeveloperEnd {
        parent: ScopeId,
        body: ScopeId,
    },
    Visit {
        syntax: LegacySyntaxNode<'tree, 'source>,
        block: ScopeId,
        flow: ScopeFlow,
    },
    IfEnd {
        parent: ScopeId,
        consequence: ScopeId,
        alternative: Option<ScopeId>,
    },
    IfAfterConsequence {
        parent: ScopeId,
        consequence: ScopeId,
        alternative: LegacySyntaxNode<'tree, 'source>,
        flow: ScopeFlow,
    },
    WhileEnd {
        parent: ScopeId,
        body: ScopeId,
        context: usize,
    },
    ForAfterInitializer {
        parent: ScopeId,
        source: ScopeForSource<'tree, 'source>,
    },
    ForAfterBody {
        work: ScopeForWork,
        update: Option<LegacySyntaxNode<'tree, 'source>>,
    },
    ForEnd(ScopeForWork),
    SwitchStatement {
        syntax: LegacySyntaxNode<'tree, 'source>,
        context: usize,
        flow: ScopeFlow,
    },
    SwitchAfterStatement {
        context: usize,
    },
    SwitchEnd {
        parent: ScopeId,
        context: usize,
    },
}

pub(super) struct ScopeTask<'tree, 'source> {
    pub(super) kind: ScopeTaskKind<'tree, 'source>,
    pub(super) span: Range<u64>,
}

pub(super) struct ScopePlanner<'tree, 'source> {
    pub(super) plan: FunctionScopePlan,
    pub(super) tasks: Vec<ScopeTask<'tree, 'source>>,
    pub(super) contexts: Vec<ScopeContext>,
    pub(super) active_task: bool,
}

impl<'tree, 'source> ScopePlanner<'tree, 'source> {
    pub(super) fn push(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        kind: ScopeTaskKind<'tree, 'source>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let count = self
            .tasks
            .len()
            .checked_add(1)
            .and_then(|count| count.checked_add(usize::from(self.active_task)))
            .ok_or_else(|| overflow(span.clone()))?;
        builder.admit_control(size_of::<ScopeTask<'_, '_>>() as u64, span.clone())?;
        builder.ledger.auxiliary_frames(count, span.clone())?;
        self.tasks
            .try_reserve(1)
            .map_err(|_| allocation(span.clone()))?;
        self.tasks.push(ScopeTask { kind, span });
        Ok(())
    }

    pub(super) fn context(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent: ScopeId,
        condition: ScopeCondition,
        is_switch: bool,
        span: Range<u64>,
    ) -> Result<usize, FunctionCompileError> {
        builder.admit_control(size_of::<ScopeContext>() as u64, span.clone())?;
        self.contexts.try_reserve(1).map_err(|_| allocation(span))?;
        let id = self.contexts.len();
        self.contexts.push(ScopeContext {
            parent,
            condition,
            breaks: Vec::new(),
            continues: Vec::new(),
            public_cases: Vec::new(),
            current_case: None,
            has_default: false,
            is_switch,
            exit_abort: ScopeAbort::Return,
        });
        Ok(id)
    }

    pub(super) fn release_context(
        context: ScopeContext,
        builder: &mut FunctionBuilder<'_, '_>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let rows = context
            .breaks
            .len()
            .checked_add(context.continues.len())
            .and_then(|rows| rows.checked_add(context.public_cases.len()))
            .ok_or_else(|| overflow(span.clone()))?;
        let bytes = (rows as u64)
            .checked_mul(size_of::<ScopeId>() as u64)
            .and_then(|bytes| bytes.checked_add(size_of::<ScopeContext>() as u64))
            .ok_or_else(|| overflow(span.clone()))?;
        drop(context);
        builder.ledger.release_control(bytes, span)
    }
}
