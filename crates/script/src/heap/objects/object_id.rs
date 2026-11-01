// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::heap::heap_identity::HeapIdentity;

/// Nonowning checked identity in one shared-object store, never a native address.
/// Copies do not root an object; explicit complete-root reclamation controls lifetime.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ObjectId {
    pub(super) owner: HeapIdentity,
    pub(super) slot: usize,
    pub(super) generation: u64,
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ObjectId(..)")
    }
}
