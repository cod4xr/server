// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::source_scope_support::{retains, succeeds};

#[test]
fn a_partial_nested_break_clears_its_private_alias_before_the_switch_exit() {
    succeeds(
        b"main(){a=[];switch(0){case 0:if(1){b=a;break;}default:b=a;break;}c=[];return 0;}",
        2,
    );
    retains(
        b"main(){a=[];switch(0){case 0:if(0){b=a;break;}default:b=a;break;}c=[];return 0;}",
        2,
    );
}

#[test]
fn a_cleared_partial_break_binding_cannot_be_read_at_the_switch_join() {
    let error = crate::module_compile_support::compile(
        b"main(){a=[];switch(0){case 0:if(1){b=a;break;}default:b=a;break;}return b;}",
        cod4x_script::binding::LegacyDeveloperMode::Disabled,
        cod4x_script::compilation::ModuleCompilationBudget::default(),
    );
    assert!(matches!(error, Err(error) if error.cause.kind ==
        cod4x_script::compilation::FunctionCompileErrorKind::UninitializedLocal));
}

#[test]
fn switch_fallthrough_cleans_private_aliases_before_the_next_case_body() {
    succeeds(
        b"main(){a=[];switch(0){case 0:b=a;default:c=[];break;}return 0;}",
        2,
    );
    succeeds(
        b"main(){a=[];switch(1){case 0:b=a;default:c=[];break;}return 0;}",
        2,
    );
}

#[test]
fn missing_and_disjoint_defaults_remove_private_aliases_but_common_cells_survive() {
    succeeds(
        b"main(){a=[];switch(0){case 0:b=a;break;}c=[];return 0;}",
        2,
    );
    succeeds(
        b"main(){a=[];switch(0){case 0:b=a;break;default:d=a;break;}c=[];return 0;}",
        2,
    );
    succeeds(
        b"main(){a=[];switch(1){case 0:b=a;break;default:d=a;break;}c=[];return 0;}",
        2,
    );
    retains(
        b"main(){a=[];switch(0){case 0:b=a;break;default:b=a;break;}c=[];return 0;}",
        2,
    );
}

#[test]
fn nested_break_branches_join_the_case_parent_before_the_switch_join() {
    retains(
        b"main(){a=[];switch(0){case 0:if(1){b=a;break;}else{b=a;break;}
        default:b=a;break;}c=[];return 0;}",
        2,
    );
}

#[test]
fn switch_break_and_outer_loop_continue_keep_distinct_targets_and_public_cells() {
    let source = b"main(){a=[];i=0;while(i<2){switch(i){case 0:b=a;i=1;continue;
        default:d=a;break;}i++;}c=[];return 0;}";
    succeeds(source, 4);
    retains(source, 3);
}
