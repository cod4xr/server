// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native count/type-only helpers; unrepresentable tags and `UINT32_MAX` stay helper-only.
#![allow(
    clippy::expect_used,
    reason = "Pinned synthetic predicate fixtures have bounded admitted replay sizes."
)]

use crate::kernel_call_support::{self, definition, integer};
use cod4x_script::execution::{
    KernelBuiltin, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn native_predicate_subset_replays_through_verified_heap_backed_instructions() -> Result<(), String>
{
    let leaf = [integer(0), KernelInstruction::Return];
    let module = kernel_call_support::verify(&[definition(&leaf, 0, 0)]);
    let reference = module.function_reference(0).expect("Function value");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("String owner");
    let string = heap
        .allocate(b"type-only fixture", HeapWorkBudget::default())
        .expect("Plain string")
        .id;
    let mut objects = ObjectHeap::new(&heap, ObjectHeapBudget::default()).expect("Objects");
    let object = objects
        .allocate(HeapWorkBudget::default())
        .expect("Ordinary struct")
        .id;
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let mut rows = 0;
    let mut replayed = 0;
    let mut successes = 0;
    let mut helper_only = 0;
    for line in include_str!("fixtures/predicate_vectors.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields[0], "P");
        let function = match fields[1] {
            "0" => KernelBuiltin::IsInt,
            "1" => KernelBuiltin::IsFloat,
            "2" => KernelBuiltin::IsVector,
            "3" => KernelBuiltin::IsString,
            _ => return Err("Unknown predicate fixture".into()),
        };
        let tag: u32 = fields[2]
            .parse()
            .map_err(|error: std::num::ParseIntError| error.to_string())?;
        let count: u32 = fields[3]
            .parse()
            .map_err(|error: std::num::ParseIntError| error.to_string())?;
        assert!([0, 1, 2, 6, 255, 256, 70_000, u32::MAX].contains(&count));
        assert_eq!(fields.len(), if count == 1 { 6 } else { 5 });
        assert_eq!(fields[4], if count == 1 { "ok" } else { "E1" });
        let value = fixture_value(
            tag,
            KernelValue::String(string),
            KernelValue::Object(object),
            KernelValue::Function(reference),
        )?;
        rows += 1;
        // Full native declared counts/tags are retained as helper evidence, not
        // claimed as allocated values or memory-admitted runtime invocations.
        let Some(value) = value.filter(|_| count != u32::MAX) else {
            helper_only += 1;
            continue;
        };
        let arguments = usize::try_from(count).expect("Bounded count");
        let mut code = vec![KernelInstruction::LoadLocal(0); arguments];
        code.extend([
            KernelInstruction::Builtin {
                function,
                arguments,
            },
            KernelInstruction::Return,
        ]);
        let proof = VerifiedKernelProgram::verify_function(
            &code,
            KernelFunctionLayout::new(1, 1).expect("Fixture argument"),
            KernelVerificationBudget::default(),
        )
        .expect("Predicate proof");
        let result =
            proof.execute_with_heap(&[value], KernelExecutionBudget::default(), &mut context);
        if count == 1 {
            let expected = fields[5]
                .parse::<i32>()
                .map_err(|error| error.to_string())?;
            assert!([0, 1].contains(&expected));
            assert_eq!(
                result.expect("Predicate result").value,
                KernelValue::Number(LegacyNumber::Integer(expected)),
                "{line}"
            );
            successes += 1;
        } else {
            let error = result.expect_err("Predicate arity checked before type access");
            assert_eq!(
                error.kind,
                KernelErrorKind::ArgumentCount {
                    expected: 1,
                    actual: u64::from(count)
                },
                "{line}"
            );
            assert_eq!(error.instruction, Some(u64::from(count)));
            assert_eq!(error.work_steps, u64::from(count) + 1);
        }
        replayed += 1;
    }
    assert_eq!(
        (rows, replayed, successes, helper_only),
        (832, 224, 32, 608)
    );
    Ok(())
}

fn fixture_value(
    tag: u32,
    string: KernelValue,
    object: KernelValue,
    function: KernelValue,
) -> Result<Option<KernelValue>, String> {
    Ok(match tag {
        0 => Some(KernelValue::Undefined),
        1 => Some(object),
        2 => Some(string),
        3 => match string {
            KernelValue::String(id) => Some(KernelValue::LocalizedString(id)),
            _ => return Err("Expected fixture string reference".into()),
        },
        4 => Some(KernelValue::Vector(LegacyVector::from_components([0.0; 3]))),
        5 => Some(KernelValue::Number(LegacyNumber::Float(f32::NAN))),
        6 => Some(KernelValue::Number(LegacyNumber::Integer(0))),
        9 => Some(function),
        7..=25 => None,
        _ => return Err("Unknown native enum tag".into()),
    })
}
