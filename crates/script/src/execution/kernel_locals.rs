// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::{
    KernelError, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelResource,
    KernelValue,
};

pub(super) struct KernelLocalAdmission {
    count: usize,
    bytes: u64,
}

impl KernelLocalAdmission {
    pub(super) fn admit(
        layout: KernelFunctionLayout,
        arguments: &[KernelValue],
        budget: KernelExecutionBudget,
    ) -> Result<Self, KernelError> {
        if arguments.len() != layout.parameters {
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
            bytes,
        })
    }

    pub(super) fn count(&self) -> u64 {
        self.count as u64
    }

    pub(super) fn allocate(self, arguments: &[KernelValue]) -> Result<KernelLocals, KernelError> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(self.count)
            .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))?;
        values.resize(self.count, None);
        for (index, value) in arguments.iter().copied().enumerate() {
            let slot = values
                .get_mut(index)
                .ok_or_else(|| kernel_error::general(KernelErrorKind::InvalidExecutionState))?;
            *slot = Some(value);
        }
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
