// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::kernel_error;
use crate::execution::{KernelError, KernelErrorKind};

pub(super) struct KernelFlowState {
    heights: Vec<Option<usize>>,
    pending: Vec<usize>,
}

impl KernelFlowState {
    pub(super) fn new(count: usize) -> Result<Self, KernelError> {
        let mut heights = Vec::new();
        heights
            .try_reserve_exact(count)
            .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))?;
        heights.resize(count, None);
        let mut pending = Vec::new();
        pending
            .try_reserve_exact(count)
            .map_err(|_| kernel_error::general(KernelErrorKind::AllocationFailed))?;
        *heights
            .get_mut(0)
            .ok_or_else(|| kernel_error::general(KernelErrorKind::EmptyProgram))? = Some(0);
        pending.push(0);
        Ok(Self { heights, pending })
    }

    pub(super) fn next_pending(&mut self) -> Result<Option<(usize, usize)>, KernelError> {
        let Some(pc) = self.pending.pop() else {
            return Ok(None);
        };
        let height = self
            .heights
            .get(pc)
            .copied()
            .flatten()
            .ok_or_else(|| kernel_error::at(pc, KernelErrorKind::InvalidExecutionState))?;
        Ok(Some((pc, height)))
    }

    pub(super) fn bind(
        &mut self,
        from: usize,
        target: usize,
        height: usize,
    ) -> Result<(), KernelError> {
        let slot = self.heights.get_mut(target).ok_or_else(|| {
            kernel_error::at(
                from,
                KernelErrorKind::InvalidTarget {
                    target: target as u64,
                },
            )
        })?;
        match *slot {
            Some(expected) if expected != height => {
                return Err(kernel_error::at(
                    from,
                    KernelErrorKind::StackJoin {
                        target: target as u64,
                        expected: expected as u64,
                        actual: height as u64,
                    },
                ));
            }
            Some(_) => {}
            None => {
                *slot = Some(height);
                // A destination is queued once. All count slots were admitted
                // before reserve, so user code cannot grow this vector past count.
                self.pending.push(target);
            }
        }
        Ok(())
    }
}
