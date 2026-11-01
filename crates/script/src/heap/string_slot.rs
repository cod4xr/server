// SPDX-License-Identifier: AGPL-3.0-or-later
pub(super) struct StringSlot {
    pub(super) generation: u64,
    pub(super) state: StringSlotState,
}

pub(super) enum StringSlotState {
    Occupied(Vec<u8>),
    Vacant { next: Option<usize> },
    Retired,
}
