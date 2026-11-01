// SPDX-License-Identifier: AGPL-3.0-or-later

/// Special source binding selected from an explicit invocation context.
/// These are semantic categories, not native IDs, addresses or opcode numbers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelScriptRoot {
    /// Current ordinary receiver supplied by the host, shared by ordinary calls.
    CurrentObject,
    /// Ordinary level object supplied by the host.
    Level,
    /// Managed game-array value location; reads create separate VM owners.
    Game,
    /// Ordinary animation root object, without animation asset/tree services.
    Animation,
}
