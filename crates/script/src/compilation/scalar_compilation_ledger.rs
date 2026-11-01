// SPDX-License-Identifier: AGPL-3.0-or-later
use std::mem;
use std::ops::Range;

use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_task::{PendingScalarInstruction, ScalarTask};
use crate::compilation::{
    ScalarCompilationBudget, ScalarCompilationResource, ScalarCompilationUsage, ScalarCompileError,
    ScalarCompileErrorKind,
};
use crate::execution::KernelInstruction;

pub(super) struct ScalarCompilationLedger {
    pub(super) budget: ScalarCompilationBudget,
    pub(super) usage: ScalarCompilationUsage,
    binding_bytes: u64,
    control_bytes: u64,
}

impl ScalarCompilationLedger {
    pub(super) fn new(budget: ScalarCompilationBudget) -> Self {
        Self {
            budget,
            binding_bytes: 0,
            control_bytes: 0,
            usage: ScalarCompilationUsage {
                compiler_steps: 0,
                instructions: 0,
                peak_task_frames: 0,
                peak_metadata_bytes: 0,
                verification_steps: 0,
                validation_steps: None,
                local_bindings: 0,
                identifier_bytes: 0,
                peak_control_bytes: 0,
            },
        }
    }

    pub(super) fn identifier(
        &mut self,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        let next = self
            .usage
            .identifier_bytes
            .checked_add(bytes)
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        check(
            ScalarCompilationResource::IdentifierBytes,
            next,
            self.budget.max_identifier_bytes,
            span.clone(),
        )?;
        self.charge(bytes, span)?;
        self.usage.identifier_bytes = next;
        Ok(())
    }

    pub(super) fn bindings(
        &mut self,
        count: u64,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        check(
            ScalarCompilationResource::LocalBindings,
            count,
            self.budget.max_local_bindings,
            span,
        )?;
        self.usage.local_bindings = count;
        self.binding_bytes = bytes;
        Ok(())
    }

    pub(super) fn charge(
        &mut self,
        amount: u64,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        let next = self
            .usage
            .compiler_steps
            .checked_add(amount)
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        check(
            ScalarCompilationResource::Steps,
            next,
            self.budget.max_steps,
            span,
        )?;
        self.usage.compiler_steps = next;
        Ok(())
    }

    pub(super) fn control(
        &mut self,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        let next = self.control_bytes.checked_add(bytes).ok_or_else(|| {
            scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
        })?;
        check(
            ScalarCompilationResource::ControlBytes,
            next,
            self.budget.max_control_bytes,
            span,
        )?;
        self.control_bytes = next;
        self.usage.peak_control_bytes = self.usage.peak_control_bytes.max(next);
        Ok(())
    }

    pub(super) fn release_control(
        &mut self,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        self.control_bytes = self.control_bytes.checked_sub(bytes).ok_or_else(|| {
            scalar_compile_error::at(span, ScalarCompileErrorKind::InvalidControlState)
        })?;
        Ok(())
    }

    pub(super) fn admit(
        &mut self,
        instructions: usize,
        tasks: usize,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        check(
            ScalarCompilationResource::Instructions,
            instructions as u64,
            self.budget.max_instructions,
            span.clone(),
        )?;
        check(
            ScalarCompilationResource::TaskFrames,
            tasks as u64,
            self.budget.max_task_frames,
            span.clone(),
        )?;
        let code =
            (instructions as u64).checked_mul(mem::size_of::<PendingScalarInstruction>() as u64);
        let frames = (tasks as u64).checked_mul(mem::size_of::<ScalarTask<'_, '_>>() as u64);
        let bytes = code
            .and_then(|code| frames.and_then(|frames| code.checked_add(frames)))
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        self.metadata_with_bindings(bytes, span)?;
        self.usage.instructions = instructions as u64;
        self.usage.peak_task_frames = self.usage.peak_task_frames.max(tasks as u64);
        Ok(())
    }

    pub(super) fn resolution(&mut self, span: Range<u64>) -> Result<(), ScalarCompileError> {
        let slot = mem::size_of::<PendingScalarInstruction>()
            .checked_add(mem::size_of::<KernelInstruction>())
            .and_then(|size| size.checked_add(mem::size_of::<Range<u64>>()))
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        let bytes = self
            .usage
            .instructions
            .checked_mul(slot as u64)
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        self.metadata_with_bindings(bytes, span)
    }

    fn metadata_with_bindings(
        &mut self,
        bytes: u64,
        span: Range<u64>,
    ) -> Result<(), ScalarCompileError> {
        let bytes = bytes
            .checked_add(self.binding_bytes)
            .and_then(|bytes| bytes.checked_add(self.control_bytes))
            .ok_or_else(|| {
                scalar_compile_error::at(span.clone(), ScalarCompileErrorKind::SizeOverflow)
            })?;
        self.metadata(bytes, span)
    }

    fn metadata(&mut self, bytes: u64, span: Range<u64>) -> Result<(), ScalarCompileError> {
        check(
            ScalarCompilationResource::MetadataBytes,
            bytes,
            self.budget.max_metadata_bytes,
            span,
        )?;
        self.usage.peak_metadata_bytes = self.usage.peak_metadata_bytes.max(bytes);
        Ok(())
    }
}

fn check(
    resource: ScalarCompilationResource,
    requested: u64,
    maximum: u64,
    span: Range<u64>,
) -> Result<(), ScalarCompileError> {
    if requested > maximum {
        return Err(scalar_compile_error::at(
            span,
            ScalarCompileErrorKind::BudgetExceeded {
                resource,
                requested,
                maximum,
            },
        ));
    }
    Ok(())
}
