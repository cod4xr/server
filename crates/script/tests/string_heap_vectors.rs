// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual native-helper results and explicit capacity expansion through owned slots.
#![allow(
    clippy::expect_used,
    reason = "Tests replay the pinned synthetic byte corpus through literal/store boundaries."
)]

use crate::string_support;
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::literal::LegacyStringKind;

#[test]
fn native_plain_results_and_expanded_capacity_survive_literal_decode_store_concat_and_reclamation()
-> Result<(), String> {
    let (mut parity, mut expansion) = (0, 0);
    for row in include_str!("fixtures/string_operation_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let seed: usize = fields[0].parse().expect("Seed");
        let left: usize = fields[1].parse().expect("Left length");
        let right: usize = fields[2].parse().expect("Right length");
        let left: Vec<_> = (0..left)
            .map(|index| b"Ab_1\xff\x80"[(index + seed) % 6])
            .collect();
        let right: Vec<_> = (0..right)
            .map(|index| b"xY-7\xe9"[(index + seed) % 5])
            .collect();
        let left = string_support::decoded(&left, LegacyStringKind::Plain)?;
        let right = string_support::decoded(&right, LegacyStringKind::Plain)?;
        let length = (left.as_bytes().len() + right.as_bytes().len()) as u64;
        let mut heap = StringHeap::new(StringHeapBudget {
            max_live_strings: 3,
            max_payload_bytes: length * 2,
            max_single_string_bytes: length,
            ..StringHeapBudget::default()
        })
        .expect("Inputs plus complete output admitted");
        let left = heap
            .allocate(left.as_bytes(), HeapWorkBudget::default())
            .expect("Owned left")
            .id;
        let right = heap
            .allocate(right.as_bytes(), HeapWorkBudget::default())
            .expect("Owned right")
            .id;
        let result = heap
            .concatenate(left, right, HeapWorkBudget::default())
            .expect("Owned complete result")
            .id;
        let bytes = heap.get(result).expect("Result bytes");
        let checksum = bytes
            .iter()
            .fold(14_695_981_039_346_656_037_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211)
            });
        assert_eq!(bytes.len() as u64, length);
        assert_eq!(
            checksum,
            u64::from_str_radix(fields[5], 16).expect("Expected checksum")
        );
        match fields[3] {
            "0" => {
                assert_eq!(fields[4], fields[5]);
                assert!(length < 8191);
                parity += 1;
            }
            "1" => {
                assert_eq!(fields[4], "~");
                assert!(length >= 8191);
                expansion += 1;
            }
            _ => return Err("Unknown native string outcome".into()),
        }
        let collection = heap
            .reclaim(&[result], HeapWorkBudget::default())
            .expect("Only result remains rooted");
        assert_eq!(collection.freed_strings, 2);
        assert_eq!(collection.freed_payload_bytes, length);
        assert_eq!(heap.usage().live_strings, 1);
        assert_eq!(heap.usage().payload_bytes, length);
        assert_eq!(heap.get(result).expect("Still live").len() as u64, length);
    }
    assert_eq!((parity, expansion), (54, 54));
    Ok(())
}
