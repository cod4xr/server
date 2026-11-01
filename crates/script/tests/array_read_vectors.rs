// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests replay selected native read results with explicit identity normalization and count every helper-only row."
)]
use crate::array_read_support::{self, State};
use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelInstruction,
    KernelScriptRoot, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

const FIXTURES: &str = include_str!("fixtures/array_read_vectors.txt");

fn decimal(value: &str) -> u32 {
    value.parse().expect("Fixture decimal")
}
fn hex(value: &str) -> u32 {
    u32::from_str_radix(value, 16).expect("Fixture word")
}
fn proof(instruction: KernelInstruction) -> VerifiedKernelProgram {
    let mut code = Vec::new();
    if matches!(instruction, KernelInstruction::Index) {
        code.push(KernelInstruction::LoadLocal(0));
    }
    code.extend([
        KernelInstruction::LoadScriptRoot(KernelScriptRoot::Game),
        instruction,
        KernelInstruction::Return,
    ]);
    VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Read proof")
}

fn expected(state: &State, tag: u32, word: u32, subtype: u32) -> Result<KernelValue, String> {
    Ok(match tag {
        0 => KernelValue::Undefined,
        1 if subtype == 18 && word == 2 => KernelValue::Object(state.shared),
        2 if word == 101 => KernelValue::String(state.plain),
        3 if word == 102 => KernelValue::LocalizedString(state.localized),
        5 => KernelValue::Number(LegacyNumber::Float(f32::from_bits(word))),
        6 => array_read_support::number(array_read_support::signed(word)),
        _ => return Err("Unexpected supported native result".into()),
    })
}

#[test]
fn native_read_results_presence_order_and_host_retention_match_all_current_profile_rows()
-> Result<(), String> {
    let proof = proof(KernelInstruction::Index);
    let (mut replayed, mut helper) = (0, 0);
    for line in FIXTURES.lines().filter(|line| line.starts_with("R ")) {
        let row: Vec<_> = line.split_whitespace().collect();
        let profile = decimal(row[1]);
        if profile != 0 {
            match profile {
                1 => {
                    assert_eq!(decimal(row[7]), 7);
                    assert_eq!(hex(row[8]), 0x1234);
                }
                2 => {
                    assert_eq!(decimal(row[7]), 1);
                    assert_eq!(decimal(row[9]), 21);
                }
                _ => return Err("Unknown helper-only read profile".into()),
            }
            helper += 1;
            continue;
        }
        let aliases = u64::from(decimal(row[3]));
        let mut state = array_read_support::state(decimal(row[2]) as usize, aliases);
        let key_tag = decimal(row[4]);
        let key_word = hex(row[5]);
        let key = state.key_value(key_tag, key_word)?;
        let lookup = if key_tag == 6 {
            ArrayKey::Index(u64::from(key_word))
        } else {
            ArrayKey::Bytes(
                array_read_support::byte_key(key_word).ok_or("Unknown native string key")?,
            )
        };
        let presence = state
            .objects
            .array_get(
                state.data,
                lookup,
                &state.strings,
                HeapWorkBudget::default(),
            )
            .expect("Presence")
            .value
            .is_some();
        assert_eq!(presence, decimal(row[6]) != 0);
        let value = expected(&state, decimal(row[7]), hex(row[8]), decimal(row[9]))?;
        let expected_keys: Vec<_> = row[14..].iter().map(|key| (*key).to_owned()).collect();
        assert_eq!(state.keys(), expected_keys);
        let outcome = proof
            .execute_with_heap(
                &[key],
                KernelExecutionBudget::default(),
                &mut state.context(HeapWorkBudget::default()),
            )
            .expect("Selected native read");
        if let (
            KernelValue::Number(LegacyNumber::Float(actual)),
            KernelValue::Number(LegacyNumber::Float(expected)),
        ) = (outcome.value, value)
        {
            assert_eq!(actual.to_bits(), expected.to_bits());
        } else {
            assert_eq!(outcome.value, value);
        }
        assert_eq!(
            decimal(row[10]),
            u32::from(matches!(
                value,
                KernelValue::String(_) | KernelValue::LocalizedString(_)
            ))
        );
        assert_eq!(decimal(row[11]), 0);
        assert_eq!(decimal(row[12]), u32::from(key_tag == 2));
        assert_eq!(decimal(row[13]), 9);
        state.assert_host_owners(aliases);
        assert_eq!(state.keys(), expected_keys);
        replayed += 1;
    }
    assert_eq!((replayed, helper), (66, 12));
    Ok(())
}

#[test]
fn native_size_rows_preserve_raw_undefined_counts_and_consume_only_the_vm_operand() {
    let proof = proof(KernelInstruction::Size);
    let mut count = 0;
    for line in FIXTURES.lines().filter(|line| line.starts_with("Z ")) {
        let row: Vec<_> = line.split_whitespace().collect();
        let aliases = u64::from(decimal(row[2]));
        let mut state = array_read_support::state(decimal(row[1]) as usize, aliases);
        let result = proof
            .execute_with_heap(
                &[KernelValue::Undefined],
                KernelExecutionBudget::default(),
                &mut state.context(HeapWorkBudget::default()),
            )
            .expect("Native array size");
        assert_eq!(
            result.value,
            array_read_support::number(array_read_support::signed(decimal(row[3])))
        );
        assert_eq!(decimal(row[4]), 1);
        state.assert_host_owners(aliases);
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn native_key_error_classes_match_supported_tags_and_other_native_categories_stay_explicit()
-> Result<(), String> {
    let program = proof(KernelInstruction::Index);
    let module =
        kernel_call_support::verify(&[definition(&[integer(1), KernelInstruction::Return], 0, 0)]);
    let function = module.function_reference(0).expect("Function key");
    let (mut replayed, mut helper) = (0, 0);
    for line in FIXTURES.lines().filter(|line| line.starts_with("E ")) {
        let row: Vec<_> = line.split_whitespace().collect();
        let tag = decimal(row[2]);
        if decimal(row[1]) == 18 {
            assert_eq!(
                (
                    tag,
                    hex(row[3]),
                    decimal(row[4]),
                    decimal(row[5]),
                    decimal(row[6])
                ),
                (6, 0, 3, 1, 0)
            );
            helper += 1;
            continue;
        }
        if decimal(row[1]) != 21 {
            return Err("Unknown native receiver subtype".into());
        }
        if matches!(tag, 7 | 8 | 10 | 11 | 12 | 13) {
            assert_eq!(
                (decimal(row[4]), decimal(row[5]), decimal(row[6])),
                (2, 0, 0)
            );
            helper += 1;
            continue;
        }
        let mut state = array_read_support::state(0, 1);
        let key = match tag {
            0 => KernelValue::Undefined,
            1 => KernelValue::Object(state.shared),
            3 => KernelValue::LocalizedString(state.localized),
            4 => KernelValue::Vector(LegacyVector::from_numbers([LegacyNumber::Integer(1); 3])),
            5 => KernelValue::Number(LegacyNumber::Float(f32::from_bits(hex(row[3])))),
            6 => array_read_support::number(array_read_support::signed(hex(row[3]))),
            9 => KernelValue::Function(function),
            _ => return Err("Unexpected supported key category".into()),
        };
        let error = program
            .execute_with_heap(
                &[key],
                KernelExecutionBudget::default(),
                &mut state.context(HeapWorkBudget::default()),
            )
            .expect_err("Strict native key error");
        assert_eq!(
            error.kind,
            match decimal(row[4]) {
                1 => KernelErrorKind::ArrayIndexOutOfRange,
                2 => KernelErrorKind::ExpectedArrayIndex,
                _ => return Err("Unexpected supported native error".into()),
            }
        );
        assert_eq!((decimal(row[5]), decimal(row[6])), (0, 0));
        state.assert_host_owners(1);
        replayed += 1;
    }
    assert_eq!((replayed, helper), (9, 7));
    Ok(())
}
