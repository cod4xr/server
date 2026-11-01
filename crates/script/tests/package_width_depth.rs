// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite generated packages explicitly admit wide catalogs and deep iterative execution."
)]
use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::{
    FunctionCompileErrorKind, ModuleCompilationResource, ScriptPackageBudget,
};
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn sixty_six_thousand_modules_and_global_functions_keep_full_namespace_indices() {
    let names: Vec<_> = (0..66_000)
        .rev()
        .map(|index| format!("file{index:05}"))
        .collect();
    let inputs: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            (
                name.as_bytes(),
                if index == 0 {
                    b"main(){return file00000::helper();}".as_slice()
                } else {
                    b"helper(){return 42;}".as_slice()
                },
            )
        })
        .collect();
    let error = compile(&inputs, Disabled, ScriptPackageBudget::default())
        .expect_err("Default resource is configurable admission");
    assert!(matches!(
        error.cause.kind,
        FunctionCompileErrorKind::ModuleBudgetExceeded {
            resource: ModuleCompilationResource::Modules,
            requested: 66_000,
            ..
        }
    ));
    let mut budget = ScriptPackageBudget {
        max_modules: 66_000,
        ..ScriptPackageBudget::default()
    };
    budget.compilation.max_steps = 128_000_000;
    let package = compile(&inputs, Disabled, budget).expect("Wide closed catalog");
    assert_eq!(package.modules().len(), 66_000);
    assert_eq!(package.functions().len(), 66_000);
    assert_eq!(package.function_for_declaration(65_999, 0), Some(65_999));
    assert_eq!(package.module_name(65_999), Some(b"file00000".as_slice()));
    assert_eq!(
        package
            .instruction_location(65_999, 0)
            .expect("Wide source module")
            .module,
        65_999
    );
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Wide cross-file target")
            .value,
        KernelValue::Number(LegacyNumber::Integer(42))
    );
}

#[test]
fn twelve_thousand_cross_file_frames_share_global_fuel_and_frame_admission() {
    let package = compile(
        &[
            (
                b"a",
                b"main(){return even(12000);}even(n){if(n==0){return 1;}else{return b::odd(n-1);}}",
            ),
            (
                b"b",
                b"odd(n){if(n==0){return 0;}else{return a::even(n-1);}}",
            ),
        ],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Cyclic file graph");
    let baseline = package
        .program()
        .execute(
            0,
            &[],
            KernelCallBudget {
                max_setup_work: 100_000_000,
                ..KernelCallBudget::default()
            },
        )
        .expect("Iterative frames");
    assert_eq!(
        baseline.value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
    assert_eq!(baseline.usage.peak_frames, 12_002);
    let exact = KernelCallBudget {
        max_steps: baseline.usage.steps,
        max_frames: baseline.usage.peak_frames,
        max_setup_work: 100_000_000,
        ..KernelCallBudget::default()
    };
    assert_eq!(
        package
            .program()
            .execute(0, &[], exact)
            .expect("Exact invocation resources")
            .usage,
        baseline.usage
    );
    for budget in [
        KernelCallBudget {
            max_steps: exact.max_steps - 1,
            ..exact
        },
        KernelCallBudget {
            max_frames: exact.max_frames - 1,
            ..exact
        },
    ] {
        let error = package
            .program()
            .execute(0, &[], budget)
            .expect_err("Global resource stays exhausted across files");
        assert!(error.peak_frames <= budget.max_frames);
        assert!(error.cause.work_steps <= budget.max_steps);
        let location = package
            .instruction_location(
                error.function.expect("Failing function"),
                error.cause.instruction.expect("Failing instruction"),
            )
            .expect("Cross-file failure map");
        assert!(location.module <= 1);
    }
}
