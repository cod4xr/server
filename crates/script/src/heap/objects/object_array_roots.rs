// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::{ObjectHeap, ObjectId};

pub(super) fn owned_data_roots<'a>(
    heap: &'a ObjectHeap,
    supplied: &'a [ObjectId],
) -> impl Iterator<Item = Option<ObjectId>> + Clone + 'a {
    supplied.iter().copied().map(Some).chain(
        heap.arrays
            .metadata
            .iter()
            .zip(&heap.objects)
            .enumerate()
            .map(|(index, (metadata, slot))| {
                (metadata.is_array && metadata.owners != 0).then_some(ObjectId {
                    owner: heap.owner,
                    slot: index,
                    generation: slot.generation,
                })
            }),
    )
}
