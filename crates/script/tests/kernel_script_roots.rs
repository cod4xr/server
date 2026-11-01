// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests standalone game-read ownership and every insufficient instruction budget."
)]
use crate::script_root_support;
use cod4x_script::execution::{
    KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext, KernelInstruction,
    KernelScriptRoot, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn standalone_game_reads_copy_move_return_and_release_on_every_instruction_failure()
-> Result<(), String> {
    let code = [
        KernelInstruction::LoadScriptRoot(KernelScriptRoot::Game),
        KernelInstruction::Duplicate,
        KernelInstruction::StoreLocal(0),
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(0, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Standalone root proof");
    for maximum in 0..=6 {
        let mut state = script_root_support::state(ObjectHeapBudget {
            max_array_values: 3,
            max_array_value_slots: 3,
            ..ObjectHeapBudget::default()
        });
        let roots = state.roots();
        let data = state
            .objects
            .array_data(state.game, HeapWorkBudget::default())
            .expect("Host data")
            .value;
        let mut context =
            KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
                .with_objects(&mut state.objects, &[])
                .with_script_roots(roots);
        let result = program.execute_with_heap(
            &[],
            KernelExecutionBudget {
                max_steps: maximum,
                ..KernelExecutionBudget::default()
            },
            &mut context,
        );
        if maximum == 6 {
            let result = result.expect("Exact six instructions");
            let KernelValue::Array(returned) = result.value else {
                return Err("Expected owned game result".into());
            };
            assert_eq!(
                context
                    .object_heap_usage()
                    .expect("Objects")
                    .live_array_values,
                2
            );
            drop(context);
            assert_eq!(
                state
                    .objects
                    .array_data(returned, HeapWorkBudget::default())
                    .expect("Result data")
                    .value,
                data
            );
            state
                .objects
                .release_array_value(returned, HeapWorkBudget::default())
                .expect("Host release");
        } else {
            result.expect_err("Every shorter instruction scope");
            assert_eq!(
                context
                    .object_heap_usage()
                    .expect("Objects")
                    .live_array_values,
                1
            );
            drop(context);
        }
        assert_eq!(
            state
                .objects
                .array_owner_count(data, HeapWorkBudget::default())
                .expect("Only original host owner")
                .value,
            1
        );
    }
    Ok(())
}
