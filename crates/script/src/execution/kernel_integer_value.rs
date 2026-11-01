// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelErrorKind, KernelValue};
use crate::legacy::LegacyNumber;

pub(super) fn require(value: KernelValue) -> Result<KernelValue, KernelErrorKind> {
    match value {
        KernelValue::Number(LegacyNumber::Integer(_)) => Ok(value),
        _ => Err(KernelErrorKind::ExpectedInteger),
    }
}
