// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests keep independent empty origins out of stored fields and restricted compiler profiles."
)]
use crate::array_read_support::number;
use crate::function_compile_support;
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{
    FunctionCompilationBudget, FunctionCompileErrorKind, ModuleCompilationBudget,
};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, StringHeap, StringHeapBudget,
};

#[test]
fn independently_constructed_arrays_cannot_be_stored_in_fields_and_failed_context_reuses_cleanly() {
    let module = module_compile_support::compile(
        b"bad(){o=spawnstruct();o.private=[];return 0;}good(){a=[];return a.size;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Runtime type gate");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    for _ in 0..32 {
        let error = module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("No persistent owner edge");
        assert_eq!(
            error.cause.kind,
            KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::ManagedArrayFieldUnsupported)
        );
        assert!(!format!("{error:?} {error}").contains("private"));
        assert_eq!(
            module
                .program()
                .execute_with_heap(1, &[], KernelCallBudget::default(), &mut context)
                .expect("Recovery")
                .value,
            number(0)
        );
        let usage = context.object_heap_usage().expect("Objects");
        assert_eq!((usage.live_array_values, usage.live_fields), (0, 0));
    }
}

#[test]
fn standalone_constant_validation_profiles_do_not_gain_mutable_array_allocation() {
    let source = b"main(){return [];}";
    assert_eq!(
        function_compile_support::compile_scalar(source, FunctionCompilationBudget::default())
            .expect_err("Scalar constant profile")
            .kind,
        FunctionCompileErrorKind::UnsupportedPrimary
    );
    assert_eq!(
        function_compile_support::compile_primitive(source, FunctionCompilationBudget::default())
            .expect_err("Primitive constant validation profile")
            .kind,
        FunctionCompileErrorKind::UnsupportedPrimary
    );
}
