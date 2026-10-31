// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests decode independent checked-in fixtures."
)]

pub fn row(name: &str) -> Vec<&'static str> {
    include_str!("../fixtures/fastfiles.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.split_ascii_whitespace().collect::<Vec<_>>())
        .find(|row| row[0] == name)
        .expect("Named independent fixture")
}

pub fn fixture(name: &str) -> Vec<u8> {
    row(name)[1]
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16)
                .expect("Valid hex")
        })
        .collect()
}
