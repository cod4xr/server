// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::zone::ZonePrefixBudget;

/// Configurable raw-file decode/work budgets in addition to envelope admission.
///
/// Metadata accounts for logical directory slots, alias bindings, and name spans,
/// excluding allocator/hash-table overhead and separately budgeted prefix metadata.
/// Names/buffers borrow the source; byte budgets also bound decode work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawFileBudget {
    /// Root tables and initial script strings.
    pub prefix: ZonePrefixBudget,
    /// One inline or referenced name, including its NUL.
    pub max_name_bytes: u64,
    /// Total inline raw-file name bytes, including NULs; references do not recopy.
    pub max_total_name_bytes: u64,
    /// One present raw-file buffer, including its final NUL.
    pub max_buffer_bytes: u64,
    /// Sum of present inline buffers, including NULs; aliases do not reread bytes.
    pub max_total_buffer_bytes: u64,
    /// Logical owned decode metadata, including staging released before return.
    pub max_metadata_bytes: u64,
}

impl Default for RawFileBudget {
    fn default() -> Self {
        Self {
            prefix: ZonePrefixBudget::default(),
            max_name_bytes: 64 * 1024,
            max_total_name_bytes: 32 * 1024 * 1024,
            max_buffer_bytes: 256 * 1024 * 1024,
            max_total_buffer_bytes: 2 * 1024 * 1024 * 1024,
            max_metadata_bytes: 64 * 1024 * 1024,
        }
    }
}
