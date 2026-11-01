// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile finite primitive integer switch sources."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn integer_switch_matches_cases_default_and_source_order_fallthrough() {
    for (text, input, expected) in [
        (
            "main(n){switch(true){case 1:return 4;default:return 5;}}",
            0,
            4,
        ),
        (
            "main(n){switch(n){case 1:return 11;case 2:return 22;default:return 33;}}",
            1,
            11,
        ),
        (
            "main(n){switch(n){case 1:return 11;case 2:return 22;default:return 33;}}",
            2,
            22,
        ),
        (
            "main(n){switch(n){case 1:return 11;case 2:return 22;default:return 33;}}",
            9,
            33,
        ),
        (
            "main(n){x=1;switch(n){case 1:x+=2;case 2:x*=3;break;default:x=17;}return x;}",
            1,
            9,
        ),
        (
            "main(n){x=1;switch(n){case 1:x+=2;case 2:x*=3;break;default:x=17;}return x;}",
            2,
            3,
        ),
        (
            "main(n){x=1;switch(n){default:x+=2;case 2:x*=3;}return x;}",
            9,
            9,
        ),
        (
            "main(n){x=1;switch(n){default:x+=2;case 2:x*=3;}return x;}",
            2,
            3,
        ),
        (
            "main(n){switch(n){case 1:case 2:return 7;default:return 8;}}",
            1,
            7,
        ),
        (
            "main(n){switch(n){case 1:case 2:return 7;default:return 8;}}",
            2,
            7,
        ),
        ("main(n){switch(n){}return 9;}", 5, 9),
        ("main(n){switch(n){case 1:return 7;}return 9;}", 5, 9),
    ] {
        let module = module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Integer switch");
        let outcome = module
            .program()
            .execute(
                0,
                &[KernelValue::Number(LegacyNumber::Integer(input))],
                KernelCallBudget::default(),
            )
            .expect("Switch result");
        assert_eq!(
            outcome.value,
            KernelValue::Number(LegacyNumber::Integer(expected)),
            "{text}; input {input}"
        );
    }
}

#[test]
fn switch_keys_preserve_the_full_signed_word_as_explicit_capacity_expansion() {
    let text = b"main(n){switch(n){case -2147483648:return 1;case -1:return 2;case 8388608:return 3;case 2147483647:return 4;default:return 5;}}";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Wide key domain, not native encoded variable IDs");
    for (key, expected) in [
        (i32::MIN, 1),
        (-1, 2),
        (8_388_608, 3),
        (i32::MAX, 4),
        (0, 5),
    ] {
        assert_eq!(
            module
                .program()
                .execute(
                    0,
                    &[KernelValue::Number(LegacyNumber::Integer(key))],
                    KernelCallBudget::default()
                )
                .expect("Full word key")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
    }
}
