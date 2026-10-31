// SPDX-License-Identifier: AGPL-3.0-or-later
/// An engine asset category. These variants have no serialized numeric identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AssetKind {
    /// Encoded image content.
    Image,
    /// Material definition content.
    Material,
    /// Model definition/content.
    Model,
    /// Animation content.
    Animation,
    /// Sound content.
    Sound,
    /// Script source or separately identified bytecode content.
    Script,
    /// Generic raw-file content.
    RawFile,
}
