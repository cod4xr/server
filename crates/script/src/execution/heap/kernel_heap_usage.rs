// SPDX-License-Identifier: AGPL-3.0-or-later
/// Shared logical heap accounting; excludes allocator rounding, spare capacity and RSS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KernelHeapUsage {
    /// Admitted discovery, comparison, storage and prepaid ownership cleanup,
    /// including failed operations. Unused cleanup credit is not refunded.
    pub steps: u64,
    /// Successfully completed string collections, including ones freeing no values.
    pub collections: u64,
    /// Completed object collections; a subsequent string collection may still fail.
    pub object_collections: u64,
    /// Peak simultaneous cache, owned-array registries, temporary roots and marks.
    pub peak_scratch_bytes: u64,
}
