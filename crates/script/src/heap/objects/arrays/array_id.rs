// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::heap::ObjectId;
use crate::heap::heap_identity::HeapIdentity;

/// Nonowning array-data snapshot; copies do not create managed value owners.
/// Data remains valid only until explicit collection reclaims its generation.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ArrayId {
    pub(in crate::heap::objects) object: ObjectId,
}

/// Checked handle to one explicitly managed array value location.
///
/// Copying this handle does not allocate a second location or owner. The location
/// owns its data until explicitly released; creating an alias requires the heap's
/// copy operation. Released/reused/foreign locations reject stale handles.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ArrayValueId {
    pub(in crate::heap::objects) owner: HeapIdentity,
    pub(in crate::heap::objects) slot: usize,
    pub(in crate::heap::objects) generation: u64,
}

impl ArrayValueId {
    /// Hashes a location within one already validated arena. Full equality still
    /// checks arena and generation; no identity or address leaves the crate.
    pub(crate) fn hash_location(&self, state: &mut impl Hasher) {
        self.slot.hash(state);
    }
}

impl fmt::Debug for ArrayId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ArrayId(..)")
    }
}

impl fmt::Debug for ArrayValueId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ArrayValueId(..)")
    }
}
