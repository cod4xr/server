// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests replay pinned native plain-string helper fixtures through typed VM operations."
)]
use crate::kernel_call_support::{self, definition};
use crate::kernel_heap_support::{self, allocate};
use crate::string_support;
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelInstruction, KernelValue,
};
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::legacy::LegacyArithmetic;
use cod4x_script::literal::LegacyStringKind;

#[test]
fn native_plain_concatenation_results_and_capacity_expansion_execute_through_local_calls()
-> Result<(), String> {
    let root = [
        KernelInstruction::LoadLocal(1),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Call {
            function: 1,
            arguments: 2,
        },
        KernelInstruction::Return,
    ];
    let add = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadLocal(1),
        KernelInstruction::Arithmetic(LegacyArithmetic::Add),
        KernelInstruction::Return,
    ];
    let module = kernel_call_support::verify(&[definition(&root, 2, 2), definition(&add, 2, 2)]);
    let (mut parity, mut expansion) = (0, 0);
    for row in include_str!("fixtures/string_operation_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        let seed: usize = fields[0].parse().expect("Seed");
        let left: usize = fields[1].parse().expect("Left length");
        let right: usize = fields[2].parse().expect("Right length");
        let left: Vec<_> = (0..left)
            .map(|index| b"Ab_1\xff\x80"[(index + seed) % 6])
            .collect();
        let right: Vec<_> = (0..right)
            .map(|index| b"xY-7\xe9"[(index + seed) % 5])
            .collect();
        let left = string_support::decoded(&left, LegacyStringKind::Plain)?;
        let right = string_support::decoded(&right, LegacyStringKind::Plain)?;
        let mut heap = kernel_heap_support::heap(3);
        let left_id = allocate(&mut heap, left.as_bytes());
        let right_id = allocate(&mut heap, right.as_bytes());
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        let result = module
            .execute_with_heap(
                0,
                &[KernelValue::String(left_id), KernelValue::String(right_id)],
                KernelCallBudget::default(),
                &mut context,
            )
            .expect("Literal bytes through args/callee/return");
        let KernelValue::String(id) = result.value else {
            return Err("Expected complete owned string".into());
        };
        let bytes = context.get(id).expect("Returned bytes");
        let checksum = bytes
            .iter()
            .fold(14_695_981_039_346_656_037_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211)
            });
        assert_eq!(bytes.len(), left.as_bytes().len() + right.as_bytes().len());
        assert_eq!(
            checksum,
            u64::from_str_radix(fields[5], 16).expect("Expected byte checksum")
        );
        match fields[3] {
            "0" => {
                assert_eq!(fields[4], fields[5]);
                parity += 1;
            }
            "1" => {
                assert_eq!(fields[4], "~");
                expansion += 1;
            }
            _ => return Err("Unknown native outcome".into()),
        }
    }
    assert_eq!((parity, expansion), (54, 54));
    Ok(())
}
