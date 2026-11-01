// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact checked host bytes, not native interning/NUL conformance or array execution.
#![allow(
    clippy::expect_used,
    reason = "Tests verify borrowed key payloads, private diagnostics and checked string namespaces."
)]
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    ArrayKey, HeapErrorKind, HeapWorkBudget, LegacyArrayKeyError, StringHeap, StringHeapBudget,
};

#[test]
fn byte_keys_borrow_full_binary_case_sensitive_large_and_duplicate_contents_without_allocation()
-> Result<(), String> {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let large = vec![b'A'; 100_000];
    let payloads = [
        b"".as_slice(),
        b"PrivateKey",
        b"privatekey",
        b"PrivateKey\0\xff",
        large.as_slice(),
        b"PrivateKey",
    ];
    let ids: Vec<_> = payloads
        .iter()
        .map(|bytes| {
            strings
                .allocate(bytes, HeapWorkBudget::default())
                .expect("Owned bytes")
                .id
        })
        .collect();
    assert_ne!(ids[1], ids[5]);
    let before = strings.usage();
    for (id, payload) in ids.iter().copied().zip(payloads) {
        let key =
            ArrayKey::from_legacy_value(KernelValue::String(id), &strings).expect("Borrowed key");
        let ArrayKey::Bytes(bytes) = key else {
            return Err("Expected byte key".into());
        };
        assert_eq!(bytes, payload);
        assert!(std::ptr::eq(
            bytes,
            strings.get(id).expect("Exact backing slice")
        ));
        assert!(!format!("{key:?}").contains("PrivateKey"));
    }
    assert_eq!(strings.usage(), before);
    assert!(!format!("{:?}", ArrayKey::Index(123_456_789)).contains("123456789"));
    Ok(())
}

#[test]
fn key_factory_rejects_foreign_and_stale_reused_strings_without_leaking_private_values() {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let stale = strings
        .allocate(b"PrivateStaleKey", HeapWorkBudget::default())
        .expect("Stale candidate")
        .id;
    strings
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Vacate generation");
    let _replacement = strings
        .allocate(b"PrivateReplacement", HeapWorkBudget::default())
        .expect("Reused slot")
        .id;
    let mut other = StringHeap::new(StringHeapBudget::default()).expect("Other store");
    let foreign = other
        .allocate(b"PrivateForeignKey", HeapWorkBudget::default())
        .expect("Foreign key")
        .id;
    for (id, expected) in [
        (stale, HeapErrorKind::StaleReference),
        (foreign, HeapErrorKind::ForeignReference),
    ] {
        let before = strings.usage();
        let error = ArrayKey::from_legacy_value(KernelValue::String(id), &strings)
            .expect_err("Checked key reference");
        assert_eq!(error, LegacyArrayKeyError::StringHeap(expected));
        assert!(!format!("{error:?} {error}").contains("Private"));
        assert_eq!(strings.usage(), before);
    }
}
