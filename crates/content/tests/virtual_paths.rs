// SPDX-License-Identifier: AGPL-3.0-or-later
//! Virtual path normalization and forbidden-component coverage.
#![allow(clippy::expect_used, reason = "Tests validate byte path boundaries.")]

use cod4x_content::{ArchiveError, ArchiveResource, VirtualPath};

#[test]
fn normalization_preserves_high_bytes_and_wide_names_without_host_paths() {
    let input = b"IMAGES\\CUSTOM_\xff.IWI";
    let path = VirtualPath::try_new(input, input.len()).expect("Path");
    assert_eq!(path.as_bytes(), b"images/custom_\xff.iwi");
    assert_eq!(input, b"IMAGES\\CUSTOM_\xff.IWI");
    let wide = vec![b'x'; 4096];
    assert!(VirtualPath::try_new(&wide, 4096).is_ok());
    assert_eq!(
        VirtualPath::try_new(&wide, 4095).err(),
        Some(ArchiveError::BudgetExceeded {
            resource: ArchiveResource::PathBytes,
        })
    );
}

#[test]
fn absolute_ambiguous_and_control_paths_fail_without_repair() {
    for bytes in [
        b"".as_slice(),
        b"/etc/test",
        b"\\server\\share",
        b"C:foo",
        b"a/../b",
        b"a/./b",
        b"a//b",
        b"a\\\\b",
        b"a/",
        b"a\\",
        b"a\0b",
        b"a\nb",
        b"a\x7fb",
    ] {
        assert_eq!(
            VirtualPath::try_new(bytes, 4096).err(),
            Some(ArchiveError::InvalidPath)
        );
    }
}
