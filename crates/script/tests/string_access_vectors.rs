// SPDX-License-Identifier: AGPL-3.0-or-later
//! Replay actual C string access outcomes through verified heap-backed execution.
#![allow(
    clippy::expect_used,
    reason = "Bounded native fixtures use admitted owned handles."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use cod4x_script::execution::{
    KernelError, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelOutcome, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget, StringId};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn native_string_byte_index_and_size_outcomes_match_owned_execution() -> Result<(), String> {
    let owner = kernel_call_support::verify(&[definition(
        &[
            KernelInstruction::Constant(KernelValue::Undefined),
            KernelInstruction::Return,
        ],
        0,
        0,
    )]);
    let function = KernelValue::Function(owner.function_reference(0).expect("Function"));
    let large = vec![b'z'; 70_000];
    let bytes: [&[u8]; 8] = [
        b"",
        b"Rust",
        b"\xff\x80A",
        b"\xc3\xa9",
        b"before\0after",
        b"\0tail",
        b"abc\0",
        &large,
    ];
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let roots: Vec<_> = bytes
        .into_iter()
        .map(|bytes| allocate(&mut heap, bytes))
        .collect();
    let index_program = kernel_heap_support::binary(KernelInstruction::Index);
    let size_program = VerifiedKernelProgram::verify_function(
        &[
            KernelInstruction::LoadLocal(0),
            KernelInstruction::Size,
            KernelInstruction::Return,
        ],
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Unary proof");
    let mut context = KernelHeapContext::new(&mut heap, &roots, HeapWorkBudget::default());
    let (mut rows, mut successes) = (0, 0);
    for row in include_str!("fixtures/string_access_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_whitespace().collect();
        let case = fields[1]
            .parse::<usize>()
            .map_err(|error| error.to_string())?;
        let string = *roots
            .get(case.checked_sub(1).ok_or("Zero case")?)
            .ok_or("Unknown case")?;
        let kind = fields[2].parse::<u8>().map_err(|error| error.to_string())?;
        let (program, arguments, status_field, operation_pc) = match fields[0] {
            "R" => {
                let bits = u32::from_str_radix(fields[3], 16).map_err(|error| error.to_string())?;
                (
                    &index_program,
                    vec![
                        value(kind, bits, string, function)?,
                        KernelValue::String(string),
                    ],
                    4,
                    2,
                )
            }
            "Z" => (
                &size_program,
                vec![value(
                    kind,
                    u32::try_from(case).expect("Case"),
                    string,
                    function,
                )?],
                3,
                1,
            ),
            _ => return Err("Unknown native string access operation".into()),
        };
        let result =
            program.execute_with_heap(&arguments, KernelExecutionBudget::default(), &mut context);
        successes += usize::from(check_result(
            &fields,
            status_field,
            operation_pc,
            result,
            &context,
            row,
        )?);
        rows += 1;
    }
    assert_eq!(rows, 896);
    assert_eq!(successes, 36);
    Ok(())
}

fn check_result(
    fields: &[&str],
    status_field: usize,
    operation_pc: u64,
    result: Result<KernelOutcome, KernelError>,
    context: &KernelHeapContext<'_>,
    row: &str,
) -> Result<bool, String> {
    match (fields[status_field], result) {
        ("0", Ok(result)) => {
            let expected = fields[status_field + 1];
            if fields[0] == "R" {
                let KernelValue::String(id) = result.value else {
                    return Err("Lost string tag".into());
                };
                let byte = u8::from_str_radix(expected, 16).map_err(|error| error.to_string())?;
                assert_eq!(context.get(id).expect("Returned byte"), &[byte], "{row}");
            } else {
                let length = expected.parse::<i32>().map_err(|error| error.to_string())?;
                assert_eq!(
                    result.value,
                    KernelValue::Number(LegacyNumber::Integer(length)),
                    "{row}"
                );
            }
            assert_eq!(fields.len(), status_field + 2);
            Ok(true)
        }
        (status, Err(error)) => {
            let expected = match status {
                "1" => KernelErrorKind::StringIndexOutOfRange,
                "2" => KernelErrorKind::ExpectedStringIndex,
                "3" => KernelErrorKind::ExpectedSized,
                _ => return Err("Unknown native error category".into()),
            };
            assert_eq!(error.kind, expected, "{row}");
            assert_eq!(error.instruction, Some(operation_pc));
            assert_eq!(fields.len(), status_field + 1);
            Ok(false)
        }
        _ => Err(format!("Native string access disagrees: {row}")),
    }
}

fn value(
    kind: u8,
    bits: u32,
    string: StringId,
    function: KernelValue,
) -> Result<KernelValue, String> {
    Ok(match kind {
        0 => KernelValue::Undefined,
        2 => KernelValue::String(string),
        3 => KernelValue::LocalizedString(string),
        4 => KernelValue::Vector(LegacyVector::from_components([0.0; 3])),
        5 => KernelValue::Number(LegacyNumber::Float(f32::from_bits(bits))),
        6 => KernelValue::Number(LegacyNumber::Integer(bits.cast_signed())),
        9 => function,
        _ => return Err("Unknown primitive fixture tag".into()),
    })
}
