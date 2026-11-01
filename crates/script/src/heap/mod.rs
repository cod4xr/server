// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned immutable string slots with checked identities and complete-root reclamation.
//!
//! This storage boundary is not a complete VM/object collector. Callers own the
//! complete root set; copied IDs are checked references, never ownership leases.
//! Logical quotas exclude allocator rounding, spare capacity and process RSS.

mod heap_budget;
mod heap_error;
mod heap_identity;
mod heap_usage;
mod heap_work;
mod string_allocation;
mod string_heap;
mod string_id;
mod string_reclamation;
mod string_slot;

pub use heap_budget::{HeapWorkBudget, StringHeapBudget};
pub use heap_error::{HeapError, HeapErrorKind, HeapResource};
pub use heap_usage::{HeapAllocation, HeapWorkUsage, StringCollection, StringHeapUsage};
pub use string_heap::StringHeap;
pub use string_id::StringId;
