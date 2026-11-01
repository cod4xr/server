// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual C comparison/arithmetic outcomes; canonical labels map to owned byte identities.
#![allow(
    clippy::expect_used,
    reason = "Tests replay the pinned bounded native string value corpus."
)]
use std::str::SplitWhitespace;

use crate::kernel_call_support::{self, definition};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget, StringId};
use cod4x_script::legacy::{LegacyArithmetic, LegacyComparison, LegacyNumber, LegacyVector};

#[test]
fn native_plain_localized_comparisons_and_localized_arithmetic_match_owned_execution()
-> Result<(), String> {
    let code = [
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::Return,
    ];
    let owner = kernel_call_support::verify(&[definition(&code, 0, 0)]);
    let function = KernelValue::Function(owner.function_reference(0).expect("Opaque function"));
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Owner");
    let strings: Vec<_> = (1..=6)
        .map(|label| {
            heap.allocate(&[label], HeapWorkBudget::default())
                .expect("Canonical label bytes")
                .id
        })
        .collect();
    let mut context = KernelHeapContext::new(&mut heap, &strings, HeapWorkBudget::default());
    let (mut rows, mut successes) = (0, 0);
    for row in include_str!("fixtures/string_value_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let mut fields = row.split_whitespace();
        assert_eq!(word(&mut fields)?, "S");
        let operation = operation(word(&mut fields)?)?;
        let arguments = [
            value(&mut fields, &strings, function)?,
            value(&mut fields, &strings, function)?,
        ];
        let code = [
            KernelInstruction::LoadLocal(0),
            KernelInstruction::LoadLocal(1),
            operation,
            KernelInstruction::Return,
        ];
        let program = VerifiedKernelProgram::verify_function(
            &code,
            KernelFunctionLayout::new(2, 2).expect("Arguments"),
            KernelVerificationBudget::default(),
        )
        .expect("Complete proof");
        let result =
            program.execute_with_heap(&arguments, KernelExecutionBudget::default(), &mut context);
        match (word(&mut fields)?, result) {
            ("ok", Ok(result)) => {
                let expected = match word(&mut fields)? {
                    "0" => 0,
                    "1" => 1,
                    _ => return Err("Invalid boolean".into()),
                };
                assert_eq!(
                    result.value,
                    KernelValue::Number(LegacyNumber::Integer(expected)),
                    "{row}"
                );
                assert_eq!(result.steps, 4);
                successes += 1;
            }
            ("E1", Err(error)) => {
                assert_eq!(error.kind, KernelErrorKind::UnmatchingTypes, "{row}");
                assert_eq!(error.instruction, Some(2));
                assert_eq!(error.work_steps, 3);
            }
            _ => {
                return Err(format!(
                    "String value result disagrees with actual helper: {row}"
                ));
            }
        }
        assert!(fields.next().is_none());
        rows += 1;
    }
    assert_eq!((rows, successes), (3024, 144));
    assert_eq!(context.heap_usage().live_strings, 6);
    Ok(())
}

fn operation(value: &str) -> Result<KernelInstruction, String> {
    Ok(match value {
        "0" => KernelInstruction::Compare(LegacyComparison::Equal),
        "1" => KernelInstruction::Compare(LegacyComparison::NotEqual),
        "2" => KernelInstruction::Compare(LegacyComparison::Less),
        "3" => KernelInstruction::Compare(LegacyComparison::Greater),
        "4" => KernelInstruction::Compare(LegacyComparison::LessOrEqual),
        "5" => KernelInstruction::Compare(LegacyComparison::GreaterOrEqual),
        "6" => KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        "7" => KernelInstruction::Arithmetic(LegacyArithmetic::Subtract),
        "8" => KernelInstruction::Arithmetic(LegacyArithmetic::Multiply),
        "9" => KernelInstruction::Divide,
        _ => return Err("Unknown string value operation".into()),
    })
}

fn value(
    fields: &mut SplitWhitespace<'_>,
    strings: &[StringId],
    function: KernelValue,
) -> Result<KernelValue, String> {
    let kind = word(fields)?;
    let mut bits = [0; 3];
    for part in &mut bits {
        *part = u32::from_str_radix(word(fields)?, 16).map_err(|error| error.to_string())?;
    }
    Ok(match kind {
        "S" | "L" if bits[1..] == [0, 0] => {
            let index = usize::try_from(bits[0])
                .map_err(|error| error.to_string())?
                .checked_sub(1)
                .ok_or("Zero label")?;
            let id = *strings.get(index).ok_or("Unknown canonical label")?;
            if kind == "S" {
                KernelValue::String(id)
            } else {
                KernelValue::LocalizedString(id)
            }
        }
        "U" if bits == [0; 3] => KernelValue::Undefined,
        "R" if bits == [1, 0, 0] => function,
        "I" if bits[1..] == [0, 0] => {
            KernelValue::Number(LegacyNumber::Integer(bits[0].cast_signed()))
        }
        "F" if bits[1..] == [0, 0] => {
            KernelValue::Number(LegacyNumber::Float(f32::from_bits(bits[0])))
        }
        "V" => KernelValue::Vector(LegacyVector::from_components(bits.map(f32::from_bits))),
        _ => return Err("Invalid string value operand".into()),
    })
}

fn word<'a>(fields: &mut SplitWhitespace<'a>) -> Result<&'a str, String> {
    fields
        .next()
        .ok_or_else(|| "Missing native string field".into())
}
