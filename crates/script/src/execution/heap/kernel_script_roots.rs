// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::heap::{ArrayValueId, ObjectId};

/// Borrowed fixed root bindings for one invocation, without ambient world access.
///
/// Copying this view creates no heap owners. Keep the game location alive until
/// execution finishes. Its reads clone managed VM locations; ordinary roots use
/// shared identity and remain GC roots even before their first script read.
/// Only ordinary structs and a managed game array are supported. This view
/// supplies no entity/thread/animation services, method receiver changes, native
/// object reference counts or game/map lifecycle policy.
#[derive(Clone, Copy)]
pub struct KernelScriptRoots {
    pub(super) level: ObjectId,
    pub(super) receiver: ObjectId,
    pub(super) game: ArrayValueId,
    pub(super) animation: ObjectId,
}

impl KernelScriptRoots {
    /// Borrows existing host bindings without allocation or validation. The bound
    /// execution context checks every root before entry and traces them during GC.
    #[must_use]
    pub fn new(
        level: ObjectId,
        receiver: ObjectId,
        game: ArrayValueId,
        animation: ObjectId,
    ) -> Self {
        Self {
            level,
            receiver,
            game,
            animation,
        }
    }

    pub(super) fn ordinary_objects(self) -> [ObjectId; 3] {
        [self.level, self.receiver, self.animation]
    }
}

impl fmt::Debug for KernelScriptRoots {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KernelScriptRoots(..)")
    }
}
