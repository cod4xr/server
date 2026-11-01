// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare VM copies, overwrites and permutations with an independent cell-owner count model."
)]
use crate::kernel_call_support::definition;
use cod4x_script::execution::{
    KernelExecutionBudget, KernelHeapContext, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

struct Model {
    code: Vec<KernelInstruction>,
    cells: [bool; 8],
    peak: u64,
}

impl Model {
    fn count(&self) -> u64 {
        self.cells.iter().map(|value| u64::from(*value)).sum()
    }

    fn copy(&mut self, source: usize, target: usize) {
        self.code.extend([
            KernelInstruction::LoadLocal(source),
            KernelInstruction::StoreLocal(target),
        ]);
        self.peak = self.peak.max(self.count() + u64::from(self.cells[source]));
        self.cells[target] = self.cells[source];
    }
}

fn generated(state: &mut u64, returned: usize) -> Model {
    let mut model = Model {
        code: Vec::new(),
        cells: [false; 8],
        peak: 2,
    };
    model.cells[..2].fill(true);
    for target in 2..8 {
        model.copy(target % 2, target);
    }
    for _ in 0..48 {
        let source = (next(state) % 8) as usize;
        let target = 2 + (next(state) % 6) as usize;
        match next(state) % 3 {
            0 => model.copy(source, target),
            1 => {
                model.code.extend([
                    KernelInstruction::Constant(KernelValue::Undefined),
                    KernelInstruction::StoreLocal(target),
                ]);
                model.cells[target] = false;
            }
            _ => {
                model.code.extend([
                    KernelInstruction::LoadLocal(source),
                    KernelInstruction::Duplicate,
                    KernelInstruction::Swap,
                    KernelInstruction::Discard,
                    KernelInstruction::Discard,
                ]);
                model.peak = model
                    .peak
                    .max(model.count() + 2 * u64::from(model.cells[source]));
            }
        }
    }
    model.code.extend([
        KernelInstruction::LoadLocal(returned),
        KernelInstruction::Return,
    ]);
    model.peak = model.peak.max(model.count() + 1);
    model
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn independent_cell_model_predicts_exact_live_admission_return_identity_and_complete_cleanup()
-> Result<(), String> {
    let mut state = 0xc389_4523_0012_abcd;
    for iteration in 0..512 {
        let returned = iteration % 2;
        let model = generated(&mut state, returned);
        let proof = VerifiedKernelProgram::verify_definition(
            definition(&model.code, 2, 8),
            KernelVerificationBudget::default(),
        )
        .expect("Finite transport program");
        for maximum in [model.peak + 1, model.peak + 2] {
            let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
            let mut objects = ObjectHeap::new(
                &strings,
                ObjectHeapBudget {
                    max_array_values: maximum,
                    max_array_value_slots: maximum,
                    ..ObjectHeapBudget::default()
                },
            )
            .expect("Exact model quota");
            let first = objects
                .new_array_value(HeapWorkBudget::default())
                .expect("First host")
                .value;
            let second = objects
                .new_array_value(HeapWorkBudget::default())
                .expect("Second host")
                .value;
            let hosts = [first, second];
            let expected = objects
                .array_data(hosts[returned], HeapWorkBudget::default())
                .expect("Expected data")
                .value;
            let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
                .with_objects(&mut objects, &[]);
            let result = proof.execute_with_heap(
                &hosts.map(KernelValue::Array),
                KernelExecutionBudget::default(),
                &mut context,
            );
            if maximum == model.peak + 1 {
                result.expect_err("One less than independent peak must fail");
                assert_eq!(
                    context
                        .object_heap_usage()
                        .expect("Objects")
                        .live_array_values,
                    2
                );
            } else {
                let outcome = result.expect("Independent exact peak must suffice");
                let KernelValue::Array(id) = outcome.value else {
                    return Err("Expected owned array result".into());
                };
                assert_eq!(
                    context
                        .object_heap_usage()
                        .expect("Objects")
                        .live_array_values,
                    3
                );
                assert_eq!(
                    context
                        .object_heap_usage()
                        .expect("Objects")
                        .array_value_slots,
                    maximum
                );
                drop(context);
                assert_eq!(
                    objects
                        .array_data(id, HeapWorkBudget::default())
                        .expect("Result data")
                        .value,
                    expected
                );
                objects
                    .release_array_value(id, HeapWorkBudget::default())
                    .expect("Release host result");
            }
            assert_eq!(objects.usage().live_array_values, 2);
            for host in hosts {
                let data = objects
                    .array_data(host, HeapWorkBudget::default())
                    .expect("Host data")
                    .value;
                assert_eq!(
                    objects
                        .array_owner_count(data, HeapWorkBudget::default())
                        .expect("Host owner")
                        .value,
                    1
                );
            }
        }
    }
    Ok(())
}
