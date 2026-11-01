// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::{
    KernelError, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelResource,
    KernelValue,
};

pub(super) struct KernelLocalAdmission {
    count: usize,
    parameters: usize,
    bytes: u64,
}

#[derive(Clone, Copy)]
pub(super) enum KernelArgumentBinding {
    Exact,
    LegacyScript,
}

impl KernelLocalAdmission {
    pub(super) fn admit(
        layout: KernelFunctionLayout,
        arguments: &[KernelValue],
        budget: KernelExecutionBudget,
        binding: KernelArgumentBinding,
    ) -> Result<Self, KernelError> {
        let admitted = match binding {
            KernelArgumentBinding::Exact => arguments.len() == layout.parameters,
            KernelArgumentBinding::LegacyScript => arguments.len() <= layout.parameters,
        };
        if !admitted {
            return Err(kernel_error::general(KernelErrorKind::ArgumentCount {
                expected: layout.parameters(),
                actual: arguments.len() as u64,
            }));
        }
        kernel_error::check(
            KernelResource::LocalValues,
            layout.local_values(),
            budget.max_local_values,
            None,
        )?;
        let bytes = layout
            .local_values()
            .checked_mul(size_of::<Option<KernelValue>>() as u64)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
        kernel_error::check(
            KernelResource::ExecutionLocalBytes,
            bytes,
            budget.max_local_bytes,
            None,
        )?;
        Ok(Self {
            count: layout.local_values,
            parameters: layout.parameters,
            bytes,
        })
    }

    pub(super) fn count(&self) -> u64 {
        self.count as u64
    }

    pub(super) fn allocate(self, arguments: &[KernelValue]) -> Result<KernelLocals, KernelError> {
        if arguments.len() > self.parameters || self.parameters > self.count {
            return Err(kernel_error::general(
                KernelErrorKind::InvalidExecutionState,
            ));
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(self.count)
            .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))?;
        // The admitted prefix and total count bound all three phases. There is
        // no padded temporary argument vector; each reserved slot is filled once.
        values.extend(arguments.iter().copied().map(Some));
        values.resize(self.parameters, Some(KernelValue::Undefined));
        values.resize(self.count, None);
        Ok(KernelLocals {
            values,
            bytes: self.bytes,
        })
    }
}

pub(super) struct KernelLocals {
    values: Vec<Option<KernelValue>>,
    bytes: u64,
}

impl KernelLocals {
    pub(super) fn values(&self) -> impl Iterator<Item = KernelValue> + Clone {
        self.values
            .iter()
            .map(|value| value.unwrap_or(KernelValue::Undefined))
    }

    pub(super) fn load(&self, slot: usize, pc: usize) -> Result<KernelValue, KernelError> {
        self.values
            .get(slot)
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?
            .ok_or_else(|| {
                kernel_error::at(
                    pc,
                    KernelErrorKind::UninitializedLocal { slot: slot as u64 },
                )
            })
    }

    pub(super) fn store(
        &mut self,
        slot: usize,
        value: KernelValue,
        pc: usize,
    ) -> Result<(), KernelError> {
        let entry = self
            .values
            .get_mut(slot)
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?;
        *entry = Some(value);
        Ok(())
    }

    pub(super) fn count(&self) -> u64 {
        self.values.len() as u64
    }
    pub(super) fn bytes(&self) -> u64 {
        self.bytes
    }
}
