// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite switch examples exercise structural path initialization."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn switch_joins_actual_exits_and_excludes_returning_and_continuing_paths() {
    for (text, expected) in [
        (
            "main(n){for(i=0;i<2;i++){switch(n){case 1:continue;default:x=7;}return x;}return 3;}",
            [7, 3, 7],
        ),
        (
            "main(n){switch(n){case 1:x=3;break;default:x=7;}return x;}",
            [7, 3, 7],
        ),
        (
            "main(n){switch(n){case 1:x=3;break;case 2:return 9;default:x=7;}return x;}",
            [7, 3, 9],
        ),
        (
            "main(n){switch(n){default:x=7;break;case 1:return 3;case 2:x=9;}return x;}",
            [7, 3, 9],
        ),
        (
            "main(n){switch(n){case 1:if(n){x=3;break;}x=4;break;default:x=7;}return x;}",
            [7, 3, 7],
        ),
        ("main(n){x=1;switch(n){case 1:x=3;}return x;}", [1, 3, 1]),
    ] {
        let module = module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Actual switch exits initialize local");
        for (key, expected) in expected.into_iter().enumerate() {
            let key = i32::try_from(key).expect("Small input");
            assert_eq!(
                module
                    .program()
                    .execute(
                        0,
                        &[KernelValue::Number(LegacyNumber::Integer(key))],
                        KernelCallBudget::default()
                    )
                    .expect("Initialized result")
                    .value,
                KernelValue::Number(LegacyNumber::Integer(expected)),
                "{text}"
            );
        }
    }
}

#[test]
fn unmatched_direct_entry_and_incomplete_exits_cannot_invent_initialization() {
    for text in [
        "main(n){switch(n){case 1:x=3;break;}return x;}",
        "main(n){switch(n){case 1:x=3;default:break;}return x;}",
        "main(n){switch(n){case 1:x=3;case 2:return x;default:return 0;}}",
        "main(n){switch(n){case 1:break;default:x=3;}return x;}",
        "main(n){switch(n){case 1:if(n){x=3;break;}break;default:x=7;}return x;}",
    ] {
        assert_eq!(
            module_compile_support::compile(
                text.as_bytes(),
                LegacyDeveloperMode::Disabled,
                ModuleCompilationBudget::default()
            )
            .expect_err("Some reachable path does not initialize")
            .cause
            .kind,
            FunctionCompileErrorKind::UninitializedLocal,
            "{text}"
        );
    }
}
