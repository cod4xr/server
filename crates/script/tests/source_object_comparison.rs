// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute source struct identity through aliases, calls, fields, cycles and collection."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn source_identity_distinguishes_aliases_and_equal_content_cycles_through_direct_and_indirect_calls()
 {
    for (source, expected) in [
        (b"main(){a=spawnstruct();a.child=spawnstruct();a.child.me=a.child;b=spawnstruct();b.child=spawnstruct();b.child.me=b.child;alias=a.child;return (alias==a.child)+10*(a.child!=b.child)+100*(a.child==b.child);}".as_slice(), 11),
        (b"main(){a=spawnstruct();alias=identity(a);cmp=::compare;if(a!=alias){return 99;}return [[cmp]](a,alias)+10*compare(spawnstruct(),a);}identity(o){return o;}compare(x,y){return x==y;}".as_slice(), 1),
        (b"main(){root=spawnstruct();root.ref=spawnstruct();root.loop=root;i=0;while(i<1000){trash=spawnstruct();trash.loop=trash;i++;}return root.ref==get(root);}get(root){return root.ref;}".as_slice(), 1),
    ] {
        let module = module_compile_support::compile(source, LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("Source identity lowering");
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget {max_live_objects: 5, max_object_slots: 5, max_live_fields: 6, max_field_slots: 6, ..ObjectHeapBudget::default()}).expect("Bounded shared graphs");
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default()).with_objects(&mut objects, &[]);
        assert_eq!(module.program().execute_with_heap(0, &[], KernelCallBudget::default(), &mut context).expect("Checked identity result").value, KernelValue::Number(LegacyNumber::Integer(expected)));
    }
}

#[test]
fn object_mixed_and_ordered_comparisons_fail_at_original_private_operator_spans()
-> Result<(), String> {
    for (source, token) in [
        (
            b"main(){o=spawnstruct();return o==undefined;}".as_slice(),
            b"==".as_slice(),
        ),
        (
            b"main(){o=spawnstruct();return o!=\"PrivateValue\";}".as_slice(),
            b"!=".as_slice(),
        ),
        (
            b"main(){o=spawnstruct();return o<o;}".as_slice(),
            b"<".as_slice(),
        ),
        (
            b"main(){o=spawnstruct();return 1==o;}".as_slice(),
            b"==".as_slice(),
        ),
    ] {
        let module = module_compile_support::compile(
            source,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Runtime comparison domains");
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        let error = module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("No mixed/object ordering coercion");
        assert_eq!(error.cause.kind, KernelErrorKind::UnmatchingTypes);
        let span = module
            .instruction_span(
                error.function.ok_or("Function")?,
                error.cause.instruction.ok_or("Instruction")?,
            )
            .ok_or("Operator map")?;
        assert_eq!(
            &source[usize::try_from(span.start).expect("Start")
                ..usize::try_from(span.end).expect("End")],
            token
        );
        assert!(!format!("{module:?} {error:?} {context:?}").contains("PrivateValue"));
    }
    Ok(())
}
