// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete supported collections preserve aliases/binary bytes without registration.
#![allow(
    clippy::expect_used,
    reason = "Tests use independent synthetic fixtures."
)]

mod zone_support;

use cod4x_content::zone::rawfiles::{RawFileBudget, RawFileCollection};

#[test]
fn inline_and_binary_buffers_borrow_exact_source_bytes_and_debug_hides_contents() {
    let stream = zone_support::raw_fixture("inline");
    let collection =
        RawFileCollection::decode(&stream, RawFileBudget::default()).expect("Collection");
    let file = collection.entries()[0].expect("Inline file");
    assert_eq!(file.name(), b"scripts/secret.gsc");
    assert_eq!(file.contents(), Some(b"synthetic\0binary\xff".as_slice()));
    let offset = stream
        .payload()
        .windows(file.name().len())
        .position(|bytes| bytes == file.name())
        .expect("Name in original bytes");
    assert_eq!(file.name().as_ptr(), stream.payload()[offset..].as_ptr());
    assert!(!format!("{file:?} {collection:?}").contains("secret"));
    assert!(!format!("{file:?}").contains("synthetic"));
    assert_eq!(collection.usage().temporary_bytes, 12);
    assert_eq!(collection.usage().virtual_bytes, 45);
    assert_eq!(collection.usage().inline_buffer_bytes, 18);
}

#[test]
fn root_and_inserted_aliases_share_views_and_temporary_headers_reuse_capacity() {
    let stream = zone_support::raw_fixture("aliases");
    let collection = RawFileCollection::decode(&stream, RawFileBudget::default()).expect("Aliases");
    let entries = collection.entries();
    assert_eq!(entries.len(), 5);
    let first = entries[0].expect("First");
    let prior = entries[1].expect("Prior root alias");
    let second = entries[2].expect("Inserted");
    let inserted = entries[3].expect("Inserted alias");
    assert_eq!(first, prior);
    assert_eq!(
        first.contents().expect("Buffer").as_ptr(),
        prior.contents().expect("Alias buffer").as_ptr()
    );
    assert_eq!(second, inserted);
    assert_eq!(second.name(), b"secret.gsc");
    assert_eq!(second.name().as_ptr(), first.name()[8..].as_ptr());
    assert_eq!(second.contents(), Some([].as_slice()));
    assert_eq!(entries[4], None);
    let usage = collection.usage();
    assert_eq!(usage.root_entries, 5);
    assert_eq!(usage.temporary_bytes, 12);
    assert_eq!(usage.virtual_bytes, 85);
    assert_eq!(usage.inline_name_bytes, 19);
    assert_eq!(usage.inline_buffer_bytes, 19);
}

#[test]
fn prefix_names_null_entries_null_buffers_and_empty_buffers_remain_distinct() {
    let stream = zone_support::raw_fixture("script_name");
    let collection =
        RawFileCollection::decode(&stream, RawFileBudget::default()).expect("Prefix name");
    assert_eq!(
        collection.entries()[0].expect("File").name(),
        b"shared/secret.gsc"
    );
    assert_eq!(collection.usage().inline_name_bytes, 0);
    for label in ["null", "forward_null"] {
        let stream = zone_support::raw_fixture(label);
        let collection =
            RawFileCollection::decode(&stream, RawFileBudget::default()).expect("Null entries");
        assert!(collection.entries().iter().all(Option::is_none));
        assert_eq!(collection.usage().temporary_bytes, 0);
    }
    let stream = zone_support::raw_fixture("null_buffer");
    let collection =
        RawFileCollection::decode(&stream, RawFileBudget::default()).expect("Absent buffer");
    assert_eq!(collection.entries()[0].expect("Named").contents(), None);
    let stream = zone_support::raw_fixture("empty");
    let collection =
        RawFileCollection::decode(&stream, RawFileBudget::default()).expect("Empty buffer");
    assert_eq!(
        collection.entries()[0].expect("Named").contents(),
        Some([].as_slice())
    );
}
