// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::{FunctionScopePlan, ScopeBlock, ScopeControl, ScopeId, allocation, invalid, overflow};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;

impl FunctionScopePlan {
    pub(super) fn retain_metadata(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let retained = self
            .retained_bytes
            .checked_add(bytes)
            .ok_or_else(|| overflow(span.clone()))?;
        builder.admit_control(bytes, span)?;
        self.retained_bytes = retained;
        Ok(())
    }

    pub(super) fn append_block(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        block: ScopeBlock,
        span: Range<u64>,
    ) -> Result<ScopeId, FunctionCompileError> {
        self.retain_metadata(builder, size_of::<ScopeBlock>() as u64, span.clone())?;
        self.blocks.try_reserve(1).map_err(|_| allocation(span))?;
        let id = ScopeId(self.blocks.len());
        self.blocks.push(block);
        Ok(id)
    }

    pub(super) fn insert_control(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        control: ScopeControl,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        builder.ledger.charge(1, span.clone())?;
        if self.controls.contains_key(&span.start) {
            return Err(invalid(span));
        }
        self.retain_metadata(
            builder,
            size_of::<(u64, ScopeControl)>() as u64,
            span.clone(),
        )?;
        self.controls
            .try_reserve(1)
            .map_err(|_| allocation(span.clone()))?;
        self.controls.insert(span.start, control);
        Ok(())
    }

    pub(super) fn insert_label(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        id: ScopeId,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        builder.ledger.charge(1, span.clone())?;
        if self.labels.contains_key(&span.start) {
            return Err(invalid(span));
        }
        self.retain_metadata(builder, size_of::<(u64, ScopeId)>() as u64, span.clone())?;
        self.labels
            .try_reserve(1)
            .map_err(|_| allocation(span.clone()))?;
        self.labels.insert(span.start, id);
        Ok(())
    }

    pub(in crate::compilation) fn release(
        self,
        builder: &mut FunctionBuilder<'_, '_>,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        if self.current != ScopeId(0) {
            return Err(invalid(span));
        }
        builder.ledger.release_control(self.retained_bytes, span)
    }
}
