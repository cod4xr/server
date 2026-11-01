// SPDX-License-Identifier: AGPL-3.0-or-later
mod arrays;
mod cycles;
mod field_index;
mod field_insertion;
mod field_removal;
mod index_rebuild;
mod object_allocation;
mod object_allocation_plan;
mod object_budget;
mod object_error;
mod object_fields;
mod object_heap;
mod object_id;
mod object_lookup;
mod object_mark;
mod object_reclamation;
mod object_slot;
mod object_strings;
#[cfg(test)]
mod object_tests;
mod object_usage;
mod object_work;

pub use arrays::{ArrayCowOutcome, ArrayId, ArrayResult, ArrayValueId};
pub use cycles::ObjectCycleAnalysis;
pub use object_budget::ObjectHeapBudget;
pub use object_error::{ObjectHeapError, ObjectHeapErrorKind, ObjectResource};
pub use object_heap::ObjectHeap;
pub use object_id::ObjectId;
pub use object_usage::{
    ObjectAllocation, ObjectCollection, ObjectFieldResult, ObjectHeapUsage, ObjectStringCollection,
};
