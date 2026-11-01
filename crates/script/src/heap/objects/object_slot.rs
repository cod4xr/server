// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;

pub(super) struct ObjectSlot {
    pub(super) generation: u64,
    pub(super) state: ObjectState,
}

pub(super) enum ObjectState {
    Occupied { first: Option<usize>, fields: usize },
    Vacant { next: Option<usize> },
    Retired,
}

pub(super) enum FieldSlot {
    Occupied(Field),
    Vacant { next: Option<usize> },
}

pub(super) struct Field {
    pub(super) owner: usize,
    pub(super) hash: u64,
    pub(super) key: Vec<u8>,
    pub(super) value: KernelValue,
    pub(super) previous: Option<usize>,
    pub(super) next: Option<usize>,
}
