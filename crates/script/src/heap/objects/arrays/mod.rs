// SPDX-License-Identifier: AGPL-3.0-or-later
mod array_allocation;
pub(super) mod array_collection;
mod array_cow;
mod array_entries;
#[cfg(test)]
mod array_entry_tests;
mod array_id;
mod array_keys;
mod array_lookup;
mod array_operation;
mod array_populated_copy;
mod array_reads;
mod array_resolution;
mod array_result;
mod array_storage;
#[cfg(test)]
mod array_tests;
mod array_value_plan;
mod array_value_slot;
mod array_values;
mod copy;

pub use array_id::{ArrayId, ArrayValueId};
pub use array_result::{ArrayCowOutcome, ArrayResult};
pub(super) use array_storage::{ArrayMetadata, ArrayStorage};
pub(super) use array_value_slot::{ArrayValueSlot, ArrayValueState};
