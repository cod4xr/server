// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual `Scr_EvalArray` vector reads run through helper/kernel, without native cleanup claims.
use crate::kernel_vector_support::KernelEvaluator;
use crate::vector_support;
use crate::vector_support::evaluator::HelperEvaluator;

#[test]
fn original_vector_index_outcomes_match_helpers_and_verified_kernel_execution() -> Result<(), String>
{
    let mut helper = 0;
    let mut kernel = 0;
    let mut successes = 0;
    for (index, line) in include_str!("fixtures/vector_index_vectors.txt")
        .lines()
        .enumerate()
    {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        successes += u64::from(
            vector_support::validate_index::<KernelEvaluator>(line)
                .map_err(|error| format!("Kernel index line {}: {error}", index + 1))?,
        );
        kernel += 1;
        // The standalone numeric-index API cannot represent a vector index operand.
        if line.split_whitespace().nth(4) != Some("V") {
            vector_support::validate_index::<HelperEvaluator>(line)
                .map_err(|error| format!("Helper index line {}: {error}", index + 1))?;
            helper += 1;
        }
    }
    assert_eq!((helper, kernel, successes), (286, 299, 39));
    Ok(())
}
