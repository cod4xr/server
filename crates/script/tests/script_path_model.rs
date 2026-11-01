// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare streaming canonicalization with an independent split/join segment model."
)]
use cod4x_script::binding::{LegacyScriptPath, ScriptPathBudget};

#[test]
fn four_thousand_ninety_six_ascii_names_match_independent_segment_normalization() {
    let mut random = 0x9f27_281c_40a5_10e3_u64;
    let alphabet = b"AaZz09 /\\._-\x01\x0a\x0d\x1f\x7f";
    for _ in 0..4096 {
        let count = usize::try_from(next(&mut random) % 257).expect("Bounded random name");
        let raw: Vec<_> = (0..count)
            .map(|_| {
                alphabet[usize::try_from(next(&mut random) % alphabet.len() as u64)
                    .expect("Alphabet index")]
            })
            .collect();
        let expected = model(&raw);
        let view = LegacyScriptPath::new(&raw, ScriptPathBudget::default())
            .expect("Safe ASCII name")
            .value;
        assert_eq!(view.canonical_bytes().collect::<Vec<_>>(), expected);
        assert_eq!(view.canonical_len(), expected.len() as u64);
        let owned = view
            .copy_owned(ScriptPathBudget::default())
            .expect("Complete independent ownership")
            .value;
        assert_eq!(owned.as_bytes(), expected);
        assert_eq!(owned.as_path(), view);
        assert_eq!(
            owned.as_path().canonical_bytes().collect::<Vec<_>>(),
            expected
        );
    }
}

fn model(raw: &[u8]) -> Vec<u8> {
    let separator = |byte: &u8| matches!(byte, b'/' | b'\\');
    let mut result = Vec::new();
    for segment in raw.split(|byte| *byte < b' ') {
        let mut first = true;
        for part in segment.split(separator).filter(|part| !part.is_empty()) {
            if !first {
                result.push(b'/');
            }
            first = false;
            result.extend(part.iter().map(u8::to_ascii_lowercase));
        }
        if !first && segment.last().is_some_and(separator) {
            result.push(b'/');
        }
    }
    result
}

fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state ^ (*state >> 29)
}
