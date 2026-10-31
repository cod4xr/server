// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests decode synthetic original-C fixtures."
)]
#![allow(
    dead_code,
    reason = "Integration test binaries use different fixture helpers."
)]

pub fn hex(text: &str) -> Vec<u8> {
    if text == "~" {
        return Vec::new();
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16).expect("Byte")
        })
        .collect()
}

pub fn rows() -> impl Iterator<Item = Vec<&'static str>> {
    include_str!("../fixtures/bsp_vectors.txt")
        .lines()
        .filter(|row| row.starts_with("bsp "))
        .map(|row| row.split_whitespace().collect())
}

pub fn source(version: u32) -> Vec<u8> {
    let row = rows()
        .find(|row| row[1].parse::<u32>().expect("Version") == version)
        .expect("Fixture version");
    hex(row[2])
}

pub fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
