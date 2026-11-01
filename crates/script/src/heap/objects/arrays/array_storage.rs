// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::ObjectId;
use crate::heap::heap_identity::HeapIdentity;
use crate::heap::objects::arrays::{ArrayValueSlot, ArrayValueState};

#[derive(Clone, Copy, Default)]
pub(in crate::heap::objects) struct ArrayMetadata {
    pub(in crate::heap::objects) is_array: bool,
    pub(in crate::heap::objects) owners: u64,
}

#[derive(Default)]
pub(in crate::heap::objects) struct ArrayStorage {
    pub(in crate::heap::objects) owner: Option<HeapIdentity>,
    pub(in crate::heap::objects) metadata: Vec<ArrayMetadata>,
    pub(in crate::heap::objects) values: Vec<ArrayValueSlot>,
    pub(in crate::heap::objects) free_value: Option<usize>,
}

impl ArrayStorage {
    pub(in crate::heap::objects) fn is_array(&self, index: usize) -> bool {
        self.metadata
            .get(index)
            .is_some_and(|metadata| metadata.is_array)
    }

    pub(in crate::heap::objects) fn roots<'a>(
        &'a self,
        supplied: &'a [ObjectId],
    ) -> impl Iterator<Item = Option<ObjectId>> + Clone + 'a {
        supplied
            .iter()
            .copied()
            .map(Some)
            .chain(self.values.iter().map(|slot| match slot.state {
                ArrayValueState::Occupied(id) => Some(id),
                ArrayValueState::Vacant { .. } | ArrayValueState::Retired => None,
            }))
    }
}
