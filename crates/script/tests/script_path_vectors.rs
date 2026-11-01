// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests replay actual ASCII filename-helper outcomes while rejecting NUL catalog names."
)]
use cod4x_script::binding::{
    LegacyScriptPath, ScriptPathBudget, ScriptPathErrorKind, ScriptPathResource,
};

#[test]
fn native_ascii_filename_outcomes_match_complete_names_and_atomic_capacity_rejection()
-> Result<(), String> {
    let (mut replayed, mut nul_only) = (0, 0);
    for row in include_str!("fixtures/script_path_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let columns: Vec<_> = row.split_whitespace().collect();
        assert_eq!(columns.len(), 5);
        assert_eq!(columns[0], "P");
        let raw = hex(columns[1])?;
        let capacity = columns[2]
            .parse::<usize>()
            .map_err(|error| error.to_string())?;
        let native = hex(columns[4])?;
        if raw.contains(&0) {
            assert_eq!(
                LegacyScriptPath::new(&raw, ScriptPathBudget::default())
                    .expect_err("No silent C-string prefix in catalog IDs")
                    .kind,
                ScriptPathErrorKind::EmbeddedNul
            );
            nul_only += 1;
            continue;
        }
        let path = LegacyScriptPath::new(&raw, ScriptPathBudget::default())
            .expect("Safe ASCII input")
            .value;
        let bytes: Vec<_> = path.canonical_bytes().collect();
        let limit = ScriptPathBudget {
            max_canonical_bytes: (capacity - 1) as u64,
            ..ScriptPathBudget::default()
        };
        if columns[3] == "S" {
            assert_eq!(bytes, native);
            let bounded = LegacyScriptPath::new(&raw, limit).expect("Native complete name fits");
            assert_eq!(
                bounded
                    .value
                    .copy_owned(limit)
                    .expect("Complete owned copy")
                    .value
                    .as_bytes(),
                native
            );
        } else {
            assert_eq!(columns[3], "E");
            assert_eq!(&bytes[..capacity], native);
            for error in [
                LegacyScriptPath::new(&raw, limit).expect_err("No partial view"),
                path.copy_owned(limit).expect_err("No partial owned bytes"),
            ] {
                assert!(matches!(
                    error.kind,
                    ScriptPathErrorKind::BudgetExceeded {
                        resource: ScriptPathResource::CanonicalBytes,
                        ..
                    }
                ));
                assert_eq!(error.usage.retained_bytes, 0);
            }
        }
        replayed += 1;
    }
    assert_eq!((replayed, nul_only), (574, 5));
    Ok(())
}

fn hex(text: &str) -> Result<Vec<u8>, String> {
    if text == "~" {
        return Ok(Vec::new());
    }
    if !text.len().is_multiple_of(2) {
        return Err("Odd fixture hex length".into());
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).map_err(|error| error.to_string())?;
            u8::from_str_radix(digits, 16).map_err(|error| error.to_string())
        })
        .collect()
}
