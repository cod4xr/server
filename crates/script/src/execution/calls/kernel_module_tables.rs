// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::kernel_literals;
use crate::execution::{
    KernelError, KernelErrorKind, KernelFunctionDefinition, KernelInstruction, KernelProgramUsage,
};

#[derive(Default)]
pub(super) struct KernelTableUsage {
    pub(super) count: u64,
    pub(super) bytes: u64,
}

#[derive(Default)]
pub(super) struct KernelModuleTables {
    pub(super) strings: KernelTableUsage,
    pub(super) fields: KernelTableUsage,
    pub(super) labels: KernelTableUsage,
}

impl KernelTableUsage {
    fn record(&mut self, rows: &[Vec<u8>]) -> Result<(), KernelError> {
        self.count = add(self.count, rows.len() as u64)?;
        self.bytes = add(self.bytes, kernel_literals::retained_bytes(rows)?)?;
        Ok(())
    }
}

impl KernelModuleTables {
    pub(super) fn visits(definition: KernelFunctionDefinition<'_>) -> Result<u64, KernelError> {
        add(
            add(
                definition.string_literals.len() as u64,
                definition.field_names.len() as u64,
            )?,
            definition.switch_labels.len() as u64,
        )?
        .checked_mul(2)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))
    }

    pub(super) fn record(
        &mut self,
        definition: KernelFunctionDefinition<'_>,
    ) -> Result<(), KernelError> {
        self.strings.record(definition.string_literals)?;
        self.fields.record(definition.field_names)?;
        self.labels.record(definition.switch_labels)
    }

    pub(super) fn bytes(&self) -> Result<u64, KernelError> {
        add(
            add(self.strings.bytes, self.fields.bytes)?,
            self.labels.bytes,
        )
    }
}

pub(super) fn definition_payload(
    definition: KernelFunctionDefinition<'_>,
) -> Result<u64, KernelError> {
    let code = (definition.instructions.len() as u64)
        .checked_mul(size_of::<KernelInstruction>() as u64)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))?;
    add(
        add(
            code,
            kernel_literals::retained_bytes(definition.string_literals)?,
        )?,
        add(
            kernel_literals::retained_bytes(definition.field_names)?,
            kernel_literals::retained_bytes(definition.switch_labels)?,
        )?,
    )
}

pub(super) fn program_payload(usage: KernelProgramUsage) -> Result<u64, KernelError> {
    add(
        add(usage.retained_code_bytes, usage.retained_string_bytes)?,
        add(usage.retained_field_bytes, usage.retained_switch_bytes)?,
    )
}

fn add(left: u64, right: u64) -> Result<u64, KernelError> {
    left.checked_add(right)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))
}
