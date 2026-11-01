// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "String dispatch must preserve existing physical scope and loop-transfer cleanup."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn string_switch_fallthrough_break_and_continue_keep_source_order_and_distinct_targets() {
    let module = compile(
        b"main(){sum=0;i=0;while(i<3){switch(pick(i)){
        case \"zero\":sum+=1;i++;continue;
        case \"one\":sum+=2;default:sum+=4;break;}i++;}return sum;}
        pick(i){if(i==0)return \"zero\";if(i==1)return \"one\";return \"other\";}",
    );
    let mut state = State::new(ObjectHeapBudget::default());
    assert_eq!(
        state
            .run(
                &module,
                &[],
                KernelCallBudget::default(),
                HeapWorkBudget::default()
            )
            .0
            .expect("Continue targets outer loop, break targets switch")
            .value,
        number(11)
    );
}

#[test]
fn string_case_alias_cleanup_controls_observable_later_cow_order() -> Result<(), String> {
    for (arms, copied) in [
        ("case \"x\":b=a;break;", false),
        ("case \"x\":b=a;break;default:d=a;break;", false),
        ("case \"x\":b=a;break;default:b=a;break;", true),
        ("case \"x\":if(1){b=a;break;}default:b=a;break;", false),
    ] {
        let source =
            format!("main(){{a=[];a[0]=1;a[1]=2;switch(\"x\"){{{arms}}}a[2]=3;return a;}}");
        let mut state = State::new(ObjectHeapBudget::default());
        let work = HeapWorkBudget::default();
        let result = state
            .run(
                &compile(source.as_bytes()),
                &[],
                KernelCallBudget::default(),
                work,
            )
            .0
            .expect("Scoped dispatch");
        let KernelValue::Array(returned) = result.value else {
            return Err("Expected owned result".into());
        };
        let data = state
            .objects
            .array_data(returned, work)
            .expect("Data")
            .value;
        let expected = if copied { [2, 0, 1] } else { [2, 1, 0] };
        let keys = state
            .objects
            .array_keys(data, work)
            .expect("Observable native sibling reversal");
        assert_eq!(keys.value.len(), 3);
        assert!(
            keys.value
                .iter()
                .zip(expected)
                .all(|(key, expected)| matches!(key, ArrayKey::Index(index) if *index == expected))
        );
        state
            .objects
            .release_array_value(returned, work)
            .expect("Host releases result");
        assert_eq!(state.objects.usage().live_array_values, 0);
    }
    Ok(())
}
