// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::symbol_error;
use crate::binding::{
    LegacyFunctionBinding, LegacyFunctionInclusion, LegacyIdentifier, LegacySymbolUsage,
    SymbolBudget, SymbolErrorKind, SymbolResource,
};

pub(super) struct SymbolLedger {
    pub(super) budget: SymbolBudget,
    pub(super) usage: LegacySymbolUsage,
}

impl SymbolLedger {
    pub(super) fn new(budget: SymbolBudget) -> Self {
        Self {
            budget,
            usage: LegacySymbolUsage {
                functions: 0,
                active_functions: 0,
                ordinary_functions: 0,
                developer_functions: 0,
                name_bytes: 0,
                largest_name_bytes: 0,
                largest_indexed_name_bytes: 0,
                steps: 0,
                metadata_bytes: 0,
            },
        }
    }

    pub(super) fn work(&mut self, steps: u64) -> Result<(), SymbolErrorKind> {
        let steps = self
            .usage
            .steps
            .checked_add(steps)
            .ok_or(SymbolErrorKind::SizeOverflow)?;
        symbol_error::check(SymbolResource::Steps, steps, self.budget.max_steps)?;
        self.usage.steps = steps;
        Ok(())
    }

    pub(super) fn function(
        &mut self,
        name_bytes: usize,
        inclusion: LegacyFunctionInclusion,
    ) -> Result<(), SymbolErrorKind> {
        let count = self
            .usage
            .functions
            .checked_add(1)
            .ok_or(SymbolErrorKind::SizeOverflow)?;
        symbol_error::check(SymbolResource::Functions, count, self.budget.max_functions)?;
        symbol_error::check(
            SymbolResource::SingleNameBytes,
            name_bytes as u64,
            self.budget.max_single_name_bytes,
        )?;
        let total = self
            .usage
            .name_bytes
            .checked_add(name_bytes as u64)
            .ok_or(SymbolErrorKind::SizeOverflow)?;
        symbol_error::check(SymbolResource::NameBytes, total, self.budget.max_name_bytes)?;
        let active = self
            .usage
            .active_functions
            .checked_add(u64::from(inclusion == LegacyFunctionInclusion::Active))
            .ok_or(SymbolErrorKind::SizeOverflow)?;
        let records = count
            .checked_mul(size_of::<LegacyFunctionBinding<'_, '_, '_>>() as u64)
            .ok_or(SymbolErrorKind::SizeOverflow)?;
        let index_bytes = size_of::<LegacyIdentifier<'_>>() + size_of::<usize>();
        let metadata = active
            .checked_mul(index_bytes as u64)
            .and_then(|bytes| bytes.checked_add(records))
            .ok_or(SymbolErrorKind::SizeOverflow)?;
        symbol_error::check(
            SymbolResource::MetadataBytes,
            metadata,
            self.budget.max_metadata_bytes,
        )?;
        self.usage.functions = count;
        self.usage.active_functions = active;
        self.usage.name_bytes = total;
        self.usage.metadata_bytes = metadata;
        self.usage.largest_name_bytes = self.usage.largest_name_bytes.max(name_bytes as u64);
        if inclusion == LegacyFunctionInclusion::Active {
            self.usage.largest_indexed_name_bytes =
                self.usage.largest_indexed_name_bytes.max(name_bytes as u64);
        }
        Ok(())
    }
}
