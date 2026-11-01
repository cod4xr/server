// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::KernelFunctionReference;
use crate::heap::{ObjectId, StringId};
use crate::legacy::{LegacyNumber, LegacyVector};

/// Primitive execution data and object-store snapshots; not the full GSC runtime.
///
/// Derived data equality is Rust equality, never a GSC language comparison.
/// Language comparisons use the explicit comparison instruction.
#[derive(Clone, Copy, PartialEq)]
pub enum KernelValue {
    /// Undefined may be returned, but numeric/branch operations reject it.
    Undefined,
    /// Legacy int32/binary32 data under the documented numeric profile.
    Number(LegacyNumber),
    /// Inline copied binary32 triple, without native pool/reference IDs.
    Vector(LegacyVector),
    /// Copied opaque module identity/index, never a native code address or ownership.
    Function(KernelFunctionReference),
    /// Nonowning immutable string reference, checked through an explicit heap context.
    String(StringId),
    /// Distinct legacy localized-string tag; no translation or host lookup occurs.
    LocalizedString(StringId),
    /// Nonowning shared struct snapshot requiring an explicit object-aware context.
    /// Reference transport does not supply source constructors/fields or array COW.
    Object(ObjectId),
}

impl KernelValue {
    pub(super) fn string_id(self) -> Option<StringId> {
        match self {
            Self::String(id) | Self::LocalizedString(id) => Some(id),
            _ => None,
        }
    }
}

impl fmt::Debug for KernelValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Undefined => "KernelValue::Undefined",
            Self::Number(_) => "KernelValue::Number(..)",
            Self::Vector(_) => "KernelValue::Vector(..)",
            Self::Function(_) => "KernelValue::Function(..)",
            Self::String(_) => "KernelValue::String(..)",
            Self::LocalizedString(_) => "KernelValue::LocalizedString(..)",
            Self::Object(_) => "KernelValue::Object(..)",
        })
    }
}
