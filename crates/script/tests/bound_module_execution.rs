// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests provide finite valid primitive source modules and arguments."
)]

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn real_source_forward_calls_and_discarded_statements_own_all_code_after_source_drop() {
    let mut bytes =
        b"PrivateMain(){discard(7);return add(7,2);}add(a,b){return a*10+b;}discard(x){return x;}"
            .to_vec();
    let module = module_compile_support::compile(
        &bytes,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Bound source compilation");
    bytes.fill(0);
    let entry = module.function_for_declaration(0).expect("Active main");
    assert_eq!(
        module
            .program()
            .execute(entry, &[], KernelCallBudget::default())
            .expect("Real source calls")
            .value,
        KernelValue::Number(LegacyNumber::Integer(72))
    );
    assert_eq!(module.declarations().len(), 3);
    assert_eq!(module.functions().len(), 3);
    assert_eq!(module.usage().verification.functions, 3);
    assert!(
        module
            .functions()
            .iter()
            .all(|function| function.usage().verification_steps > 0
                && function.usage().validation_steps.is_none())
    );
    assert!(!format!("{module:?} {:?}", module.functions()).contains("PrivateMain"));
}

#[test]
fn source_mutual_recursion_and_vector_results_execute_through_shared_frames() {
    let module = module_compile_support::compile(b"main(n){return even(n);}even(n){if(n==0){return true;}return odd(n-1);}odd(n){if(n==0){return false;}return even(n-1);}", LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("Mutually recursive source");
    for (input, expected) in [(128, 1), (129, 0)] {
        let arguments = [KernelValue::Number(LegacyNumber::Integer(input))];
        let outcome = module
            .program()
            .execute(0, &arguments, KernelCallBudget::default())
            .expect("Source recursion");
        assert_eq!(
            outcome.value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
        assert!(outcome.usage.peak_frames > 32);
        assert_eq!(
            arguments[0],
            KernelValue::Number(LegacyNumber::Integer(input))
        );
    }
    let module = module_compile_support::compile(b"main(){return vec(4)[1]+offset();}vec(x){v=(x,2,3);return(v[0],v[1]+5,v[2]);}offset(){sum=0;for(i=0;i<3;i++){sum+=value(i);}return sum;}value(i){return i;}", LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("Source vectors and loop calls");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Copied vector returns preserve caller operands")
            .value,
        KernelValue::Number(LegacyNumber::Float(10.0))
    );
}
