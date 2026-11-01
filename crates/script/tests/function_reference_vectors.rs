// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual C comparisons with synthetic labels mapped bijectively to opaque references.
#![allow(
    clippy::expect_used,
    reason = "The committed native comparison corpus is structurally valid."
)]

use crate::kernel_call_support::{self, definition};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelModule, VerifiedKernelProgram,
};
use cod4x_script::legacy::{LegacyComparison, LegacyNumber, LegacyVector};
use std::str::SplitWhitespace;

#[test]
fn native_function_identity_comparisons_match_all_opaque_kernel_outcomes() -> Result<(), String> {
    let leaf = [
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let owner = kernel_call_support::verify(&[definition(&leaf, 0, 0); 6]);
    let mut rows = 0;
    let mut successes = 0;
    for line in include_str!("fixtures/function_reference_vectors.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let mut fields = line.split_whitespace();
        assert_eq!(word(&mut fields)?, "R");
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
            KernelInstruction::Constant(value(&mut fields, &owner)?),
            KernelInstruction::Constant(value(&mut fields, &owner)?),
            KernelInstruction::Compare(comparison),
            KernelInstruction::Return,
        ];
        let proof = VerifiedKernelProgram::verify(&code, KernelVerificationBudget::default())
            .expect("Verified comparison");
        match (
            word(&mut fields)?,
            proof.execute(KernelExecutionBudget::default()),
        ) {
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
                    "Function identity comparison disagrees with native corpus: {line}"
                ));
            }
        }
        assert!(fields.next().is_none());
        rows += 1;
    }
    assert_eq!((rows, successes), (2664, 72));
    Ok(())
}

fn value(
    fields: &mut SplitWhitespace<'_>,
    owner: &VerifiedKernelModule,
) -> Result<KernelValue, String> {
    let kind = word(fields)?;
    let mut bits = [0; 3];
    for part in &mut bits {
        *part = u32::from_str_radix(word(fields)?, 16).map_err(|error| error.to_string())?;
    }
    match kind {
        "R" if bits[1..] == [0, 0] => {
            // Labels represent native identity only, never code addresses or Rust indices.
            let index = [0, 1, 2, 0x7fff_ffff, 0x8000_0000, u32::MAX]
                .iter()
                .position(|label| *label == bits[0])
                .ok_or("Unknown synthetic identity")?;
            Ok(KernelValue::Function(
                owner
                    .function_reference(index as u64)
                    .ok_or("Missing reference")?,
            ))
        }
        "U" if bits == [0; 3] => Ok(KernelValue::Undefined),
        "I" if bits[1..] == [0, 0] => Ok(KernelValue::Number(LegacyNumber::Integer(
            bits[0].cast_signed(),
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
