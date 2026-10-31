// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests decode independent, checked-in fixtures."
)]
#![allow(
    dead_code,
    reason = "Integration test binaries use different fixture helpers."
)]

use cod4x_content::VirtualPath;

pub fn fixture(name: &str) -> Vec<u8> {
    let row = include_str!("../fixtures/archives.txt")
        .lines()
        .filter_map(|line| line.split_once(' '))
        .find(|(label, _)| *label == name)
        .expect("Named independent fixture");
    row.1
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16)
                .expect("Valid hex byte")
        })
        .collect()
}

pub fn path(bytes: &[u8]) -> VirtualPath {
    VirtualPath::try_new(bytes, 4096).expect("Valid test path")
}
