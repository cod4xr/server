// SPDX-License-Identifier: AGPL-3.0-or-later
/// Shared logical heap accounting; excludes allocator rounding, spare capacity and RSS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KernelHeapUsage {
    /// Admitted root discovery, comparison and storage operation work, including failures.
    pub steps: u64,
    /// Successfully completed pressure collections, including ones freeing no values.
    pub collections: u64,
    /// Peak simultaneous literal cache/indices, temporary root IDs and logical mark flags.
    pub peak_scratch_bytes: u64,
}
