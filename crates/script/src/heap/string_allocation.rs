// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::heap_error;
use crate::heap::heap_work::HeapWork;
use crate::heap::string_slot::{StringSlot, StringSlotState};
use crate::heap::{
    HeapAllocation, HeapError, HeapErrorKind, HeapResource, HeapWorkBudget, StringHeap,
    StringHeapUsage, StringId,
};

#[derive(Clone, Copy)]
pub(super) enum StringInput<'a> {
    Bytes(&'a [u8]),
    Pair { left: StringId, right: StringId },
}

impl StringInput<'_> {
    fn length(self, heap: &StringHeap) -> Result<usize, HeapErrorKind> {
        match self {
            Self::Bytes(bytes) => Ok(bytes.len()),
            Self::Pair { left, right } => heap
                .get(left)?
                .len()
                .checked_add(heap.get(right)?.len())
                .ok_or(HeapErrorKind::SizeOverflow),
        }
    }

    fn copy(self, heap: &StringHeap, output: &mut Vec<u8>) -> Result<(), HeapErrorKind> {
        match self {
            Self::Bytes(bytes) => output.extend_from_slice(bytes),
            Self::Pair { left, right } => {
                output.extend_from_slice(heap.get(left)?);
                output.extend_from_slice(heap.get(right)?);
            }
        }
        Ok(())
    }
}

pub(super) fn allocate(
    heap: &mut StringHeap,
    input: StringInput<'_>,
    budget: HeapWorkBudget,
) -> Result<HeapAllocation, HeapError> {
    let mut work = HeapWork::new(budget);
    let id = insert(heap, input, &mut work).map_err(|kind| work.error(kind))?;
    Ok(HeapAllocation {
        id,
        usage: work.usage,
    })
}

fn insert(
    heap: &mut StringHeap,
    input: StringInput<'_>,
    work: &mut HeapWork,
) -> Result<StringId, HeapErrorKind> {
    work.charge(match input {
        StringInput::Bytes(_) => 1,
        StringInput::Pair { .. } => 5,
    })?;
    let length = input.length(heap)?;
    let next = admission(heap, length)?;
    let index = heap.free_head.unwrap_or(heap.slots.len());
    let (generation, next_free) = if let Some(index) = heap.free_head {
        let slot = heap.slots.get(index).ok_or(HeapErrorKind::InvalidState)?;
        let StringSlotState::Vacant { next } = slot.state else {
            return Err(HeapErrorKind::InvalidState);
        };
        (slot.generation, next)
    } else {
        (1, None)
    };
    work.charge(length as u64)?;
    if heap.free_head.is_none() {
        if heap.slots.len() == heap.slots.capacity() {
            work.charge(heap.slots.len() as u64)?;
        }
        heap.slots
            .try_reserve(1)
            .map_err(|_| HeapErrorKind::AllocationFailed)?;
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| HeapErrorKind::AllocationFailed)?;
    input.copy(heap, &mut bytes)?;
    if heap.free_head.is_some() {
        heap.slots
            .get_mut(index)
            .ok_or(HeapErrorKind::InvalidState)?
            .state = StringSlotState::Occupied(bytes);
        heap.free_head = next_free;
    } else {
        heap.slots.push(StringSlot {
            generation,
            state: StringSlotState::Occupied(bytes),
        });
    }
    heap.usage = next;
    Ok(StringId {
        owner: heap.owner,
        slot: index,
        generation,
    })
}

fn admission(heap: &StringHeap, length: usize) -> Result<StringHeapUsage, HeapErrorKind> {
    let budget = heap.budget;
    heap_error::check(
        HeapResource::SingleStringBytes,
        length as u64,
        budget.max_single_string_bytes,
    )?;
    let mut next = heap.usage;
    next.live_strings = next
        .live_strings
        .checked_add(1)
        .ok_or(HeapErrorKind::SizeOverflow)?;
    next.payload_bytes = next
        .payload_bytes
        .checked_add(length as u64)
        .ok_or(HeapErrorKind::SizeOverflow)?;
    if heap.free_head.is_none() {
        next.slots = next
            .slots
            .checked_add(1)
            .ok_or(HeapErrorKind::SizeOverflow)?;
    }
    next.metadata_bytes = next
        .slots
        .checked_mul(size_of::<StringSlot>() as u64)
        .ok_or(HeapErrorKind::SizeOverflow)?;
    for (resource, requested, maximum) in [
        (
            HeapResource::LiveStrings,
            next.live_strings,
            budget.max_live_strings,
        ),
        (
            HeapResource::PayloadBytes,
            next.payload_bytes,
            budget.max_payload_bytes,
        ),
        (HeapResource::Slots, next.slots, budget.max_slots),
        (
            HeapResource::MetadataBytes,
            next.metadata_bytes,
            budget.max_metadata_bytes,
        ),
    ] {
        heap_error::check(resource, requested, maximum)?;
    }
    Ok(next)
}
