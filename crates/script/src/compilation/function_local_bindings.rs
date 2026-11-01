// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

use crate::binding::LegacyIdentifier;

#[derive(Clone, Copy)]
pub(super) struct FunctionLocalSlot {
    pub(super) index: usize,
    pub(super) initialized: bool,
}

pub(super) struct FunctionLocalBindings<'source> {
    pub(super) names: HashMap<LegacyIdentifier<'source>, FunctionLocalSlot>,
}

impl FunctionLocalBindings<'_> {
    pub(super) fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum FunctionLocalDeclaration {
    Parameter,
    Assignment,
}
