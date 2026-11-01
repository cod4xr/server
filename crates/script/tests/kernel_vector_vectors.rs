// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original vector construction/operators execute through actual owned verified programs.
use crate::kernel_vector_support::KernelEvaluator;
use crate::vector_support;

#[test]
fn original_vector_construction_and_operator_outcomes_match_verified_kernel_execution()
-> Result<(), String> {
    let mut counts = [0_u64; 3];
    let mut coercion_only = 0;
    for (index, line) in include_str!("fixtures/vector_vectors.txt")
        .lines()
        .enumerate()
    {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("P ") {
            coercion_only += 1;
            continue;
        }
        let kind = vector_support::validate_with::<KernelEvaluator>(line)
            .map_err(|error| format!("Kernel vector case line {}: {error}", index + 1))?;
        let slot = match kind {
            "V" => 0,
            "A" => 1,
            "C" => 2,
            _ => return Err("Unexpected kernel corpus row".into()),
        };
        counts[slot] += 1;
    }
    assert_eq!(counts, [756, 6944, 10416]);
    assert_eq!(counts.iter().sum::<u64>(), 18_116);
    assert_eq!(coercion_only, 1736);
    Ok(())
}
