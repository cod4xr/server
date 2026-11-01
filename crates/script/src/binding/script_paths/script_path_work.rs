// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::script_paths::{
    ScriptPathBudget, ScriptPathError, ScriptPathErrorKind, ScriptPathResource, ScriptPathUsage,
};

pub(super) struct ScriptPathWork {
    budget: ScriptPathBudget,
    pub(super) usage: ScriptPathUsage,
}

impl ScriptPathWork {
    pub(super) fn new(input: usize, budget: ScriptPathBudget) -> Result<Self, ScriptPathError> {
        let result = Self {
            budget,
            usage: ScriptPathUsage {
                input_bytes: input as u64,
                ..ScriptPathUsage::default()
            },
        };
        result.check(
            ScriptPathResource::InputBytes,
            input as u64,
            budget.max_input_bytes,
        )?;
        Ok(result)
    }

    pub(super) fn charge(&mut self, count: u64) -> Result<(), ScriptPathError> {
        let next = self
            .usage
            .work_steps
            .checked_add(count)
            .ok_or_else(|| self.error(ScriptPathErrorKind::SizeOverflow))?;
        self.check(ScriptPathResource::WorkSteps, next, self.budget.max_steps)?;
        self.usage.work_steps = next;
        Ok(())
    }

    pub(super) fn canonical(&mut self, count: usize) -> Result<(), ScriptPathError> {
        self.check(
            ScriptPathResource::CanonicalBytes,
            count as u64,
            self.budget.max_canonical_bytes,
        )?;
        self.usage.canonical_bytes = count as u64;
        Ok(())
    }

    pub(super) fn error(&self, kind: ScriptPathErrorKind) -> ScriptPathError {
        ScriptPathError {
            kind,
            usage: self.usage,
        }
    }

    fn check(
        &self,
        resource: ScriptPathResource,
        requested: u64,
        maximum: u64,
    ) -> Result<(), ScriptPathError> {
        if requested > maximum {
            return Err(self.error(ScriptPathErrorKind::BudgetExceeded {
                resource,
                requested,
                maximum,
            }));
        }
        Ok(())
    }
}
