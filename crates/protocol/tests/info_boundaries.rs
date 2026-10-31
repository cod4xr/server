// SPDX-License-Identifier: AGPL-3.0-or-later
//! Info-string resource bounds and mutation safety.
#![allow(
    clippy::expect_used,
    reason = "Tests construct bounded valid info strings."
)]

use cod4x_protocol::{InfoError, InfoLimit, InfoString, InfoWriter};

#[test]
fn optional_initial_delimiters_empty_values_and_high_bytes_are_preserved() {
    for bytes in [
        b"key\xff\\value\xfe\\empty\\".as_slice(),
        b"\\key\xff\\value\xfe\\empty\\",
    ] {
        let info = InfoString::parse(bytes, InfoLimit::Normal).expect("Valid high bytes");
        assert_eq!(info.get(b"KEY\xff"), Some(b"value\xfe".as_slice()));
        assert_eq!(info.get(b"empty"), Some(b"".as_slice()));
        assert_eq!(info.get(b"absent"), None);
        assert_eq!(info.pairs().count(), 2);
    }
}

#[test]
fn duplicate_keys_keep_first_case_insensitive_lookup_order() {
    let info = InfoString::parse(b"\\KEY\\first\\key\\second", InfoLimit::Normal)
        .expect("Duplicates are valid wire data");
    assert_eq!(info.get(b"key"), Some(b"first".as_slice()));
    assert_eq!(info.pairs().count(), 2);
}

#[test]
fn malformed_structures_and_forbidden_characters_are_rejected() {
    for input in [
        b"\\".as_slice(),
        b"key",
        b"\\\\v",
        b"\\k\\v\\",
        b"\\k\n\\v",
        b"\\k\\v\0",
        b"\\k\\v;",
        b"\\k\\v\"",
        b"\\k\\v\x7f",
    ] {
        assert!(InfoString::parse(input, InfoLimit::Normal).is_err());
    }
}

#[test]
fn exact_normal_and_big_limits_reserve_the_original_c_terminator() {
    for limit in [InfoLimit::Normal, InfoLimit::Big] {
        let maximum = limit.maximum_bytes();
        let mut storage = vec![0xa5; maximum + 1];
        let mut writer = InfoWriter::new(&mut storage, limit);
        let value = vec![b'v'; maximum - 3];
        writer.set(b"k", &value).expect("Exact encoded length");
        assert_eq!(writer.as_bytes().len(), maximum);
        assert!(InfoString::parse(writer.as_bytes(), limit).is_ok());
        let before = writer.as_bytes().to_vec();
        assert_eq!(writer.set(b"x", b"y"), Err(InfoError::TooLong));
        assert_eq!(writer.as_bytes(), before);
        let mut oversized = before;
        oversized.push(b'v');
        assert_eq!(
            InfoString::parse(&oversized, limit),
            Err(InfoError::TooLong)
        );
    }
}

#[test]
fn failed_replacement_does_not_remove_the_previous_value() {
    let mut storage = [0; 8];
    let mut writer = InfoWriter::new(&mut storage, InfoLimit::Normal);
    writer.set(b"k", b"old").expect("Fits");
    let before = writer.as_bytes().to_vec();
    for (key, value) in [
        (b"k".as_slice(), b"too long".as_slice()),
        (b"k", b"bad;"),
        (b"k", b"bad\\"),
        (b"", b"value"),
    ] {
        assert!(writer.set(key, value).is_err());
        assert_eq!(writer.as_bytes(), before);
    }
    writer.set(b"k", b"").expect("Removal fits");
    assert!(writer.as_bytes().is_empty());
}

#[test]
fn inserting_and_removing_middle_pairs_preserves_neighbors() {
    let mut storage = [0; 64];
    let mut writer = InfoWriter::new(&mut storage, InfoLimit::Normal);
    for key in [b"a", b"b", b"c"] {
        writer.set(key, b"1").expect("Small field");
    }
    writer.set(b"b", b"").expect("Remove middle");
    assert_eq!(writer.as_bytes(), b"\\c\\1\\a\\1");
    writer.set(b"a", b"2").expect("Replace tail");
    assert_eq!(writer.as_bytes(), b"\\a\\2\\c\\1");
    writer
        .set(b"missing", b"")
        .expect("Removing absent key is a no-op");
    assert_eq!(writer.as_bytes(), b"\\a\\2\\c\\1");
}
