// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;

pub(in crate::execution) enum KernelHeapStep {
    Unhandled,
    Applied(Option<KernelValue>),
}
