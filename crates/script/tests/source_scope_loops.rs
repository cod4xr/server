// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::source_scope_support::{retains, succeeds};

#[test]
fn dynamic_loop_breaks_remove_private_aliases_and_true_loops_preserve_public_ones() {
    for source in [
        b"main(){a=[];i=0;while(i<1){b=a;break;}c=[];return 0;}".as_slice(),
        b"main(){a=[];for(i=0;i<1;i++){b=a;break;}c=[];return 0;}",
    ] {
        succeeds(source, 2);
    }
    for source in [
        b"main(){a=[];while(1){b=a;break;}c=[];return 0;}".as_slice(),
        b"main(){a=[];for(i=0;;i++){b=a;break;}c=[];return 0;}",
        b"main(){a=[];for(i=0;1;i++){b=a;break;}c=[];return 0;}",
    ] {
        retains(source, 2);
    }
}

#[test]
fn continued_public_aliases_survive_a_later_untaken_branch_and_loop_exit() {
    retains(
        b"main(){a=[];i=0;while(i<2){if(i==0){b=a;i++;continue;}i++;}c=[];return 0;}",
        2,
    );
}

#[test]
fn inherited_public_prefixes_preserve_and_then_clear_displaced_inner_loop_locals() {
    let result = succeeds(
        b"main(){a=[];i=0;while(i<2){if(i==1){b=a;i++;continue;}
        if(i==0){while(i<1){d=a;i++;}}}c=[];return 0;}",
        3,
    );
    assert_eq!(result.objects.array_value_slots, 3);
}

#[test]
fn grouped_numeric_vector_and_string_constant_truth_uses_the_existing_value_rules() {
    for condition in [
        "((1+2)==3)",
        "0.5",
        "true",
        "(1,2,3)==(1,2,3)",
        "undefined==undefined",
        "\"a\"+\"b\"==\"ab\"",
        "&\"a\"==&\"a\"",
    ] {
        let source = format!("main(){{a=[];while({condition}){{b=a;break;}}c=[];return 0;}}");
        retains(source.as_bytes(), 2);
    }
}

#[test]
fn disjoint_break_paths_do_not_become_public_merely_because_the_loop_is_true() {
    succeeds(
        b"main(){a=[];while(1){if(1){b=a;break;}else{d=a;break;}}c=[];return 0;}",
        2,
    );
    succeeds(
        b"main(){a=[];while(1){if(0){b=a;break;}else{d=a;break;}}c=[];return 0;}",
        2,
    );
}
