// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Observable sibling order must expose extra or missing source aliases."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn source_scope_cleanup_decides_whether_a_later_write_reverses_existing_siblings()
-> Result<(), String> {
    for (middle, copied) in [
        ("", false),
        ("if(1){b=a;}", false),
        ("if(0){b=a;}", false),
        ("if(1){b=a;}else{d=a;}", false),
        ("b=undefined;if(1){b=a;}", true),
        ("if(1){b=a;}else{b=a;}", true),
        ("i=0;while(i<1){b=a;break;}", false),
        ("while(1){b=a;break;}", true),
        ("for(;;){b=a;break;}", true),
        ("for(i=0;i<1;i++){b=a;break;}", false),
        ("switch(0){case 0:b=a;break;}", false),
        ("switch(0){case 0:b=a;break;default:d=a;break;}", false),
        ("switch(0){case 0:b=a;break;default:b=a;break;}", true),
        (
            "switch(0){case 0:if(1){b=a;break;}default:b=a;break;}",
            false,
        ),
        (
            "switch(0){case 0:if(0){b=a;break;}default:b=a;break;}",
            true,
        ),
    ] {
        let source = format!("main(){{a=[];a[0]=11;a[1]=22;{middle}a[2]=33;return a;}}");
        let module = compile(source.as_bytes());
        let mut state = State::new(ObjectHeapBudget::default());
        let work = HeapWorkBudget::default();
        let result = state
            .run(&module, &[], KernelCallBudget::default(), work)
            .0
            .expect("Scope source");
        let KernelValue::Array(returned) = result.value else {
            return Err("Expected owned array".into());
        };
        let data = state
            .objects
            .array_data(returned, work)
            .expect("Returned data")
            .value;
        assert_eq!(
            state
                .objects
                .array_get(data, ArrayKey::Index(0), &state.strings, work)
                .expect("Preserved entry")
                .value,
            Some(number(11)),
            "{middle}"
        );
        let expected = if copied { [2, 0, 1] } else { [2, 1, 0] };
        let keys = state
            .objects
            .array_keys(data, work)
            .expect("Observable order");
        assert_eq!(keys.value.len(), 3);
        assert!(
            keys.value
                .iter()
                .zip(expected)
                .all(|(key, expected)| matches!(key, ArrayKey::Index(index) if *index == expected)),
            "{middle}"
        );
        assert_eq!(state.objects.usage().live_array_values, 1);
        state
            .objects
            .release_array_value(returned, work)
            .expect("Host releases result");
        assert_eq!(state.objects.usage().live_array_values, 0);
    }
    Ok(())
}

#[test]
fn source_assignment_runs_rhs_then_key_before_selecting_the_writable_location() {
    let module = compile(
        b"main(){a=undefined;o=spawnstruct();o.order=0;
        a[key(o)]=rhs(o);return a[0]+o.order;}
        rhs(o){o.order=1;return 5;}key(o){o.order=o.order*10+2;return 0;}",
    );
    let mut state = State::new(ObjectHeapBudget::default());
    let result = state
        .run(
            &module,
            &[],
            KernelCallBudget::default(),
            HeapWorkBudget::default(),
        )
        .0
        .expect("Ordered source effects");
    assert_eq!(result.value, number(17));
}
