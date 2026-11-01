// SPDX-License-Identifier: AGPL-3.0-or-later
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::heap::HeapErrorKind;

// Store identity minting only; no bytes, values, gameplay or publication state.
static LAST_HEAP_IDENTITY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) struct HeapIdentity(NonZeroU64);

impl HeapIdentity {
    pub(super) fn allocate() -> Result<Self, HeapErrorKind> {
        allocate(&LAST_HEAP_IDENTITY)
    }
}

fn allocate(counter: &AtomicU64) -> Result<HeapIdentity, HeapErrorKind> {
    let previous = counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |last| {
            last.checked_add(1)
        })
        .map_err(|_| HeapErrorKind::IdentityExhausted)?;
    previous
        .checked_add(1)
        .and_then(NonZeroU64::new)
        .map(HeapIdentity)
        .ok_or(HeapErrorKind::IdentityExhausted)
}

#[cfg(test)]
mod tests {
    use crate::heap::HeapErrorKind;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn full_nonzero_heap_identity_domain_never_wraps() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(
            super::allocate(&counter).map(|value| value.0.get()),
            Ok(u64::MAX)
        );
        assert_eq!(
            super::allocate(&counter).err(),
            Some(HeapErrorKind::IdentityExhausted)
        );
        assert_eq!(
            super::allocate(&counter).err(),
            Some(HeapErrorKind::IdentityExhausted)
        );
    }
}
