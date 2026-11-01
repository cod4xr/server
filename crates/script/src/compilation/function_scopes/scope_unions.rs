// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::{FunctionScopePlan, ScopeId, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;

impl FunctionScopePlan {
    pub(super) fn union_names(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        target: ScopeId,
        sources: &[ScopeId],
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        for source in sources {
            let count = self.block(*source, span.clone())?.names.len();
            for index in 0..count {
                builder.ledger.charge(1, span.clone())?;
                let name = self.block(*source, span.clone())?.names[index];
                self.ensure_name(builder, target, name, span.clone())?;
            }
        }
        Ok(())
    }

    pub(super) fn intersect_names(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        target: ScopeId,
        sources: &[ScopeId],
        span: Range<u64>,
    ) -> Result<(), FunctionCompileError> {
        let Some(first) = sources.first().copied() else {
            return Ok(());
        };
        let count = self.block(first, span.clone())?.names.len();
        for index in 0..count {
            builder.ledger.charge(1, span.clone())?;
            let name = self.block(first, span.clone())?.names[index];
            let mut shared = true;
            for source in &sources[1..] {
                builder.ledger.charge(1, span.clone())?;
                if !self
                    .block(*source, span.clone())?
                    .positions
                    .contains_key(&name)
                {
                    shared = false;
                    break;
                }
            }
            if shared {
                self.ensure_name(builder, target, name, span.clone())?;
            }
        }
        Ok(())
    }

    pub(super) fn make_public(
        &mut self,
        builder: &mut FunctionBuilder<'_, '_>,
        parent: ScopeId,
        sources: &[ScopeId],
        span: Range<u64>,
    ) -> Result<usize, FunctionCompileError> {
        let count = self.block(parent, span.clone())?.names.len();
        for source in sources {
            if *source == parent {
                return Err(invalid(span));
            }
            self.align_prefix(builder, parent, *source, count, span.clone())?;
        }
        Ok(count)
    }
}
