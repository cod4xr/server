// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests check all insufficient constructor work prefixes and scratch admission."
)]
use crate::object_heap_support::constructor;
use cod4x_script::execution::{
    KernelBuiltin, KernelErrorKind, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelResource, KernelValue, KernelVerificationBudget,
    VerifiedKernelProgram,
};
use cod4x_script::heap::HeapWorkBudget;

#[test]
fn constructor_pressure_shares_exact_global_work_scratch_and_failed_operations_preserve_reachable_fields()
 {
    let code = [
        KernelInstruction::Builtin {
            function: KernelBuiltin::SpawnStruct,
            arguments: 0,
        },
        KernelInstruction::Discard,
        KernelInstruction::LoadLocal(0),
        KernelInstruction::Return,
    ];
    let program = VerifiedKernelProgram::verify_function(
        &code,
        KernelFunctionLayout::new(1, 1).expect("Layout"),
        KernelVerificationBudget::default(),
    )
    .expect("Constructor pressure proof");
    let mut state = constructor::pressure();
    let mut context = KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
        .with_objects(&mut state.objects, &[]);
    assert_eq!(
        program
            .execute_with_heap(
                &[KernelValue::Object(state.root)],
                KernelExecutionBudget::default(),
                &mut context
            )
            .expect("Measure")
            .value,
        KernelValue::Object(state.root)
    );
    let usage = context.usage();
    let budgets = (0..usage.steps)
        .map(|max_steps| HeapWorkBudget {
            max_steps,
            ..HeapWorkBudget::default()
        })
        .chain([HeapWorkBudget {
            max_scratch_bytes: usage.peak_scratch_bytes - 1,
            ..HeapWorkBudget::default()
        }]);
    for budget in budgets {
        let mut state = constructor::pressure();
        let before = state.objects.usage();
        let mut context = KernelHeapContext::new(&mut state.strings, &[], budget)
            .with_objects(&mut state.objects, &[]);
        let error = program
            .execute_with_heap(
                &[KernelValue::Object(state.root)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect_err("Incomplete work/scratch");
        assert!(
            matches!(error.kind, KernelErrorKind::BudgetExceeded { resource: KernelResource::HeapWorkSteps | KernelResource::HeapScratchBytes, requested, maximum } if requested > maximum)
        );
        assert!(context.usage().steps <= budget.max_steps);
        assert!(context.usage().peak_scratch_bytes <= budget.max_scratch_bytes);
        assert_eq!(
            context
                .get(state.held)
                .expect("Every reachable payload survives"),
            b"held"
        );
        if context.usage().object_collections == 0 {
            assert_eq!(context.object_heap_usage(), Some(before));
        }
        assert_eq!(state.objects.field_count(state.root), Ok(1));
    }
    let mut state = constructor::pressure();
    let mut context = KernelHeapContext::new(
        &mut state.strings,
        &[],
        HeapWorkBudget {
            max_steps: usage.steps,
            max_scratch_bytes: usage.peak_scratch_bytes,
        },
    )
    .with_objects(&mut state.objects, &[]);
    program
        .execute_with_heap(
            &[KernelValue::Object(state.root)],
            KernelExecutionBudget::default(),
            &mut context,
        )
        .expect("Exact constructor admission");
    assert_eq!(context.usage(), usage);
}
