// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::heap::heap_identity::HeapIdentity;

/// Opaque owner/slot/generation reference, with no native/serialized representation.
///
/// Copying does not keep bytes or a store alive. Reclamation requires a complete
/// root set; subsequent resolution rejects missing, stale and foreign references.
/// Derived equality compares references, never GSC string byte contents.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct StringId {
    pub(super) owner: HeapIdentity,
    pub(super) slot: usize,
    pub(super) generation: u64,
}

impl fmt::Debug for StringId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("StringId(..)")
    }
}
