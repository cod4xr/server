// SPDX-License-Identifier: AGPL-3.0-or-later
use std::mem;

use crate::execution::kernel_error;
use crate::execution::{
    KernelError, KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue,
};
use crate::legacy::LegacyNumber;

pub(super) struct KernelStack {
    values: Vec<KernelValue>,
    limit: usize,
    peak: usize,
}

impl KernelStack {
    pub(super) fn new(maximum: u64, budget: KernelExecutionBudget) -> Result<Self, KernelError> {
        kernel_error::check(
            KernelResource::StackValues,
            maximum,
            budget.max_stack_values,
            None,
        )?;
        let bytes = maximum
            .checked_mul(mem::size_of::<KernelValue>() as u64)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        kernel_error::check(
            KernelResource::ExecutionStackBytes,
            bytes,
            budget.max_stack_bytes,
            None,
        )?;
        let limit = usize::try_from(maximum)
            .map_err(|_| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(limit)
            .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))?;
        Ok(Self {
            values,
            limit,
            peak: 0,
        })
    }

    pub(super) fn push(&mut self, value: KernelValue, pc: usize) -> Result<(), KernelError> {
        if self.values.len() >= self.limit {
            return Err(kernel_error::at(pc, KernelErrorKind::InvalidExecutionState));
        }
        self.values.push(value);
        self.peak = self.peak.max(self.values.len());
        Ok(())
    }

    pub(super) fn pop(&mut self, pc: usize) -> Result<KernelValue, KernelError> {
        self.values
            .pop()
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))
    }

    pub(super) fn number(&mut self, pc: usize) -> Result<LegacyNumber, KernelError> {
        match self.pop(pc)? {
            KernelValue::Number(value) => Ok(value),
            KernelValue::Undefined
            | KernelValue::Vector(_)
            | KernelValue::Function(_)
            | KernelValue::String(_) => Err(kernel_error::at(pc, KernelErrorKind::ExpectedNumber)),
        }
    }

    pub(super) fn pair(&mut self, pc: usize) -> Result<(LegacyNumber, LegacyNumber), KernelError> {
        let right = self.number(pc)?;
        let left = self.number(pc)?;
        Ok((left, right))
    }

    pub(super) fn duplicate(&mut self, pc: usize) -> Result<(), KernelError> {
        let value = self
            .values
            .last()
            .copied()
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?;
        self.push(value, pc)
    }

    pub(super) fn empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(super) fn values(&self) -> impl Iterator<Item = KernelValue> + Clone {
        self.values.iter().copied()
    }

    pub(super) fn binary_values(&self) -> Option<(KernelValue, KernelValue)> {
        let [.., left, right] = self.values.as_slice() else {
            return None;
        };
        Some((*left, *right))
    }

    pub(super) fn peak(&self) -> u64 {
        self.peak as u64
    }
}
