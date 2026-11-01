// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests reject every invalid supplied root before entry even if the script does not read it."
)]
use crate::module_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelScriptRoots, KernelValue,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn foreign_and_stale_roots_at_every_position_reject_before_mutating_or_cloning() {
    let module = module_compile_support::compile(
        b"main(){return 1;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Unused roots");
    let work = HeapWorkBudget::default();
    let mut state = script_root_support::state(ObjectHeapBudget::default());
    let other = script_root_support::state(ObjectHeapBudget::default());
    let dead = state.objects.allocate(work).expect("Dead ordinary root").id;
    state
        .objects
        .reclaim(&[state.level, state.receiver, state.animation], work)
        .expect("Keep valid roots only");
    let dead_game = state
        .objects
        .copy_array_value(state.game, work)
        .expect("Dead game location")
        .value;
    state
        .objects
        .release_array_value(dead_game, work)
        .expect("Invalidate game location");
    let mut cases = Vec::new();
    for (id, expected) in [
        (other.level, ObjectHeapErrorKind::ForeignReference),
        (dead, ObjectHeapErrorKind::StaleReference),
    ] {
        cases.extend([
            (
                KernelScriptRoots::new(id, state.receiver, state.game, state.animation),
                expected,
            ),
            (
                KernelScriptRoots::new(state.level, id, state.game, state.animation),
                expected,
            ),
            (
                KernelScriptRoots::new(state.level, state.receiver, state.game, id),
                expected,
            ),
        ]);
    }
    cases.extend([
        (
            KernelScriptRoots::new(state.level, state.receiver, other.game, state.animation),
            ObjectHeapErrorKind::ForeignArrayValue,
        ),
        (
            KernelScriptRoots::new(state.level, state.receiver, dead_game, state.animation),
            ObjectHeapErrorKind::StaleArrayValue,
        ),
    ]);
    let before = state.objects.usage();
    for (roots, expected) in cases {
        let mut context = KernelHeapContext::new(&mut state.strings, &[], work)
            .with_objects(&mut state.objects, &[])
            .with_script_roots(roots);
        let error = module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("All supplied roots are checked");
        assert_eq!(error.cause.kind, KernelErrorKind::ObjectHeap(expected));
        drop(context);
        assert_eq!(state.objects.usage(), before);
    }
    assert_eq!(
        state.objects.field_count(dead),
        Err(ObjectHeapErrorKind::StaleReference)
    );
}

#[test]
fn roots_require_matching_object_storage_and_aliasing_ordinary_bindings_are_valid() {
    let module = module_compile_support::compile(
        b"main(){return level==self;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Root equality");
    let mut state = script_root_support::state(ObjectHeapBudget::default());
    let roots = KernelScriptRoots::new(state.level, state.level, state.game, state.animation);
    let mut context = KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
        .with_script_roots(roots);
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("No object store")
            .cause
            .kind,
        KernelErrorKind::ObjectExecutionUnsupported
    );
    drop(context);
    let mut context = KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
        .with_objects(&mut state.objects, &[])
        .with_script_roots(roots);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Aliased ordinary roots");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(1)));
}
