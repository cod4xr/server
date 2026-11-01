// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;
use std::ops::Range;

use super::{FunctionScopePlan, ScopeAbort, ScopeBlock, ScopeId, ScopeLocal, allocation, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;

const NAME_BYTES: u64 = (size_of::<ScopeLocal>() + size_of::<(ScopeLocal, usize)>()) as u64;

impl FunctionScopePlan {
    pub(super) fn root(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parameters: usize,
        span: Range<u64>,
    ) -> Result<ScopeId, FunctionCompileError> {
        let root = self.append_block(
            builder,
            ScopeBlock {
                parent: None,
                names: Vec::new(),
                positions: HashMap::new(),
                public: parameters,
                created: parameters,
                abort: ScopeAbort::Open,
            },
            span.clone(),
        )?;
        for index in 0..parameters {
            self.ensure_name(builder, root, ScopeLocal(index), span.clone())?;
        }
        Ok(root)
    }

    pub(super) fn clone_block(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent_id: ScopeId,
        span: Range<u64>,
    ) -> Result<ScopeId, FunctionCompileError> {
        let parent = self.block(parent_id, span.clone())?;
        let count = parent.names.len();
        let mut copy = ScopeBlock {
            parent: Some(parent_id),
            names: Vec::new(),
            positions: HashMap::new(),
            public: 0,
            created: parent.created,
            abort: parent.abort,
        };
        // Reserve complete staging while the parent's names remain admitted.
        let bytes = (count as u64)
            .checked_mul(NAME_BYTES)
            .ok_or_else(|| super::overflow(span.clone()))?;
        self.retain_metadata(builder, bytes, span.clone())?;
        copy.names
            .try_reserve_exact(count)
            .map_err(|_| allocation(span.clone()))?;
        copy.positions
            .try_reserve(count)
            .map_err(|_| allocation(span.clone()))?;
        let parent = self.block(parent_id, span.clone())?;
        for name in &parent.names {
            builder.ledger.charge(2, span.clone())?;
            copy.positions.insert(*name, copy.names.len());
            copy.names.push(*name);
        }
        self.append_block(builder, copy, span)
    }

    pub(super) fn ensure_name(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        id: ScopeId,
        name: ScopeLocal,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        builder.ledger.charge(1, span.clone())?;
        if self.block(id, span.clone())?.positions.contains_key(&name) {
            return Ok(());
        }
        self.retain_metadata(builder, NAME_BYTES, span.clone())?;
        let block = self.block_mut(id, span.clone())?;
        block
            .names
            .try_reserve(1)
            .map_err(|_| allocation(span.clone()))?;
        block
            .positions
            .try_reserve(1)
            .map_err(|_| allocation(span.clone()))?;
        builder.ledger.charge(1, span)?;
        block.positions.insert(name, block.names.len());
        block.names.push(name);
        Ok(())
    }

    pub(super) fn align_prefix(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent: ScopeId,
        child: ScopeId,
        count: usize,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        if parent == child || count > self.block(parent, span.clone())?.names.len() {
            return Err(invalid(span));
        }
        for index in 0..count {
            self.place_name(builder, parent, child, index, false, span.clone())?;
        }
        self.block_mut(child, span)?.public = count;
        Ok(())
    }

    pub(super) fn transfer(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent: ScopeId,
        child: ScopeId,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        if parent == child {
            return Err(invalid(span));
        }
        let created = self.block(parent, span.clone())?.created;
        let mut index = 0;
        while index < self.block(child, span.clone())?.public.max(created) {
            self.place_name(builder, parent, child, index, true, span.clone())?;
            index = index
                .checked_add(1)
                .ok_or_else(|| super::overflow(span.clone()))?;
        }
        self.block_mut(child, span)?.created = created;
        Ok(())
    }

    fn place_name(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent: ScopeId,
        child: ScopeId,
        index: usize,
        extend_public: bool,
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        builder.ledger.charge(2, span.clone())?;
        let name = *self
            .block(parent, span.clone())?
            .names
            .get(index)
            .ok_or_else(|| invalid(span.clone()))?;
        self.ensure_name(builder, child, name, span.clone())?;
        let block = self.block_mut(child, span.clone())?;
        let previous = *block
            .positions
            .get(&name)
            .ok_or_else(|| invalid(span.clone()))?;
        if previous < index {
            return Err(invalid(span));
        }
        if extend_public && previous >= block.public {
            block.public = block
                .public
                .checked_add(1)
                .ok_or_else(|| super::overflow(span.clone()))?;
        }
        builder
            .ledger
            .charge((previous - index) as u64, span.clone())?;
        for position in (index..previous).rev() {
            let moved = block.names[position];
            block.names[position + 1] = moved;
            *block
                .positions
                .get_mut(&moved)
                .ok_or_else(|| invalid(span.clone()))? = position + 1;
        }
        block.names[index] = name;
        *block
            .positions
            .get_mut(&name)
            .ok_or_else(|| invalid(span.clone()))? = index;
        Ok(())
    }
}
