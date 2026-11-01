// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned immutable strings and a prototype shared-object store with checked identities.
//!
//! This storage boundary is not a complete VM/object collector. Callers own the
//! complete root set; copied IDs are checked references, never ownership leases.
//! Logical quotas exclude allocator rounding, spare capacity and process RSS.

mod arrays;
mod heap_budget;
mod heap_error;
mod heap_identity;
mod heap_usage;
mod heap_work;
mod objects;
mod string_allocation;
mod string_heap;
mod string_id;
mod string_reclamation;
mod string_slot;

pub use arrays::{ArrayKey, LegacyArrayKeyError};
pub use heap_budget::{HeapWorkBudget, StringHeapBudget};
pub use heap_error::{HeapError, HeapErrorKind, HeapResource};
pub use heap_usage::{HeapAllocation, HeapWorkUsage, StringCollection, StringHeapUsage};
pub(crate) use objects::ObjectArrayOwnerAudit;
pub use objects::{
    ArrayCowOutcome, ArrayId, ArrayResult, ArrayValueId, ObjectAllocation, ObjectCollection,
    ObjectCycleAnalysis, ObjectFieldResult, ObjectHeap, ObjectHeapBudget, ObjectHeapError,
    ObjectHeapErrorKind, ObjectHeapUsage, ObjectId, ObjectResource, ObjectStringCollection,
};
pub use string_heap::StringHeap;
pub use string_id::StringId;
