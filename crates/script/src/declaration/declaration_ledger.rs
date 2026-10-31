// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::declaration::declaration_error;
use crate::declaration::{
    DeclarationBudget, DeclarationErrorKind, DeclarationResource, LegacyModuleUsage,
};

pub(super) struct DeclarationLedger {
    pub(super) budget: DeclarationBudget,
    pub(super) usage: LegacyModuleUsage,
}

impl DeclarationLedger {
    pub(super) fn new(budget: DeclarationBudget) -> Self {
        Self {
            budget,
            usage: LegacyModuleUsage {
                items: 0,
                includes: 0,
                parameters: 0,
                steps: 0,
                metadata_bytes: 0,
            },
        }
    }

    pub(super) fn visit(&mut self) -> Result<(), DeclarationErrorKind> {
        self.usage.steps = self
            .usage
            .steps
            .checked_add(1)
            .ok_or(DeclarationErrorKind::SizeOverflow)?;
        declaration_error::check(
            DeclarationResource::Steps,
            self.usage.steps,
            self.budget.max_steps,
        )
    }

    pub(super) fn admit(
        &mut self,
        resource: DeclarationResource,
        bytes: usize,
    ) -> Result<(), DeclarationErrorKind> {
        let (count, maximum) = match resource {
            DeclarationResource::Items => (&mut self.usage.items, self.budget.max_items),
            DeclarationResource::Includes => (&mut self.usage.includes, self.budget.max_includes),
            DeclarationResource::Parameters => {
                (&mut self.usage.parameters, self.budget.max_parameters)
            }
            _ => return Err(DeclarationErrorKind::InvalidStructure),
        };
        let next = count
            .checked_add(1)
            .ok_or(DeclarationErrorKind::SizeOverflow)?;
        declaration_error::check(resource, next, maximum)?;
        let size = self
            .usage
            .metadata_bytes
            .checked_add(bytes as u64)
            .ok_or(DeclarationErrorKind::SizeOverflow)?;
        declaration_error::check(
            DeclarationResource::MetadataBytes,
            size,
            self.budget.max_metadata_bytes,
        )?;
        *count = next;
        self.usage.metadata_bytes = size;
        Ok(())
    }
}
