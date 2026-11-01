// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_tasks::{ScopeFlow, ScopePlanner, ScopeTaskKind};
use super::{ScopeAbort, ScopeControl, ScopeId, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::syntax::LegacySyntaxNode;

impl<'tree, 'source> ScopePlanner<'tree, 'source> {
    pub(super) fn begin_if(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        parent: ScopeId,
        consequence: LegacySyntaxNode<'tree, 'source>,
        alternative: Option<LegacySyntaxNode<'tree, 'source>>,
        flow: ScopeFlow,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let child = self.plan.clone_block(builder, parent, span.clone())?;
        self.plan.insert_control(
            builder,
            ScopeControl::If {
                parent,
                consequence: child,
                alternative: None,
            },
            span.clone(),
        )?;
        let finish = if let Some(alternative) = alternative {
            ScopeTaskKind::IfAfterConsequence {
                parent,
                consequence: child,
                alternative,
                flow,
            }
        } else {
            ScopeTaskKind::IfEnd {
                parent,
                consequence: child,
                alternative: None,
            }
        };
        self.push(builder, finish, span)?;
        self.visit_task(builder, consequence, child, flow)
    }

    pub(super) fn after_consequence(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        parent: ScopeId,
        consequence: ScopeId,
        alternative: LegacySyntaxNode<'tree, 'source>,
        flow: ScopeFlow,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let child = self.plan.clone_block(builder, parent, span.clone())?;
        let Some(ScopeControl::If {
            alternative: slot, ..
        }) = self.plan.controls.get_mut(&span.start)
        else {
            return Err(invalid(span));
        };
        if slot.replace(child).is_some() {
            return Err(invalid(span));
        }
        self.push(
            builder,
            ScopeTaskKind::IfEnd {
                parent,
                consequence,
                alternative: Some(child),
            },
            span,
        )?;
        self.visit_task(builder, alternative, child, flow)
    }

    pub(super) fn finish_if(
        &mut self,
        builder: &mut FunctionBuilder<'tree, 'source>,
        parent: ScopeId,
        consequence: ScopeId,
        alternative: Option<ScopeId>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let Some(alternative) = alternative else {
            self.plan
                .make_public(builder, parent, &[consequence], span)?;
            return Ok(());
        };
        let left = self.plan.block(consequence, span.clone())?.abort;
        let right = self.plan.block(alternative, span.clone())?.abort;
        self.plan.block_mut(parent, span.clone())?.abort = left.min(right);
        let mut reaching = [consequence, alternative];
        let count = match (left == ScopeAbort::Open, right == ScopeAbort::Open) {
            (true, true) => 2,
            (true, false) => 1,
            (false, true) => {
                reaching[0] = alternative;
                1
            }
            (false, false) => 0,
        };
        self.plan
            .intersect_names(builder, parent, &reaching[..count], span.clone())?;
        self.plan
            .make_public(builder, parent, &reaching[..count], span)?;
        Ok(())
    }
}
