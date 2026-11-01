// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Synthetic nested switches exercise real loop and switch destinations."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelResource, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn breaks_and_continues_target_the_correct_owner_in_every_nesting_direction() {
    for (text, expected) in [
        (
            "main(){x=0;while(x<3){switch(x){case 0:x++;break;default:x++;break;}x+=10;}return x;}",
            11,
        ),
        (
            "main(){x=0;y=0;while(x<4){x++;switch(x){case 1:continue;case 2:break;default:y+=10;}y++;}return y;}",
            23,
        ),
        (
            "main(){x=0;switch(1){case 1:while(x<5){x++;break;}x+=10;break;default:x=99;}return x;}",
            11,
        ),
        (
            "main(){x=0;switch(1){case 1:switch(2){case 2:x=5;break;default:x=99;}x+=10;break;default:x=99;}return x;}",
            15,
        ),
        (
            "main(){x=0;y=0;for(x=0;x<4;x++){switch(x){case 0:continue;case 1:break;default:y+=10;}y++;}return x*100+y;}",
            423,
        ),
        (
            "main(){x=0;y=0;while(x<3){x++;switch(x){case 1:switch(x){default:continue;}default:y++;}}return y;}",
            2,
        ),
        (
            "main(){x=0;switch(1){case 1:for(i=0;i<3;i++){switch(i){case 1:continue;default:x++;}}x+=10;break;default:x=99;}return x;}",
            12,
        ),
    ] {
        let module = module_compile_support::compile(
            text.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Nested control ownership");
        assert_eq!(
            module
                .program()
                .execute(0, &[], KernelCallBudget::default())
                .expect("Correct control targets")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected)),
            "{text}"
        );
    }
}

#[test]
fn switch_expressions_evaluate_once_and_can_call_scripts_indirectly_and_builtins() {
    let text = b"main(){f=::choose;switch([[f]](7)){case 0:return 11;case 1:return 22;default:return 33;}}choose(n){return isint(n);}";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Real switch expression calls");
    let result = module
        .program()
        .execute(0, &[], KernelCallBudget::default())
        .expect("Single expression result");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(22)));
    assert_eq!(result.usage.peak_frames, 2);
    assert_eq!(result.usage.steps, 19);
}

#[test]
fn switch_continues_preserve_for_update_initialization_and_global_fuel() {
    let text =
        b"main(n){for(i=0;i<2;x++){switch(n){case 1:x=0;continue;default:return 7;}}return 0;}";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Only continuing paths initialize update x");
    assert_eq!(
        module
            .program()
            .execute(
                0,
                &[KernelValue::Number(LegacyNumber::Integer(0))],
                KernelCallBudget::default()
            )
            .expect("Returning path does not execute update")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
    let error = module
        .program()
        .execute(
            0,
            &[KernelValue::Number(LegacyNumber::Integer(1))],
            KernelCallBudget {
                max_steps: 128,
                ..KernelCallBudget::default()
            },
        )
        .expect_err("Continues reevaluate until global fuel exhausts");
    assert!(matches!(
        error.cause.kind,
        KernelErrorKind::BudgetExceeded {
            resource: KernelResource::ExecutionSteps,
            ..
        }
    ));
    assert_eq!(error.cause.work_steps, 128);
    assert_eq!(error.peak_frames, 1);
}
