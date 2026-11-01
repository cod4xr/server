// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::heap::{HeapWorkUsage, ObjectId};

/// Read-only graph analysis under a complete supplied root set, not native GC state.
///
/// Counts exclude vacant/retired arena slots. Output references are checked
/// nonowning snapshots; retaining this report does not root targets or their store.
pub struct ObjectCycleAnalysis {
    /// Objects not reachable from supplied roots, including acyclic garbage.
    pub unreachable_objects: u64,
    /// Unreachable objects in cycles or reachable from unreachable cycles.
    /// This is the reference-count-retained closure, not just cycle members.
    pub cycle_retained_objects: u64,
    /// Object-reference edges from that closure into root-reachable objects.
    pub cycle_to_live_edges: u64,
    /// Charged work and peak logical flag/count/queue/output scratch.
    pub usage: HeapWorkUsage,
    pub(super) live_targets: Vec<ObjectId>,
}

impl ObjectCycleAnalysis {
    /// Borrows unique boundary targets in arena-slot order, without ownership leases.
    #[must_use]
    pub fn live_targets(&self) -> &[ObjectId] {
        &self.live_targets
    }
}

impl fmt::Debug for ObjectCycleAnalysis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ObjectCycleAnalysis")
            .field("unreachable_objects", &self.unreachable_objects)
            .field("cycle_retained_objects", &self.cycle_retained_objects)
            .field("cycle_to_live_edges", &self.cycle_to_live_edges)
            .field("live_targets", &self.live_targets.len())
            .field("usage", &self.usage)
            .finish()
    }
}
