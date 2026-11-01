// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_condition::ScopeCondition;
use super::scope_tasks::{ScopeContext, ScopePlanner};
use super::{ScopeAbort, ScopeId, allocation, invalid};
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind, function_compile_error};

impl ScopePlanner<'_, '_> {
    pub(super) fn record_transfer(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        block: ScopeId,
        context: Option<usize>,
        continuing: bool,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let context = context.ok_or_else(|| {
            function_compile_error::at(
                span.clone(),
                if continuing {
                    FunctionCompileErrorKind::IllegalContinue
                } else {
                    FunctionCompileErrorKind::IllegalBreak
                },
            )
        })?;
        let context = self
            .contexts
            .get_mut(context)
            .ok_or_else(|| invalid(span.clone()))?;
        if continuing
            || context.is_switch
            || matches!(context.condition, ScopeCondition::Always(true))
        {
            builder.admit_control(size_of::<ScopeId>() as u64, span.clone())?;
            let rows = if continuing {
                &mut context.continues
            } else {
                &mut context.breaks
            };
            rows.try_reserve(1).map_err(|_| allocation(span.clone()))?;
            rows.push(block);
        }
        self.plan.block_mut(block, span)?.abort = if continuing {
            ScopeAbort::Continue
        } else {
            ScopeAbort::Break
        };
        Ok(())
    }

    pub(super) fn implicit_continue(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        context: usize,
        body: ScopeId,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        if self.plan.block(body, span.clone())?.abort == ScopeAbort::Open {
            builder.admit_control(size_of::<ScopeId>() as u64, span.clone())?;
            let rows = &mut self
                .contexts
                .get_mut(context)
                .ok_or_else(|| invalid(span.clone()))?
                .continues;
            rows.try_reserve(1).map_err(|_| allocation(span.clone()))?;
            rows.push(body);
        }
        let rows = &self
            .contexts
            .get(context)
            .ok_or_else(|| invalid(span.clone()))?
            .continues;
        for block in rows {
            builder.ledger.charge(1, span.clone())?;
            self.plan.block_mut(*block, span.clone())?.abort = ScopeAbort::Open;
        }
        Ok(())
    }

    pub(super) fn pop_context(
        &mut self,
        index: usize,
        span: Range<u64>,
    ) -> Result<ScopeContext, FunctionCompileError> {
        if self.contexts.len().checked_sub(1) != Some(index) {
            return Err(invalid(span));
        }
        self.contexts.pop().ok_or_else(|| invalid(span))
    }
}
