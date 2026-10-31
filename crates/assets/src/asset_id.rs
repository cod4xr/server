// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::AssetError;

// This counter only mints catalog namespaces. It holds no asset/game state and
// prevents a handle from accidentally resolving in a newly created catalog.
static NEXT_CATALOG: AtomicU64 = AtomicU64::new(1);

/// Opaque engine handle checked against its catalog, slot, and generation.
///
/// It has no wire representation. A peer-specific mapper must explicitly choose
/// a supported serialized namespace instead of narrowing this handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AssetId {
    pub(super) catalog: u64,
    pub(super) slot: usize,
    pub(super) generation: u64,
}

impl fmt::Display for AssetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}:{}",
            self.catalog, self.slot, self.generation
        )
    }
}

pub(super) fn allocate_catalog() -> Result<u64, AssetError> {
    allocate(&NEXT_CATALOG)
}

fn allocate(counter: &AtomicU64) -> Result<u64, AssetError> {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| AssetError::IdentityExhausted)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use super::allocate;
    use crate::AssetError;

    #[test]
    fn catalog_namespace_never_wraps() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(allocate(&counter), Ok(u64::MAX - 1));
        assert_eq!(allocate(&counter), Err(AssetError::IdentityExhausted));
        assert_eq!(allocate(&counter), Err(AssetError::IdentityExhausted));
    }
}
