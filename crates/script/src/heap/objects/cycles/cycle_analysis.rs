// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::cycles::cycle_edges;
use crate::heap::objects::cycles::cycle_workspace::CycleWorkspace;
use crate::heap::objects::object_slot::ObjectState;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{
    HeapWorkBudget, ObjectCycleAnalysis, ObjectHeap, ObjectHeapError, ObjectHeapErrorKind, ObjectId,
};

impl ObjectHeap {
    /// Analyzes unreachable reference-cycle closure and its unique live targets.
    ///
    /// Read-only and iterative. Complete supplied roots plus live managed array
    /// locations determine reachability. Array snapshots alone are not roots. Native ref
    /// counts, array behavior and native GC are not inferred. All node/edge visits,
    /// checked references and logical flag/count/queue/output payload are admitted.
    /// Keys/payloads are not read, normalized or exposed. Ordinary reclamation is unchanged.
    /// # Errors
    /// Invalid/stale/foreign roots, graph invariants, allocation or work/scratch
    /// limits return charged usage without changing the graph or publishing a partial report.
    pub fn analyze_cycles(
        &self,
        roots: &[ObjectId],
        budget: HeapWorkBudget,
    ) -> Result<ObjectCycleAnalysis, ObjectHeapError> {
        let mut work = ObjectWork::new(budget);
        analyze(self, roots, &mut work).map_err(|kind| work.error(kind))
    }
}

fn analyze(
    heap: &ObjectHeap,
    roots: &[ObjectId],
    work: &mut ObjectWork,
) -> Result<ObjectCycleAnalysis, ObjectHeapErrorKind> {
    let mut state = CycleWorkspace::new(heap, roots, work)?;
    indegrees(heap, &mut state, work)?;
    for (index, slot) in heap.objects.iter().enumerate() {
        work.charge(1)?;
        if matches!(slot.state, ObjectState::Occupied { .. })
            && !state.live[index]
            && state.degrees[index] == 0
        {
            state.enqueue(index)?;
        }
    }
    let mut cursor = 0;
    while cursor < state.queue.len() {
        work.charge(1)?;
        let owner = state.queue[cursor];
        cursor += 1;
        cycle_edges::walk(heap, owner, work, |target| {
            if !state.live[target] {
                let degree = state
                    .degrees
                    .get_mut(target)
                    .ok_or(ObjectHeapErrorKind::InvalidState)?;
                *degree = degree
                    .checked_sub(1)
                    .ok_or(ObjectHeapErrorKind::InvalidState)?;
                if *degree == 0 {
                    state.enqueue(target)?;
                }
            }
            Ok(())
        })?;
    }
    boundary(heap, &mut state, work)
}

fn indegrees(
    heap: &ObjectHeap,
    state: &mut CycleWorkspace,
    work: &mut ObjectWork,
) -> Result<(), ObjectHeapErrorKind> {
    let (mut objects, mut fields) = (0_u64, 0_u64);
    for (index, slot) in heap.objects.iter().enumerate() {
        work.charge(1)?;
        if matches!(slot.state, ObjectState::Occupied { .. }) {
            objects = objects
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            let count = cycle_edges::walk(heap, index, work, |target| {
                if state.live[index] && !state.live[target] {
                    return Err(ObjectHeapErrorKind::InvalidState);
                }
                if !state.live[index] && !state.live[target] {
                    let degree = state
                        .degrees
                        .get_mut(target)
                        .ok_or(ObjectHeapErrorKind::InvalidState)?;
                    *degree = degree
                        .checked_add(1)
                        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
                }
                Ok(())
            })?;
            fields = fields
                .checked_add(count as u64)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        }
    }
    if objects != heap.usage.live_objects || fields != heap.usage.live_fields {
        return Err(ObjectHeapErrorKind::InvalidState);
    }
    Ok(())
}

fn boundary(
    heap: &ObjectHeap,
    state: &mut CycleWorkspace,
    work: &mut ObjectWork,
) -> Result<ObjectCycleAnalysis, ObjectHeapErrorKind> {
    let (mut unreachable, mut retained, mut edges) = (0_u64, 0_u64, 0_u64);
    for (index, slot) in heap.objects.iter().enumerate() {
        work.charge(1)?;
        if !matches!(slot.state, ObjectState::Occupied { .. }) || state.live[index] {
            continue;
        }
        unreachable = unreachable
            .checked_add(1)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        if state.degrees[index] > 0 {
            retained = retained
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
            cycle_edges::walk(heap, index, work, |target| {
                if state.live[target] {
                    edges = edges
                        .checked_add(1)
                        .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
                    state.targets[target] = true;
                }
                Ok(())
            })?;
        }
    }
    let live_targets = state.output(heap, work)?;
    Ok(ObjectCycleAnalysis {
        unreachable_objects: unreachable,
        cycle_retained_objects: retained,
        cycle_to_live_edges: edges,
        live_targets,
        usage: work.usage(),
    })
}
