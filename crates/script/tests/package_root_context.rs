// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests explicit root bindings remain invocation-wide across qualified and included package calls."
)]
use crate::package_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ScriptPackageBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn supplied_package_roots_follow_the_host_through_qualified_pointers_and_imported_calls() {
    let package = package_compile_support::compile(&[
        (b"root", b"#include lib; main(){level.total=1;f=lib::make;owned=[[f]]();return receive(owned);}"),
        (b"lib", b"make(){self.flag=7;return game;}receive(value){return level.total+self.flag+isdefined(value)+anim.size;}"),
    ], LegacyDeveloperMode::Disabled, ScriptPackageBudget::default()).expect("Owned root-aware package");
    let mut state = script_root_support::state(ObjectHeapBudget {
        max_array_values: 4,
        max_array_value_slots: 4,
        ..ObjectHeapBudget::default()
    });
    let roots = state.roots();
    let mut context = KernelHeapContext::new(&mut state.strings, &[], HeapWorkBudget::default())
        .with_objects(&mut state.objects, &[])
        .with_script_roots(roots);
    let entry = package.function_for_declaration(0, 0).expect("Entry");
    let result = package
        .program()
        .execute_with_heap(entry, &[], KernelCallBudget::default(), &mut context)
        .expect("Same root view across file scopes");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(10)));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    drop(context);
    assert_eq!(state.objects.field_count(state.level), Ok(1));
    assert_eq!(state.objects.field_count(state.receiver), Ok(1));
    assert_eq!(state.objects.field_count(state.animation), Ok(0));
}
