// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual source helpers with stubbed services, not native VM/refcount/error parity.
use crate::vector_support;

#[test]
fn original_vector_helpers_match_all_recorded_construction_coercion_and_operator_cases()
-> Result<(), String> {
    let mut counts = [0_u64; 4];
    for (index, line) in include_str!("fixtures/vector_vectors.txt")
        .lines()
        .enumerate()
    {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let kind = vector_support::validate(line)
            .map_err(|error| format!("Vector case line {}: {error}", index + 1))?;
        let slot = match kind {
            "V" => 0,
            "P" => 1,
            "A" => 2,
            "C" => 3,
            _ => return Err("Unknown validated row".into()),
        };
        counts[slot] += 1;
    }
    assert_eq!(counts, [756, 1736, 6944, 10416]);
    assert_eq!(counts.iter().sum::<u64>(), 19_852);
    Ok(())
}
