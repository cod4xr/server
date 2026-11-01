// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::ObjectId;

pub(in crate::heap::objects) struct ArrayValueSlot {
    pub(in crate::heap::objects) generation: u64,
    pub(in crate::heap::objects) state: ArrayValueState,
}

pub(in crate::heap::objects) enum ArrayValueState {
    Occupied(ObjectId),
    Vacant { next: Option<usize> },
    Retired,
}
