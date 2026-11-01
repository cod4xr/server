// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Developer transfer regions must not leak discarded patches or aliases into enclosing constructs."
)]
use crate::{
    array_read_support::number, source_array_write_support::State,
    source_developer_support::compile,
};
use cod4x_script::binding::LegacyDeveloperMode::{Disabled, Enabled};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn developer_break_continue_and_nested_local_control_preserve_enclosing_targets_in_both_modes() {
    for (source, enabled, disabled) in [
        (b"main(){s=0;for(i=0;i<3;i++){/# if(i==1){break;}if(i==0){continue;} #/s++;}return s;}".as_slice(), 0, 3),
        (b"main(){s=0;switch(1){case 1:/# break; #/s=42;break;default:s=-1;}return s;}", 0, 42),
        (b"main(){s=0;i=0;while(i<3){i++;/# a=[];continue; #/s++;}return s;}", 0, 3),
        (b"main(){s=1;/# for(i=0;i<3;i++){switch(i){case 1:continue;default:s+=i;}} #/return s;}", 3, 1),
    ] {
        for (mode, expected) in [(Enabled, enabled), (Disabled, disabled)] {
            let mut state = State::new(ObjectHeapBudget::default());
            assert_eq!(state.run(&compile(source, mode), &[], KernelCallBudget::default(), HeapWorkBudget::default()).0.expect("Developer control destinations").value, number(expected));
            assert_eq!(state.objects.usage().live_array_values, 0);
        }
    }
}

#[test]
fn enabled_private_aliases_clear_before_later_array_mutation_without_fake_receiver_copies() {
    for mode in [Enabled, Disabled] {
        let module = compile(
            b"main(){a=[];a[0]=0;a[1]=1;/# b=a; #/a[2]=2;return a;}",
            mode,
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
            .expect("Private alias cleanup");
        let value = match result.value {
            KernelValue::Array(value) => Some(value),
            _ => None,
        }
        .expect("Owned return");
        let work = HeapWorkBudget::default();
        let data = state
            .objects
            .array_data(value, work)
            .expect("Returned data")
            .value;
        assert!(matches!(
            state
                .objects
                .array_keys(data, work)
                .expect("Unique insertion order")
                .value
                .as_slice(),
            [ArrayKey::Index(2), ArrayKey::Index(1), ArrayKey::Index(0)]
        ));
        state
            .objects
            .release_array_value(value, work)
            .expect("Host release");
        assert_eq!(state.objects.usage().live_array_values, 0);
    }
}
