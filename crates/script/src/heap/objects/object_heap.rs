// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::KernelValue;
use crate::heap::heap_identity::HeapIdentity;
use crate::heap::objects::field_index::FieldIndex;
use crate::heap::objects::object_slot::{FieldSlot, ObjectSlot, ObjectState};
use crate::heap::objects::{
    ObjectAllocation, ObjectCollection, ObjectFieldResult, ObjectHeapBudget, ObjectHeapError,
    ObjectHeapErrorKind, ObjectHeapUsage, ObjectId, ObjectStringCollection,
};
use crate::heap::objects::{object_allocation, object_fields, object_reclamation, object_strings};
use crate::heap::{HeapWorkBudget, HeapWorkUsage, StringHeap};

/// Prototype shared struct graph with checked nonowning identities and explicit roots.
///
/// Copies preserve object identity. No array COW, VM/source execution, native GC or
/// game lifetime policy is implied. Supply every live object root to reclamation.
/// The associated string store remains separately owned; use `reclaim_strings` with
/// every additional string root, after object collection when cycles should release
/// strings. Logical quotas exclude allocator/spare capacity/fixed fields and RSS.
pub struct ObjectHeap {
    pub(super) owner: HeapIdentity,
    pub(super) string_owner: HeapIdentity,
    pub(super) budget: ObjectHeapBudget,
    pub(super) objects: Vec<ObjectSlot>,
    pub(super) fields: Vec<FieldSlot>,
    pub(super) free_object: Option<usize>,
    pub(super) free_field: Option<usize>,
    pub(super) index: FieldIndex,
    pub(super) usage: ObjectHeapUsage,
}

impl ObjectHeap {
    /// Creates empty metadata without allocating arenas or adopting string ownership.
    /// # Errors
    /// Rejects identity exhaustion instead of wrapping/reusing a namespace.
    pub fn new(strings: &StringHeap, budget: ObjectHeapBudget) -> Result<Self, ObjectHeapError> {
        let owner = HeapIdentity::allocate().map_err(|_| ObjectHeapError {
            kind: ObjectHeapErrorKind::IdentityExhausted,
            usage: HeapWorkUsage::default(),
        })?;
        Ok(Self {
            owner,
            string_owner: strings.owner,
            budget,
            objects: Vec::new(),
            fields: Vec::new(),
            free_object: None,
            free_field: None,
            index: FieldIndex::default(),
            usage: ObjectHeapUsage::default(),
        })
    }

    /// Returns logical live/retained usage without keys, values or identities.
    #[must_use]
    pub fn usage(&self) -> ObjectHeapUsage {
        self.usage
    }

    /// Returns configured resource admission without changing storage.
    #[must_use]
    pub fn budget(&self) -> ObjectHeapBudget {
        self.budget
    }

    /// Allocates one empty shared object after complete resource admission.
    /// # Errors
    /// Invalid private state, overflow, exhaustion or allocation failure publishes
    /// no object. Spare arena capacity may grow outside logical counters.
    pub fn allocate(
        &mut self,
        budget: HeapWorkBudget,
    ) -> Result<ObjectAllocation, ObjectHeapError> {
        object_allocation::allocate(self, budget)
    }

    /// Returns stored field count, distinct from the legacy script .size operation.
    /// # Errors
    /// Rejects foreign/stale/vacant/retired references without mutation.
    pub fn field_count(&self, id: ObjectId) -> Result<u64, ObjectHeapErrorKind> {
        let index = self.resolve(id)?;
        match self.objects[index].state {
            ObjectState::Occupied { fields, .. } => Ok(fields as u64),
            _ => Err(ObjectHeapErrorKind::InvalidState),
        }
    }

    /// Copies a checked field value; absence differs from stored Undefined.
    /// # Errors
    /// Rejects invalid object/string references, key/work limits and private state.
    pub fn get(
        &self,
        id: ObjectId,
        key: &[u8],
        strings: &StringHeap,
        budget: HeapWorkBudget,
    ) -> Result<ObjectFieldResult, ObjectHeapError> {
        object_fields::get(self, id, key, strings, budget)
    }

    /// Inserts/replaces an exact owned key and copied value, preserving identity.
    /// Object/string references are checked; opaque functions grant no module lifetime.
    /// # Errors
    /// Invalid references, limits, overflow/allocation failures leave fields/usage
    /// unchanged. Keys are not normalized; script field rules belong above storage.
    pub fn put(
        &mut self,
        id: ObjectId,
        key: &[u8],
        value: KernelValue,
        strings: &StringHeap,
        budget: HeapWorkBudget,
    ) -> Result<ObjectFieldResult, ObjectHeapError> {
        object_fields::put(self, id, key, value, strings, budget)
    }

    /// Removes one exact field and returns a nonowning copied previous value.
    /// # Errors
    /// Invalid references/limits leave fields/usage unchanged. A returned object
    /// must be rooted before later collection; removal is not an ownership lease.
    pub fn remove(
        &mut self,
        id: ObjectId,
        key: &[u8],
        strings: &StringHeap,
        budget: HeapWorkBudget,
    ) -> Result<ObjectFieldResult, ObjectHeapError> {
        object_fields::remove(self, id, key, strings, budget)
    }

    /// Iteratively traces the complete supplied object roots, reclaiming unreachable cycles.
    /// # Errors
    /// All validation/admission/allocation preflights before deletion. Failures
    /// preserve values, identities and usage, retaining charged work/scratch.
    pub fn reclaim(
        &mut self,
        roots: &[ObjectId],
        budget: HeapWorkBudget,
    ) -> Result<ObjectCollection, ObjectHeapError> {
        object_reclamation::reclaim(self, roots, budget)
    }

    /// Reclaims strings using every stored string field plus complete extra roots.
    /// # Errors
    /// Rejects wrong/stale stores/roots or exhausted work/scratch before deleting
    /// strings. This does not collect objects; unreachable object fields retain strings.
    pub fn reclaim_strings(
        &self,
        strings: &mut StringHeap,
        extra: &[crate::heap::StringId],
        budget: HeapWorkBudget,
    ) -> Result<ObjectStringCollection, ObjectHeapError> {
        object_strings::reclaim(self, strings, extra, budget)
    }

    pub(super) fn resolve(&self, id: ObjectId) -> Result<usize, ObjectHeapErrorKind> {
        if id.owner != self.owner {
            return Err(ObjectHeapErrorKind::ForeignReference);
        }
        let slot = self
            .objects
            .get(id.slot)
            .ok_or(ObjectHeapErrorKind::StaleReference)?;
        if slot.generation != id.generation || !matches!(slot.state, ObjectState::Occupied { .. }) {
            return Err(ObjectHeapErrorKind::StaleReference);
        }
        Ok(id.slot)
    }

    pub(crate) fn validate_strings(&self, strings: &StringHeap) -> Result<(), ObjectHeapErrorKind> {
        if self.string_owner != strings.owner {
            return Err(ObjectHeapErrorKind::ForeignStringHeap);
        }
        Ok(())
    }
}

impl fmt::Debug for ObjectHeap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ObjectHeap")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
