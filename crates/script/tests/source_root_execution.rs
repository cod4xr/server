// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare distinct host roots, shared field updates and explicit game-result ownership."
)]
use crate::module_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn all_four_roots_keep_their_categories_through_direct_indirect_calls_and_shared_field_updates() {
    let module = module_compile_support::compile(
        b"main(){level.value=10;self.value=20;anim.value=30;f=::pick;temp=game;
          return [[f]](level,self,anim,temp);}
          pick(l,s,a,g){l.value+=1;s.value+=2;a.value+=3;
          return l.value*10000+s.value*100+a.value+isdefined(g);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Root fields and calls");
    let mut state = script_root_support::state(ObjectHeapBudget::default());
    let roots = state.roots();
    let mut context = KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
        .with_objects(&mut state.objects, &[])
        .with_script_roots(roots);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Explicit script context");
    assert_eq!(
        result.value,
        KernelValue::Number(LegacyNumber::Integer(112_234))
    );
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    drop(context);
    for (root, expected) in [
        (state.level, 11),
        (state.receiver, 22),
        (state.animation, 33),
    ] {
        assert_eq!(
            state
                .objects
                .get(root, b"value", &state.strings, HeapWorkBudget::default())
                .expect("Shared field")
                .value,
            Some(KernelValue::Number(LegacyNumber::Integer(expected)))
        );
    }
}

#[test]
fn owned_image_rebinds_to_distinct_worlds_and_game_results_transfer_one_new_host_owner()
-> Result<(), String> {
    let module = module_compile_support::compile(
        b"getlevel(){return level;}getself(){return self;}getanim(){return anim;}
          getgame(){return relay();}relay(){return game;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Root getters");
    for _ in 0..2 {
        let mut state = script_root_support::state(ObjectHeapBudget::default());
        let roots = state.roots();
        assert_eq!(format!("{roots:?}"), "KernelScriptRoots(..)");
        let expected = [state.level, state.receiver, state.animation];
        let data = state
            .objects
            .array_data(state.game, HeapWorkBudget::default())
            .expect("Game data")
            .value;
        let mut context =
            KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
                .with_objects(&mut state.objects, &[])
                .with_script_roots(roots);
        for (function, root) in expected.into_iter().enumerate() {
            let result = module
                .program()
                .execute_with_heap(
                    function as u64,
                    &[],
                    KernelCallBudget::default(),
                    &mut context,
                )
                .expect("Bound ordinary root");
            assert_eq!(result.value, KernelValue::Object(root));
        }
        let result = module
            .program()
            .execute_with_heap(3, &[], KernelCallBudget::default(), &mut context)
            .expect("Game read through child frame");
        let KernelValue::Array(returned) = result.value else {
            return Err("Expected managed game result".into());
        };
        assert_ne!(returned, state.game);
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
                .expect("Returned data")
                .value,
            data
        );
        state
            .objects
            .release_array_value(returned, HeapWorkBudget::default())
            .expect("Release host result");
        assert_eq!(
            state
                .objects
                .array_owner_count(data, HeapWorkBudget::default())
                .expect("Host game survives")
                .value,
            1
        );
    }
    Ok(())
}
