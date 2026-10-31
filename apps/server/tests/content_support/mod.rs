// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests own exact synthetic fixtures and files."
)]
#![allow(
    dead_code,
    reason = "Integration binaries exercise different content formats."
)]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct FixtureFile(pub PathBuf);

impl FixtureFile {
    pub fn iwd(label: &str) -> Self {
        Self::from_dataset(
            label,
            include_str!("../../../../crates/content/tests/fixtures/archives.txt"),
        )
    }

    pub fn fastfile(label: &str) -> Self {
        Self::from_dataset(
            label,
            include_str!("../../../../crates/content/tests/fixtures/fastfiles.txt"),
        )
    }

    pub fn zone(label: &str) -> Self {
        Self::from_dataset(
            label,
            include_str!("../../../../crates/content/tests/fixtures/zones.txt"),
        )
    }

    pub fn rawfile(label: &str) -> Self {
        Self::from_dataset(
            label,
            include_str!("../../../../crates/content/tests/fixtures/rawfiles.txt"),
        )
    }

    fn from_dataset(label: &str, dataset: &str) -> Self {
        static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "cod4x-secret-content-{}-{}.fixture",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        let row = dataset
            .lines()
            .map(|line| line.split_ascii_whitespace().collect::<Vec<_>>())
            .find(|row| row.first().copied() == Some(label))
            .expect("Independent fixture");
        let bytes: Vec<u8> = row[1]
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16).expect("Byte")
            })
            .collect();
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("Exclusive test-owned file");
        file.write_all(&bytes).expect("Synthetic content");
        Self(path)
    }
}

impl Drop for FixtureFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
