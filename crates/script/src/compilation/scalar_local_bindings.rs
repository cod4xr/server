// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

use crate::binding::LegacyIdentifier;

#[derive(Clone, Copy)]
pub(super) struct ScalarLocalSlot {
    pub(super) index: usize,
    pub(super) initialized: bool,
}

pub(super) struct ScalarLocalBindings<'source> {
    pub(super) names: HashMap<LegacyIdentifier<'source>, ScalarLocalSlot>,
}

impl ScalarLocalBindings<'_> {
    pub(super) fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ScalarLocalDeclaration {
    Parameter,
    Assignment,
}
