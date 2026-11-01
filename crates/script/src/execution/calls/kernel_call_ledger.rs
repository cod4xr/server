// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::calls::kernel_call_frame::KernelCallFrame;
use crate::execution::calls::{KernelCallBudget, KernelCallUsage, KernelModuleError};
use crate::execution::kernel_error;
use crate::execution::{
    KernelError, KernelErrorKind, KernelExecutionBudget, KernelResource, KernelValue,
    VerifiedKernelProgram,
};

#[derive(Clone, Copy, Default)]
struct LivePayload {
    frames: u64,
    stack_values: u64,
    local_values: u64,
}

pub(super) struct KernelCallLedger {
    pub(super) usage: KernelCallUsage,
    budget: KernelCallBudget,
    live: LivePayload,
}

impl KernelCallLedger {
    pub(super) fn new(budget: KernelCallBudget) -> Self {
        Self {
            budget,
            usage: KernelCallUsage::default(),
            live: LivePayload::default(),
        }
    }

    pub(super) fn admit(
        &mut self,
        proof: &VerifiedKernelProgram,
        arguments: usize,
    ) -> Result<KernelExecutionBudget, KernelError> {
        let next = LivePayload {
            frames: add(self.live.frames, 1)?,
            stack_values: add(self.live.stack_values, proof.usage.max_stack_values)?,
            local_values: add(self.live.local_values, proof.layout.local_values())?,
        };
        let frame_bytes = bytes(next.frames, size_of::<KernelCallFrame>())?;
        let stack_bytes = bytes(next.stack_values, size_of::<KernelValue>())?;
        let local_bytes = bytes(next.local_values, size_of::<Option<KernelValue>>())?;
        let setup = add(
            add(
                add(add(self.usage.setup_work, 1)?, proof.layout.local_values())?,
                arguments as u64,
            )?,
            self.live.frames,
        )?;
        for (resource, requested, maximum) in [
            (KernelResource::Frames, next.frames, self.budget.max_frames),
            (
                KernelResource::ExecutionFrameBytes,
                frame_bytes,
                self.budget.max_frame_bytes,
            ),
            (
                KernelResource::StackValues,
                next.stack_values,
                self.budget.max_stack_values,
            ),
            (
                KernelResource::ExecutionStackBytes,
                stack_bytes,
                self.budget.max_stack_bytes,
            ),
            (
                KernelResource::LocalValues,
                next.local_values,
                self.budget.max_local_values,
            ),
            (
                KernelResource::ExecutionLocalBytes,
                local_bytes,
                self.budget.max_local_bytes,
            ),
            (
                KernelResource::CallSetupWork,
                setup,
                self.budget.max_setup_work,
            ),
        ] {
            kernel_error::check(resource, requested, maximum, None)?;
        }
        self.live = next;
        self.usage.setup_work = setup;
        self.usage.peak_frames = self.usage.peak_frames.max(next.frames);
        self.usage.peak_frame_bytes = self.usage.peak_frame_bytes.max(frame_bytes);
        self.usage.peak_stack_values = self.usage.peak_stack_values.max(next.stack_values);
        self.usage.peak_stack_bytes = self.usage.peak_stack_bytes.max(stack_bytes);
        self.usage.peak_local_values = self.usage.peak_local_values.max(next.local_values);
        self.usage.peak_local_bytes = self.usage.peak_local_bytes.max(local_bytes);
        Ok(KernelExecutionBudget {
            max_steps: self.budget.max_steps,
            max_stack_values: proof.usage.max_stack_values,
            max_stack_bytes: bytes(proof.usage.max_stack_values, size_of::<KernelValue>())?,
            max_local_values: proof.layout.local_values(),
            max_local_bytes: bytes(
                proof.layout.local_values(),
                size_of::<Option<KernelValue>>(),
            )?,
        })
    }

    pub(super) fn release(&mut self, proof: &VerifiedKernelProgram) -> Result<(), KernelError> {
        self.live = LivePayload {
            frames: subtract(self.live.frames, 1)?,
            stack_values: subtract(self.live.stack_values, proof.usage.max_stack_values)?,
            local_values: subtract(self.live.local_values, proof.layout.local_values())?,
        };
        Ok(())
    }

    pub(super) fn error(
        &self,
        function: Option<usize>,
        pc: Option<usize>,
        mut cause: KernelError,
    ) -> KernelModuleError {
        if cause.instruction.is_none() {
            cause.instruction = pc.map(|pc| pc as u64);
        }
        cause.work_steps = self.usage.steps;
        KernelModuleError {
            function: function.map(|function| function as u64),
            cause,
            setup_work: self.usage.setup_work,
            peak_frames: self.usage.peak_frames,
            peak_reserved_stack_values: self.usage.peak_stack_values,
            peak_local_values: self.usage.peak_local_values,
        }
    }
}

fn bytes(count: u64, size: usize) -> Result<u64, KernelError> {
    count
        .checked_mul(size as u64)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))
}

fn add(left: u64, right: u64) -> Result<u64, KernelError> {
    left.checked_add(right)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::SizeOverflow))
}

fn subtract(left: u64, right: u64) -> Result<u64, KernelError> {
    left.checked_sub(right)
        .ok_or_else(|| kernel_error::general(KernelErrorKind::InvalidExecutionState))
}
