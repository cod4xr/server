// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::{FunctionScopePlan, ScopeControl, ScopeId, ScopeLocal, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::execution::{KernelInstruction, KernelValue};

pub(super) fn with_plan<T: Default>(
    builder: &mut FunctionBuilder<'_, '_>,
    operation: impl FnOnce(
        &mut FunctionScopePlan,
        &mut FunctionBuilder<'_, '_>,
    ) -> Result<T, FunctionCompileError>,
) -> Result<T, FunctionCompileError> {
    let Some(mut plan) = builder.scopes.take() else {
        return Ok(T::default());
    };
    let result = operation(&mut plan, builder);
    builder.scopes = Some(plan);
    result
}

pub(in crate::compilation) fn current(builder: &FunctionBuilder<'_, '_>) -> Option<ScopeId> {
    builder.scopes.as_ref().map(|plan| plan.current)
}

pub(in crate::compilation) fn local_store(
    builder: &mut FunctionBuilder<'_, '_>,
    slot: usize,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        builder.ledger.charge(1, span.clone())?;
        let block = plan.block_mut(plan.current, span.clone())?;
        let position = block
            .positions
            .get(&ScopeLocal(slot))
            .copied()
            .ok_or_else(|| invalid(span.clone()))?;
        let created = position
            .checked_add(1)
            .ok_or_else(|| super::overflow(span))?;
        block.created = block.created.max(created);
        Ok(())
    })
}

pub(in crate::compilation) fn break_edge(
    builder: &mut FunctionBuilder<'_, '_>,
    target: Option<ScopeId>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let target = target.ok_or_else(|| invalid(span.clone()))?;
        let public = plan.block(target, span.clone())?.public;
        plan.clear(builder, public, span)
    })
}

pub(in crate::compilation) fn continue_edge(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| plan.clear_current(builder, span))
}

impl FunctionScopePlan {
    pub(super) fn control(
        &self,
        builder: &mut FunctionBuilder<'_, '_>,
        span: Range<u64>,
    ) -> Result<ScopeControl, FunctionCompileError> {
        builder.ledger.charge(1, span.clone())?;
        self.controls
            .get(&span.start)
            .copied()
            .ok_or_else(|| invalid(span))
    }

    pub(super) fn enter(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent: ScopeId,
        child: ScopeId,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        if self.current != parent {
            return Err(invalid(span));
        }
        self.transfer(builder, parent, child, span)?;
        self.current = child;
        Ok(())
    }

    pub(super) fn clear(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        target_public: usize,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let block = self.block(self.current, span.clone())?;
        let created = block.created;
        if target_public > created || created > block.names.len() {
            return Err(invalid(span));
        }
        if created == target_public {
            return Ok(());
        }
        for position in (target_public..created).rev() {
            builder.ledger.charge(1, span.clone())?;
            let slot = self.block(self.current, span.clone())?.names[position].0;
            builder.emit(
                KernelInstruction::Constant(KernelValue::Undefined),
                span.clone(),
            )?;
            builder.emit(KernelInstruction::StoreLocal(slot), span.clone())?;
            crate::compilation::function_local_names::uninitialize(builder, slot, span.clone())?;
        }
        let block = self.block_mut(self.current, span)?;
        // The native break reset uses the current public count, even when the
        // removal range used a distinct break-target public count.
        block.created = block.public;
        Ok(())
    }

    pub(super) fn clear_current(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let public = self.block(self.current, span.clone())?.public;
        self.clear(builder, public, span)
    }
}
