// SPDX-License-Identifier: AGPL-3.0-or-later
use std::mem;

use crate::heap::heap_work::HeapWork;
use crate::heap::string_slot::StringSlotState;
use crate::heap::{
    HeapError, HeapErrorKind, HeapWorkBudget, StringCollection, StringHeap, StringHeapUsage,
    StringId,
};

pub(super) fn reclaim(
    heap: &mut StringHeap,
    roots: &[StringId],
    budget: HeapWorkBudget,
) -> Result<StringCollection, HeapError> {
    let mut work = HeapWork::new(budget);
    collect(heap, roots, &mut work).map_err(|kind| work.error(kind))
}

fn collect(
    heap: &mut StringHeap,
    roots: &[StringId],
    work: &mut HeapWork,
) -> Result<StringCollection, HeapErrorKind> {
    let count = heap.slots.len();
    work.scratch(count as u64)?;
    work.charge(count as u64)?;
    let mut marked = Vec::new();
    marked
        .try_reserve_exact(count)
        .map_err(|_| HeapErrorKind::AllocationFailed)?;
    marked.resize(count, false);
    for root in roots {
        work.charge(1)?;
        let index = heap.index(*root)?;
        *marked.get_mut(index).ok_or(HeapErrorKind::InvalidState)? = true;
    }
    let next = preflight(heap, &marked, work)?;
    // Reserve every final sweep visit before any mutation. Applying the plan has
    // no allocation, user callbacks, reference validation or fallible arithmetic.
    work.charge(count as u64)?;
    apply(heap, &marked);
    let freed_strings = heap.usage.live_strings - next.live_strings;
    let freed_payload_bytes = heap.usage.payload_bytes - next.payload_bytes;
    heap.usage = next;
    Ok(StringCollection {
        freed_strings,
        freed_payload_bytes,
        usage: work.usage,
    })
}

fn preflight(
    heap: &StringHeap,
    marked: &[bool],
    work: &mut HeapWork,
) -> Result<StringHeapUsage, HeapErrorKind> {
    if marked.len() != heap.slots.len() {
        return Err(HeapErrorKind::InvalidState);
    }
    let mut next = heap.usage;
    for (slot, marked) in heap.slots.iter().zip(marked) {
        work.charge(1)?;
        if let StringSlotState::Occupied(bytes) = &slot.state
            && !marked
        {
            next.live_strings = next
                .live_strings
                .checked_sub(1)
                .ok_or(HeapErrorKind::InvalidState)?;
            next.payload_bytes = next
                .payload_bytes
                .checked_sub(bytes.len() as u64)
                .ok_or(HeapErrorKind::InvalidState)?;
            if slot.generation == u64::MAX {
                next.retired_slots = next
                    .retired_slots
                    .checked_add(1)
                    .ok_or(HeapErrorKind::SizeOverflow)?;
            }
        }
    }
    Ok(next)
}

fn apply(heap: &mut StringHeap, marked: &[bool]) {
    let mut free_head = heap.free_head;
    for (index, (slot, marked)) in heap.slots.iter_mut().zip(marked).enumerate() {
        if *marked || !matches!(slot.state, StringSlotState::Occupied(_)) {
            continue;
        }
        let old = mem::replace(&mut slot.state, StringSlotState::Retired);
        drop(old);
        if let Some(next) = slot.generation.checked_add(1) {
            slot.generation = next;
            slot.state = StringSlotState::Vacant { next: free_head };
            free_head = Some(index);
        }
    }
    heap.free_head = free_head;
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "Private tests inject the unforgeable generation boundary."
    )]
    use crate::heap::{
        HeapErrorKind, HeapResource, HeapWorkBudget, StringHeap, StringHeapBudget, StringId,
    };

    #[test]
    fn exhausted_generations_retire_instead_of_wrapping_or_reusing_references() {
        let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Store");
        let first = heap
            .allocate(b"old", HeapWorkBudget::default())
            .expect("Value")
            .id;
        heap.slots[0].generation = u64::MAX;
        let last = StringId {
            generation: u64::MAX,
            ..first
        };
        assert_eq!(heap.get(last).expect("Full generation domain"), b"old");
        heap.reclaim(&[], HeapWorkBudget::default())
            .expect("Retire");
        assert_eq!(heap.usage().retired_slots, 1);
        assert_eq!(heap.get(last), Err(HeapErrorKind::StaleReference));
        let next = heap
            .allocate(b"new", HeapWorkBudget::default())
            .expect("Fresh slot")
            .id;
        assert_eq!(next.slot, 1);
        assert_eq!(next.generation, 1);
        heap.reclaim(&[], HeapWorkBudget::default())
            .expect("Reusable ordinary generation");
        let again = heap
            .allocate(b"again", HeapWorkBudget::default())
            .expect("Reuse")
            .id;
        assert_eq!(again.slot, 1);
        assert_eq!(again.generation, 2);
        assert_eq!(heap.usage().slots, 2);
        assert_eq!(heap.usage().retired_slots, 1);
    }

    #[test]
    fn retired_slots_still_consume_retained_slot_admission() {
        let mut heap = StringHeap::new(StringHeapBudget {
            max_slots: 1,
            ..StringHeapBudget::default()
        })
        .expect("Store");
        heap.allocate(b"old", HeapWorkBudget::default())
            .expect("Value");
        heap.slots[0].generation = u64::MAX;
        heap.reclaim(&[], HeapWorkBudget::default())
            .expect("Retire");
        let before = heap.usage();
        assert_eq!(
            heap.allocate(b"new", HeapWorkBudget::default())
                .expect_err("Cannot reuse retired storage")
                .kind,
            HeapErrorKind::BudgetExceeded {
                resource: HeapResource::Slots,
                requested: 2,
                maximum: 1
            }
        );
        assert_eq!(heap.usage(), before);
    }
}
