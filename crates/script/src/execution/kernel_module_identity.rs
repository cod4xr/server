// SPDX-License-Identifier: AGPL-3.0-or-later
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::execution::KernelErrorKind;

// Identity minting only: no game state, publication synchronization or module data.
static LAST_MODULE_IDENTITY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) struct KernelModuleIdentity(NonZeroU64);

impl KernelModuleIdentity {
    pub(super) fn allocate() -> Result<Self, KernelErrorKind> {
        allocate(&LAST_MODULE_IDENTITY)
    }
}

fn allocate(counter: &AtomicU64) -> Result<KernelModuleIdentity, KernelErrorKind> {
    let previous = counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |last| {
            last.checked_add(1)
        })
        .map_err(|_| KernelErrorKind::ModuleIdentityExhausted)?;
    previous
        .checked_add(1)
        .and_then(NonZeroU64::new)
        .map(KernelModuleIdentity)
        .ok_or(KernelErrorKind::ModuleIdentityExhausted)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use super::allocate;
    use crate::execution::KernelErrorKind;

    #[test]
    fn identities_use_the_full_nonzero_domain_and_never_wrap() {
        let counter = AtomicU64::new(0);
        assert_eq!(allocate(&counter).map(|identity| identity.0.get()), Ok(1));
        assert_eq!(allocate(&counter).map(|identity| identity.0.get()), Ok(2));
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(
            allocate(&counter).map(|identity| identity.0.get()),
            Ok(u64::MAX)
        );
        assert_eq!(
            allocate(&counter).err(),
            Some(KernelErrorKind::ModuleIdentityExhausted)
        );
        assert_eq!(
            allocate(&counter).err(),
            Some(KernelErrorKind::ModuleIdentityExhausted)
        );
    }
}
