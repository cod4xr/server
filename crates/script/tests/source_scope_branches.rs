// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::source_scope_support::{retains, succeeds};

#[test]
fn one_sided_and_disjoint_branch_aliases_end_before_the_next_constructor() {
    for source in [
        b"main(){a=[];if(1){b=a;}c=[];return 0;}".as_slice(),
        b"main(){a=[];if(0){b=a;}c=[];return 0;}",
        b"main(){a=[];if(1){b=a;}else{d=a;}c=[];return 0;}",
        b"main(){a=[];if(0){b=a;}else{d=a;}c=[];return 0;}",
        b"main(){a=[];if(1){b=a;}c=[];b=undefined;return 0;}",
    ] {
        assert_eq!(succeeds(source, 2).objects.array_value_slots, 2);
    }
}

#[test]
fn ordinary_blocks_common_branches_and_existing_outer_cells_keep_their_aliases() {
    for source in [
        b"main(){a=[];{b=a;}c=[];return 0;}".as_slice(),
        b"main(){a=[];b=undefined;if(1){b=a;}c=[];return 0;}",
        b"main(){a=[];if(1){b=a;}else{b=a;}c=[];return 0;}",
        b"main(){a=[];if(0){b=a;}else{b=a;}c=[];return 0;}",
    ] {
        retains(source, 2);
    }
}

#[test]
fn returning_branches_do_not_remove_the_other_paths_public_cells() {
    succeeds(
        b"main(){a=[];if(1){b=a;return 0;}else{c=a;}d=[];return 0;}",
        2,
    );
    retains(
        b"main(){a=[];if(0){b=a;return 0;}else{c=a;}d=[];return 0;}",
        2,
    );
    retains(
        b"main(){a=[];if(1){c=a;}else{b=a;return 0;}d=[];return 0;}",
        2,
    );
}
