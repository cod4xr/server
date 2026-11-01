// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::heap::heap_identity::HeapIdentity;
use crate::heap::string_allocation::{self, StringInput};
use crate::heap::string_reclamation;
use crate::heap::string_slot::{StringSlot, StringSlotState};
use crate::heap::{
    HeapAllocation, HeapError, HeapErrorKind, HeapWorkBudget, HeapWorkUsage, StringCollection,
    StringHeapBudget, StringHeapUsage, StringId,
};

/// Growable owned immutable byte strings with checked, nonowning wide references.
///
/// Copied IDs are not roots/leases automatically. Callers supply every live root to
/// reclamation; unrooted IDs become stale. No interning, encoding repair, implicit
/// terminators, native pointer ABI, host state or general object collector is added.
pub struct StringHeap {
    pub(super) owner: HeapIdentity,
    pub(super) budget: StringHeapBudget,
    pub(super) slots: Vec<StringSlot>,
    pub(super) free_head: Option<usize>,
    pub(super) usage: StringHeapUsage,
}

impl StringHeap {
    /// Creates an empty store without allocating payload or slot storage.
    ///
    /// # Errors
    /// Returns identity exhaustion rather than wrapping/reusing a store namespace.
    pub fn new(budget: StringHeapBudget) -> Result<Self, HeapError> {
        let owner = HeapIdentity::allocate().map_err(|kind| HeapError {
            kind,
            usage: HeapWorkUsage::default(),
        })?;
        Ok(Self {
            owner,
            budget,
            slots: Vec::new(),
            free_head: None,
            usage: StringHeapUsage::default(),
        })
    }

    /// Returns logical retained counts without exposing payload or IDs.
    #[must_use]
    pub fn usage(&self) -> StringHeapUsage {
        self.usage
    }

    /// Returns configured admission limits without changing live storage.
    #[must_use]
    pub fn budget(&self) -> StringHeapBudget {
        self.budget
    }

    /// Copies complete bytes after count/payload/slot/work admission.
    ///
    /// # Errors
    /// Rejects overflow, exhaustion and fallible allocation. No partial value is
    /// inserted; logical retained usage/old values stay unchanged. Spare capacity
    /// may grow before a later allocation fails; it is explicitly outside counters.
    pub fn allocate(
        &mut self,
        bytes: &[u8],
        work: HeapWorkBudget,
    ) -> Result<HeapAllocation, HeapError> {
        string_allocation::allocate(self, StringInput::Bytes(bytes), work)
    }

    /// Copies checked left/right payloads into one admitted immutable result.
    ///
    /// # Errors
    /// Rejects stale/foreign operands, checked overflow, budget exhaustion or
    /// allocation failure. Inputs remain unchanged; no partial result is inserted.
    pub fn concatenate(
        &mut self,
        left: StringId,
        right: StringId,
        work: HeapWorkBudget,
    ) -> Result<HeapAllocation, HeapError> {
        string_allocation::allocate(self, StringInput::Pair { left, right }, work)
    }

    /// Borrows exact bytes for a checked reference, without assuming UTF-8.
    /// The borrow prevents mutable store operations through this owner until it ends.
    ///
    /// # Errors
    /// Rejects foreign/recreated owners and missing/vacant/retired/stale generations.
    pub fn get(&self, id: StringId) -> Result<&[u8], HeapErrorKind> {
        let slot = self
            .slots
            .get(self.index(id)?)
            .ok_or(HeapErrorKind::InvalidState)?;
        match &slot.state {
            StringSlotState::Occupied(bytes) => Ok(bytes),
            _ => Err(HeapErrorKind::StaleReference),
        }
    }

    /// Reclaims every string absent from the complete supplied root set.
    ///
    /// Roots may repeat. All references and fallible work/scratch/count changes
    /// preflight before deletion; copies alone do not keep strings alive. This
    /// immutable-leaf operation does not trace objects, frames or external state.
    ///
    /// # Errors
    /// Failed root validation/admission/allocation leaves payload, IDs and logical
    /// retained usage unchanged, retaining charged work/scratch in the error.
    pub fn reclaim(
        &mut self,
        roots: &[StringId],
        work: HeapWorkBudget,
    ) -> Result<StringCollection, HeapError> {
        string_reclamation::reclaim(self, roots, work)
    }

    pub(super) fn index(&self, id: StringId) -> Result<usize, HeapErrorKind> {
        if id.owner != self.owner {
            return Err(HeapErrorKind::ForeignReference);
        }
        let slot = self
            .slots
            .get(id.slot)
            .ok_or(HeapErrorKind::StaleReference)?;
        if slot.generation != id.generation || !matches!(slot.state, StringSlotState::Occupied(_)) {
            return Err(HeapErrorKind::StaleReference);
        }
        Ok(id.slot)
    }
}

impl fmt::Debug for StringHeap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StringHeap")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
