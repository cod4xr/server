// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests replay native script-index outcomes without treating packed name decoding as script authority."
)]
use crate::kernel_call_support::{definition, verify};
use cod4x_script::execution::{KernelCallBudget, KernelInstruction, KernelValue};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, LegacyArrayKeyError, ObjectHeap, ObjectHeapBudget, StringHeap,
    StringHeapBudget,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn legacy_integer_keys_match_thirteen_native_unsigned_word_outcomes_without_limiting_engine_keys()
-> Result<(), String> {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("String namespace");
    let mut count = 0;
    for row in include_str!("fixtures/array_vectors.txt")
        .lines()
        .filter(|row| row.starts_with("I "))
    {
        let columns: Vec<_> = row.split_whitespace().collect();
        let word = u32::from_str_radix(columns[1], 16).map_err(|error| error.to_string())?;
        let integer = i32::from_ne_bytes(word.to_ne_bytes());
        let result = ArrayKey::from_legacy_value(
            KernelValue::Number(LegacyNumber::Integer(integer)),
            &strings,
        );
        if columns[2] == "1" {
            let ArrayKey::Index(index) = result.expect("Native valid index") else {
                return Err("Expected integer key".into());
            };
            assert_eq!(index, u64::from(word));
            assert_eq!(columns[4], "6");
            assert_eq!(
                u32::from_str_radix(columns[5], 16).map_err(|error| error.to_string())?,
                word
            );
        } else {
            assert_eq!(
                result.expect_err("Native range rejection"),
                LegacyArrayKeyError::IndexOutOfRange
            );
        }
        count += 1;
    }
    assert_eq!(count, 13);
    let ArrayKey::Index(index) = ArrayKey::Index(u64::MAX) else {
        return Err("Wide key constructor".into());
    };
    assert_eq!(index, u64::MAX);
    Ok(())
}

#[test]
fn localized_float_undefined_vector_function_and_object_values_are_not_coerced_to_array_keys() {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let localized = strings
        .allocate(b"PrivateLocalized", HeapWorkBudget::default())
        .expect("Localized payload")
        .id;
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let object = objects
        .allocate(HeapWorkBudget::default())
        .expect("Object")
        .id;
    let code = [
        KernelInstruction::FunctionReference(0),
        KernelInstruction::Return,
    ];
    let module = verify(&[definition(&code, 0, 0)]);
    let function = module
        .execute(0, &[], KernelCallBudget::default())
        .expect("Function snapshot")
        .value;
    for value in [
        KernelValue::Undefined,
        KernelValue::Number(LegacyNumber::Float(1.0)),
        KernelValue::Number(LegacyNumber::Float(f32::NAN)),
        KernelValue::Vector(LegacyVector::from_components([1.0, 2.0, 3.0])),
        KernelValue::LocalizedString(localized),
        KernelValue::Object(object),
        function,
    ] {
        assert_eq!(
            ArrayKey::from_legacy_value(value, &strings).expect_err("Strict key type"),
            LegacyArrayKeyError::ExpectedIndex
        );
    }
}
