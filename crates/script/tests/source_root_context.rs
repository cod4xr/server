// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests require explicit root context only on executed accesses and preserve borrowed roots on errors."
)]
use crate::module_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn missing_context_fails_only_at_executed_roots_without_fabricating_world_objects() {
    let module = module_compile_support::compile(
        b"a(){return level;}b(){return self;}c(){return anim;}d(){return game;}
          skipped(){if(0)return game;return 7;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Root access source");
    let mut state = script_root_support::state(ObjectHeapBudget::default());
    let before = state.objects.usage();
    for entry in 0..4 {
        assert_eq!(
            module
                .program()
                .execute(entry, &[], KernelCallBudget::default())
                .expect_err("Pure root access")
                .cause
                .kind,
            KernelErrorKind::ScriptRootContextRequired
        );
        let mut strings_only =
            KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default());
        assert_eq!(
            module
                .program()
                .execute_with_heap(entry, &[], KernelCallBudget::default(), &mut strings_only)
                .expect_err("No bindings")
                .cause
                .kind,
            KernelErrorKind::ScriptRootContextRequired
        );
        drop(strings_only);
        let mut context =
            KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
                .with_objects(&mut state.objects, &[]);
        assert_eq!(
            module
                .program()
                .execute_with_heap(entry, &[], KernelCallBudget::default(), &mut context)
                .expect_err("Objects are not an implicit world")
                .cause
                .kind,
            KernelErrorKind::ScriptRootContextRequired
        );
    }
    assert_eq!(state.objects.usage(), before);
    assert_eq!(
        module
            .program()
            .execute(4, &[], KernelCallBudget::default())
            .expect("Skipped root access")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
}

#[test]
fn unsupported_game_operations_release_read_aliases_and_do_not_change_the_host_binding() {
    for source in [
        "main(){return game[0];}",
        "main(){return game.size;}",
        "main(){level.private=game;return 1;}",
        "main(){return game+1;}",
    ] {
        let module = module_compile_support::compile(
            source.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Root expressions");
        let mut state = script_root_support::state(ObjectHeapBudget::default());
        let roots = state.roots();
        let data = state
            .objects
            .array_data(state.game, HeapWorkBudget::default())
            .expect("Data")
            .value;
        let mut context =
            KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
                .with_objects(&mut state.objects, &[])
                .with_script_roots(roots);
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("Array operations remain separately gated");
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            1
        );
        drop(context);
        assert_eq!(
            state
                .objects
                .array_data(state.game, HeapWorkBudget::default())
                .expect("Same host binding")
                .value,
            data
        );
        assert_eq!(
            state
                .objects
                .array_owner_count(data, HeapWorkBudget::default())
                .expect("Host owner")
                .value,
            1
        );
    }
}
