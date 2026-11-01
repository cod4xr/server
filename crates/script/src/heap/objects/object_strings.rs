// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::objects::object_slot::FieldSlot;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{
    ObjectHeap, ObjectHeapError, ObjectHeapErrorKind, ObjectResource, ObjectStringCollection,
};
use crate::heap::{
    HeapErrorKind, HeapResource, HeapWorkBudget, HeapWorkUsage, StringHeap, StringId,
};

pub(super) fn reclaim(
    heap: &ObjectHeap,
    strings: &mut StringHeap,
    extra: &[StringId],
    budget: HeapWorkBudget,
) -> Result<ObjectStringCollection, ObjectHeapError> {
    let mut work = ObjectWork::new(budget);
    collect(heap, strings, extra, &mut work).map_err(|kind| work.error(kind))
}

fn collect(
    heap: &ObjectHeap,
    strings: &mut StringHeap,
    extra: &[StringId],
    work: &mut ObjectWork,
) -> Result<ObjectStringCollection, ObjectHeapErrorKind> {
    work.charge(1)?;
    if strings.owner != heap.string_owner {
        return Err(ObjectHeapErrorKind::ForeignStringHeap);
    }
    work.charge(heap.fields.len() as u64)?;
    let count = heap
        .fields
        .iter()
        .filter_map(string)
        .count()
        .checked_add(extra.len())
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    let bytes = (count as u64)
        .checked_mul(size_of::<StringId>() as u64)
        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
    work.scratch(bytes)?;
    work.charge(heap.fields.len() as u64)?;
    work.charge(extra.len() as u64)?;
    let mut roots = Vec::new();
    roots
        .try_reserve_exact(count)
        .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
    roots.extend(heap.fields.iter().filter_map(string));
    roots.extend_from_slice(extra);
    let base = work.usage().steps;
    let remaining = HeapWorkBudget {
        max_steps: work
            .budget
            .max_steps
            .checked_sub(base)
            .ok_or(ObjectHeapErrorKind::InvalidState)?,
        max_scratch_bytes: work
            .budget
            .max_scratch_bytes
            .checked_sub(bytes)
            .ok_or(ObjectHeapErrorKind::InvalidState)?,
    };
    match strings.reclaim(&roots, remaining) {
        Ok(result) => {
            record(work, result.usage, bytes)?;
            Ok(ObjectStringCollection {
                strings: result.freed_strings,
                payload_bytes: result.freed_payload_bytes,
                usage: work.usage(),
            })
        }
        Err(error) => {
            record(work, error.usage, bytes)?;
            Err(match error.kind {
                HeapErrorKind::BudgetExceeded {
                    resource: HeapResource::WorkSteps,
                    requested,
                    ..
                } => ObjectHeapErrorKind::BudgetExceeded {
                    resource: ObjectResource::WorkSteps,
                    requested: base
                        .checked_add(requested)
                        .ok_or(ObjectHeapErrorKind::SizeOverflow)?,
                    maximum: work.budget.max_steps,
                },
                HeapErrorKind::BudgetExceeded {
                    resource: HeapResource::ScratchBytes,
                    requested,
                    ..
                } => ObjectHeapErrorKind::BudgetExceeded {
                    resource: ObjectResource::ScratchBytes,
                    requested: bytes
                        .checked_add(requested)
                        .ok_or(ObjectHeapErrorKind::SizeOverflow)?,
                    maximum: work.budget.max_scratch_bytes,
                },
                kind => ObjectHeapErrorKind::StringHeap(kind),
            })
        }
    }
}

fn record(
    work: &mut ObjectWork,
    usage: HeapWorkUsage,
    roots: u64,
) -> Result<(), ObjectHeapErrorKind> {
    work.charge(usage.steps)?;
    work.scratch(
        roots
            .checked_add(usage.scratch_bytes)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?,
    )
}

fn string(slot: &FieldSlot) -> Option<StringId> {
    match slot {
        FieldSlot::Occupied(field) => match field.value {
            KernelValue::String(id) | KernelValue::LocalizedString(id) => Some(id),
            KernelValue::Undefined
            | KernelValue::Number(_)
            | KernelValue::Vector(_)
            | KernelValue::Function(_)
            | KernelValue::Object(_)
            | KernelValue::Array(_) => None,
        },
        FieldSlot::Vacant { .. } => None,
    }
}
