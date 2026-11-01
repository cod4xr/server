// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests own immutable string stores and deliberate stale/foreign references."
)]

use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget, StringHeap, StringHeapBudget};

#[test]
fn string_store_owns_exact_bytes_and_copies_references_without_payload_or_identity_leaks() {
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Store");
    let mut input = b"PrivatePayload\0\xff".to_vec();
    let allocation = heap
        .allocate(&input, HeapWorkBudget::default())
        .expect("Owned bytes");
    let id = allocation.id;
    let copy = id;
    input.fill(0);
    assert_eq!(
        heap.get(copy).expect("Independent ownership"),
        b"PrivatePayload\0\xff"
    );
    assert_eq!(heap.usage().live_strings, 1);
    assert_eq!(heap.usage().payload_bytes, 16);
    assert_eq!(format!("{id:?}"), "StringId(..)");
    assert!(!format!("{heap:?}").contains("Private"));
    heap.reclaim(&[id, copy], HeapWorkBudget::default())
        .expect("Both copies explicitly rooted");
    assert_eq!(heap.get(id).expect("Still live"), b"PrivatePayload\0\xff");
}

#[test]
fn collection_reuses_slots_with_fresh_generations_and_rejects_dropped_or_foreign_owners() {
    let mut old = StringHeap::new(StringHeapBudget::default()).expect("Old store");
    let stale = old
        .allocate(b"old", HeapWorkBudget::default())
        .expect("Value")
        .id;
    old.reclaim(&[], HeapWorkBudget::default())
        .expect("No live roots");
    assert_eq!(old.get(stale), Err(HeapErrorKind::StaleReference));
    let current = old
        .allocate(b"new", HeapWorkBudget::default())
        .expect("Reused slot")
        .id;
    assert_ne!(stale, current);
    assert_eq!(old.usage().slots, 1);
    assert_eq!(old.get(current).expect("New generation"), b"new");
    drop(old);
    let mut new = StringHeap::new(StringHeapBudget::default()).expect("Replacement store");
    let replacement = new
        .allocate(b"new", HeapWorkBudget::default())
        .expect("Replacement value")
        .id;
    assert_ne!(current, replacement);
    assert_eq!(new.get(current), Err(HeapErrorKind::ForeignReference));
    assert_eq!(new.get(stale), Err(HeapErrorKind::ForeignReference));
}

#[test]
fn concatenation_preserves_full_plain_bytes_and_self_aliases_without_native_scratch_limits() {
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Store");
    let bytes = vec![0x80; 100_000];
    let left = heap
        .allocate(&bytes, HeapWorkBudget::default())
        .expect("Large source")
        .id;
    let joined = heap
        .concatenate(left, left, HeapWorkBudget::default())
        .expect("Self concatenation");
    assert!((200_005..=200_007).contains(&joined.usage.steps));
    assert_eq!(
        heap.get(joined.id).expect("Full result"),
        vec![0x80; 200_000]
    );
    assert_eq!(heap.get(left).expect("Unchanged source"), bytes);
    let empty = heap
        .allocate(&[], HeapWorkBudget::default())
        .expect("Empty string")
        .id;
    let same = heap
        .concatenate(empty, joined.id, HeapWorkBudget::default())
        .expect("Empty prefix")
        .id;
    assert_eq!(
        heap.get(same).expect("Copied content"),
        heap.get(joined.id).expect("Original content")
    );
    assert_ne!(same, joined.id);
}
