// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual C outcomes involving undefined, replayed through immutable kernel proofs.
#![allow(
    clippy::expect_used,
    reason = "The committed native oracle corpus is structurally valid."
)]

use std::str::SplitWhitespace;

use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyComparison, LegacyNumber, LegacyVector};

#[test]
fn original_undefined_comparisons_match_all_kernel_outcomes() -> Result<(), String> {
    let mut rows = 0;
    let mut successes = 0;
    for line in include_str!("fixtures/undefined_vectors.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let mut fields = line.split_whitespace();
        assert_eq!(word(&mut fields)?, "U");
        let comparison = match word(&mut fields)? {
            "0" => LegacyComparison::Equal,
            "1" => LegacyComparison::NotEqual,
            "2" => LegacyComparison::Less,
            "3" => LegacyComparison::Greater,
            "4" => LegacyComparison::LessOrEqual,
            "5" => LegacyComparison::GreaterOrEqual,
            _ => return Err("Invalid fixture comparison".into()),
        };
        let code = [
            KernelInstruction::Constant(value(&mut fields)?),
            KernelInstruction::Constant(value(&mut fields)?),
            KernelInstruction::Compare(comparison),
            KernelInstruction::Return,
        ];
        let proof = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect("Verified comparison");
        let result = proof.execute(KernelExecutionBudget::default());
        match (word(&mut fields)?, result) {
            ("ok", Ok(outcome)) => {
                let expected = match word(&mut fields)? {
                    "0" => 0,
                    "1" => 1,
                    _ => return Err("Invalid fixture boolean".into()),
                };
                assert_eq!(
                    outcome.value,
                    KernelValue::Number(LegacyNumber::Integer(expected)),
                    "{line}"
                );
                assert_eq!(outcome.steps, 4);
                successes += 1;
            }
            ("E1", Err(error)) => {
                assert_eq!(error.kind, KernelErrorKind::UnmatchingTypes, "{line}");
                assert_eq!(error.instruction, Some(2));
                assert_eq!(error.work_steps, 3);
            }
            _ => {
                return Err(format!(
                    "Kernel comparison disagrees with native corpus: {line}"
                ));
            }
        }
        assert!(fields.next().is_none());
        rows += 1;
    }
    assert_eq!((rows, successes), (402, 2));
    Ok(())
}

fn value(fields: &mut SplitWhitespace<'_>) -> Result<KernelValue, String> {
    let kind = word(fields)?;
    let mut bits = [0; 3];
    for part in &mut bits {
        *part = u32::from_str_radix(word(fields)?, 16).map_err(|error| error.to_string())?;
    }
    match kind {
        "U" if bits == [0; 3] => Ok(KernelValue::Undefined),
        "I" if bits[1..] == [0, 0] => Ok(KernelValue::Number(LegacyNumber::Integer(
            i32::from_ne_bytes(bits[0].to_ne_bytes()),
        ))),
        "F" if bits[1..] == [0, 0] => Ok(KernelValue::Number(LegacyNumber::Float(f32::from_bits(
            bits[0],
        )))),
        "V" => Ok(KernelValue::Vector(LegacyVector::from_components(
            bits.map(f32::from_bits),
        ))),
        _ => Err("Invalid fixture operand".into()),
    }
}

fn word<'a>(fields: &mut SplitWhitespace<'a>) -> Result<&'a str, String> {
    fields.next().ok_or_else(|| "Missing fixture field".into())
}
