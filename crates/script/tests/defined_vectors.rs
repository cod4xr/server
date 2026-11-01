// SPDX-License-Identifier: AGPL-3.0-or-later
//! Isolated archival comparison instructions plus modeled parameter admission, not a native VM.
#![allow(
    clippy::expect_used,
    reason = "Fixtures have bounded admitted primitive replay sizes."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::kernel_heap_support::allocate;
use cod4x_script::execution::{
    KernelBuiltin, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectId, StringHeap, StringHeapBudget, StringId,
};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn archived_definedness_primitive_subset_and_first_parameter_admission_replay() -> Result<(), String>
{
    let owner =
        kernel_call_support::verify(&[definition(&[integer(0), KernelInstruction::Return], 0, 0)]);
    let function = KernelValue::Function(owner.function_reference(0).expect("Function"));
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let string = allocate(&mut heap, b"");
    let mut objects = ObjectHeap::new(&heap, ObjectHeapBudget::default()).expect("Objects");
    let object = objects
        .allocate(HeapWorkBudget::default())
        .expect("Shared struct")
        .id;
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let (mut rows, mut successes, mut failures, mut helper_only) = (0, 0, 0, 0);
    for row in include_str!("fixtures/defined_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split_whitespace().collect();
        assert_eq!(fields[0], "D");
        let tag = fields[1].parse::<u8>().map_err(|error| error.to_string())?;
        let parent = u32::from_str_radix(fields[2], 16).map_err(|error| error.to_string())?;
        let count = fields[3]
            .parse::<u32>()
            .map_err(|error| error.to_string())?;
        assert!(tag <= 25);
        assert!([0, 1, 2, 3, 32, 255, 256, 65536, u32::MAX].contains(&count));
        assert!(if tag == 1 {
            (14..=25).contains(&parent) || [0x8000_0000, u32::MAX].contains(&parent)
        } else {
            parent == 0
        });
        assert_eq!(fields[4], if count == 0 { "missing" } else { "ok" });
        assert_eq!(fields.len(), if count == 0 { 5 } else { 6 });
        if count != 0 {
            assert!(["0", "1"].contains(&fields[5]));
        }
        rows += 1;
        let Some(value) =
            value(tag, parent, object, string, function).filter(|_| count != u32::MAX)
        else {
            helper_only += 1;
            continue;
        };
        let arguments = usize::try_from(count).expect("Bounded count");
        let mut code = vec![KernelInstruction::LoadLocal(0); arguments];
        code.extend([
            KernelInstruction::Builtin {
                function: KernelBuiltin::IsDefined,
                arguments,
            },
            KernelInstruction::Return,
        ]);
        let program = VerifiedKernelProgram::verify_function(
            &code,
            KernelFunctionLayout::new(1, 1).expect("Local"),
            KernelVerificationBudget::default(),
        )
        .expect("Proof");
        let result =
            program.execute_with_heap(&[value], KernelExecutionBudget::default(), &mut context);
        if count == 0 {
            let error = result.expect_err("Missing first argument");
            assert_eq!(
                error.kind,
                KernelErrorKind::ArgumentCount {
                    expected: 1,
                    actual: 0
                }
            );
            assert_eq!(error.instruction, Some(0));
            assert_eq!(error.work_steps, 1);
            failures += 1;
        } else {
            let expected = fields[5]
                .parse::<i32>()
                .map_err(|error| error.to_string())?;
            let result = result.expect("First argument with optional extras");
            assert_eq!(
                result.value,
                KernelValue::Number(LegacyNumber::Integer(expected)),
                "{row}"
            );
            assert_eq!(result.steps, u64::from(count) + 2);
            successes += 1;
        }
    }
    assert_eq!((rows, successes, failures, helper_only), (351, 56, 8, 287));
    assert_eq!(context.heap_usage().live_strings, 1);
    Ok(())
}

fn value(
    tag: u8,
    parent: u32,
    object: ObjectId,
    string: StringId,
    function: KernelValue,
) -> Option<KernelValue> {
    match tag {
        0 => Some(KernelValue::Undefined),
        1 if parent == 18 => Some(KernelValue::Object(object)),
        2 => Some(KernelValue::String(string)),
        3 => Some(KernelValue::LocalizedString(string)),
        4 => Some(KernelValue::Vector(LegacyVector::from_components([0.0; 3]))),
        5 => Some(KernelValue::Number(LegacyNumber::Float(f32::NAN))),
        6 => Some(KernelValue::Number(LegacyNumber::Integer(0))),
        9 => Some(function),
        _ => None,
    }
}
