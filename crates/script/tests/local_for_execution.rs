// SPDX-License-Identifier: AGPL-3.0-or-later
//! For phases preserve initializer/update ordering and mixed nearest-loop transfers.
#![allow(
    clippy::expect_used,
    reason = "Tests compile finite valid synthetic for functions."
)]

use crate::function_compile_support;
use cod4x_script::compilation::FunctionCompilationBudget;
use cod4x_script::execution::{KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn for_conditions_updates_and_mixed_loop_transfers_execute() {
    for (text, results) in [
        (
            "main(n){sum=0;for(i=0;i<n;i++){if(i==2)continue;if(i>5)break;sum+=i;}return sum;}",
            (0..9)
                .map(|n| (0..n.min(6)).filter(|i| *i != 2).sum())
                .collect::<Vec<i32>>(),
        ),
        (
            "main(n){sum=0;for(n-=1;n>0;n--){sum++;}return sum;}",
            (0..9).map(|n| (n - 1).max(0)).collect(),
        ),
        (
            "main(n){sum=0;for(i=0;i<n;i++){j=0;while(j<4){j++;if(j==2)continue;if(j==4)break;sum+=j;}if(i==1)continue;sum+=10;}return sum;}",
            (0..9)
                .map(|n| n * 4 + 10 * (n - i32::from(n >= 2)))
                .collect(),
        ),
        (
            "main(n){i=0;sum=0;while(i<n){i++;for(j=0;j<3;j++){if(j==1)continue;sum++;}if(i==2)continue;sum+=10;}return sum;}",
            (0..9)
                .map(|n| n * 2 + 10 * (n - i32::from(n >= 2)))
                .collect(),
        ),
    ] {
        let mut source = text.as_bytes().to_vec();
        let compiled = function_compile_support::compile_primitive(
            &source,
            FunctionCompilationBudget::default(),
        )
        .expect("For source");
        source.fill(0);
        assert_eq!(compiled.usage().validation_steps, None);
        for (arg, expected) in (0..9).zip(results) {
            let args = [KernelValue::Number(LegacyNumber::Integer(arg))];
            assert_eq!(
                compiled
                    .program()
                    .execute_with_arguments(&args, KernelExecutionBudget::default())
                    .expect("For invocation")
                    .value,
                KernelValue::Number(LegacyNumber::Integer(expected)),
                "{text}"
            );
            assert_eq!(args[0], KernelValue::Number(LegacyNumber::Integer(arg)));
        }
    }
}

#[test]
fn every_optional_clause_combination_and_returning_body_have_real_results() {
    for initializer in ["", "i=0"] {
        for condition in ["", "i<3"] {
            for update in ["", "i++"] {
                let manual = if update.is_empty() { "i++;" } else { "" };
                let source = format!(
                    "main(){{i=0;for({initializer};{condition};{update}){{if(i>=3)break;{manual}}}return i;}}"
                );
                let compiled = function_compile_support::compile_primitive(
                    source.as_bytes(),
                    FunctionCompilationBudget::default(),
                )
                .expect("Optional for clauses");
                assert_eq!(
                    compiled
                        .program()
                        .execute(KernelExecutionBudget::default())
                        .expect("Optional clause execution")
                        .value,
                    KernelValue::Number(LegacyNumber::Integer(3))
                );
            }
        }
    }
    for (text, expected) in [
        (
            "main(){for(;;){break;}return 4;}",
            KernelValue::Number(LegacyNumber::Integer(4)),
        ),
        (
            "main(){for(i=0;i<3;i++){return 5;}}",
            KernelValue::Number(LegacyNumber::Integer(5)),
        ),
        (
            "main(){for(i=0;i<0;i++){return 5;}}",
            KernelValue::Undefined,
        ),
        (
            "main(){for(i=0;i<2;i=i+1){}return i;}",
            KernelValue::Number(LegacyNumber::Integer(2)),
        ),
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Return/empty update source");
        assert_eq!(
            compiled
                .program()
                .execute(KernelExecutionBudget::default())
                .expect("Real return")
                .value,
            expected
        );
    }
}
