// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute field update operators and distinguish repeated versus selected receivers."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;

fn execute(source: &[u8]) -> KernelValue {
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Bound field updates");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Checked field update")
        .value
}

#[test]
fn all_ten_compound_field_operators_and_integer_postfix_boundaries_execute() {
    for (operator, expected) in [
        ("|=", LegacyNumber::Integer(15)),
        ("^=", LegacyNumber::Integer(11)),
        ("&=", LegacyNumber::Integer(4)),
        ("<<=", LegacyNumber::Integer(240)),
        (">>=", LegacyNumber::Integer(0)),
        ("+=", LegacyNumber::Integer(19)),
        ("-=", LegacyNumber::Integer(11)),
        ("*=", LegacyNumber::Integer(60)),
        ("/=", LegacyNumber::Float(3.75)),
        ("%=", LegacyNumber::Integer(3)),
    ] {
        let source =
            format!("main(){{o=spawnstruct();o.field=15;o.field{operator}4;return o.field;}}");
        assert_eq!(execute(source.as_bytes()), KernelValue::Number(expected));
    }
    for (initial, operator, expected) in [
        ("2147483647", "++", i32::MIN),
        ("-2147483648", "--", i32::MAX),
        ("1", "++", 2),
        ("1", "--", 0),
    ] {
        let source = format!(
            "main(){{o=spawnstruct();o.field={initial};o.field{operator};return o.field;}}"
        );
        assert_eq!(
            execute(source.as_bytes()),
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
    assert_eq!(
        execute(b"main(){o=spawnstruct();o.v=(1,2,3);o.v+=(4,5,6);return o.v[2];}"),
        KernelValue::Number(LegacyNumber::Float(9.0))
    );
}

#[test]
fn compound_evaluates_receiver_before_rhs_and_again_for_write_while_postfix_selects_once() {
    let source = b"main(){s=spawnstruct();s.calls=0;s.trace=0;s.old=spawnstruct();s.new=spawnstruct();s.old.v=10;s.new.v=100;target(s).v+=rhs(s);return s.trace*10000+s.old.v*100+s.new.v;}target(s){s.calls++;s.trace=s.trace*10+1;if(s.calls==1){return s.old;}return s.new;}rhs(s){s.trace=s.trace*10+2;return 5;}";
    assert_eq!(
        execute(source),
        KernelValue::Number(LegacyNumber::Integer(1_211_015))
    );
    for (operator, expected) in [("++", 11_200), ("--", 11_000)] {
        let source = format!(
            "main(){{s=spawnstruct();s.calls=0;s.old=spawnstruct();s.new=spawnstruct();s.old.v=10;s.new.v=100;target(s).v{operator};return s.calls*10000+s.old.v*100+s.new.v;}}target(s){{s.calls++;if(s.calls==1){{return s.old;}}return s.new;}}"
        );
        assert_eq!(
            execute(source.as_bytes()),
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
}

#[test]
fn compound_string_fields_survive_pressure_with_old_values_and_cached_localized_roots() {
    let source = b"main(){r=spawnstruct();r.text=\"\";r.anchor=&\"MENU\";i=0;while(i<1000){r.text+=\"x\";r.count=i;i++;}return r.text.size+isdefined(r.anchor);}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("String compound source");
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 5,
        max_slots: 5,
        max_payload_bytes: 4096,
        ..StringHeapBudget::default()
    })
    .expect("Five string slots");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Rooted field concatenation")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1001))
    );
    assert!(context.usage().collections > 900);
    assert_eq!(context.heap_usage().slots, 5);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Rooted struct")
            .live_fields,
        3
    );
}

#[test]
fn compound_old_string_operand_survives_rhs_deletion_and_collection_before_write()
-> Result<(), String> {
    let source = b"main(){o=spawnstruct();o.text=\"h\"+\"eld\";o.text+=clear_and_make(o);return o.text;}clear_and_make(o){o.text=undefined;i=0;while(i<100){tmp=\"x\"+\"y\";i++;}return \"tail\";}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("RHS deletes the read field");
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 8,
        max_slots: 8,
        ..StringHeapBudget::default()
    })
    .expect("Bounded operand roots");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Old field value survives caller stack collection");
    let KernelValue::String(id) = result.value else {
        return Err("Expected concatenated field bytes".into());
    };
    assert_eq!(context.get(id).expect("Owned returned bytes"), b"heldtail");
    assert!(context.usage().collections > 20);
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Restored field")
            .live_fields,
        1
    );
    Ok(())
}
