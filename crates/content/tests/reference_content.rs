// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original C filesystem comparison and keyed archive checksum evidence.
#![allow(
    clippy::expect_used,
    reason = "Tests decode original C oracle vectors."
)]

mod common;

use cod4x_content::{ArchiveBudget, IwdArchive, VirtualPath};

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16).expect("Hex byte")
        })
        .collect()
}

#[test]
fn normalized_byte_paths_and_seeded_crc_match_pinned_original_functions() -> Result<(), String> {
    let mut checksums = 0;
    let mut paths = 0;
    for row in include_str!("fixtures/content_vectors.txt").lines() {
        if row.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = row.split_whitespace().collect();
        match fields.as_slice() {
            ["path", left, right, equal] => {
                let left = VirtualPath::try_new(&bytes(left), 4096).expect("Left");
                let right = VirtualPath::try_new(&bytes(right), 4096).expect("Right");
                assert_eq!(left == right, *equal == "1");
                paths += 1;
            }
            ["checksum", label, feed, expected] => {
                let data = common::fixture(label);
                let archive = IwdArchive::open(&data, ArchiveBudget::default()).expect("Archive");
                let feed = u32::from_str_radix(feed, 16).expect("Feed");
                let expected = u32::from_str_radix(expected, 16).expect("Checksum");
                assert_eq!(archive.legacy_checksum(feed), expected, "{label}, {feed}");
                checksums += 1;
            }
            _ => return Err("Unknown C fixture row".into()),
        }
    }
    assert_eq!(paths, 7);
    assert_eq!(checksums, 25);
    Ok(())
}
