// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primitive source triples own code/maps and preserve coordinate/value operations.
#![allow(
    clippy::expect_used,
    reason = "Tests use valid finite primitive source and exact bit results."
)]

use crate::function_compile_support;
use cod4x_script::compilation::{FunctionCompilationBudget, FunctionSourceProfile};
use cod4x_script::execution::{KernelExecutionBudget, KernelValue};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn source_vectors_execute_returns_assignments_compounds_loops_and_singleton_groups() {
    for (text, expected) in [
        (
            "main(a){v=(a,2,3);v*=2;v-=(1,1,1);return v;}",
            [5.0_f32, 3.0, 5.0],
        ),
        (
            "main(a){v=(a,a+1,a+2);for(i=0;i<3;i++){v*=2;}if(a)return v;return(0,0,0);}",
            [24.0, 32.0, 40.0],
        ),
        ("main(a){return((((a,2,3))));}", [3.0, 2.0, 3.0]),
        ("main(a){return((a),((2)),3);}", [3.0, 2.0, 3.0]),
    ] {
        let mut source = text.as_bytes().to_vec();
        let compiled = function_compile_support::compile_primitive(
            &source,
            FunctionCompilationBudget::default(),
        )
        .expect("Vector source");
        source.fill(0);
        assert_eq!(
            compiled.profile(),
            FunctionSourceProfile::PrimitiveStatements
        );
        assert_eq!(compiled.usage().validation_steps, None);
        let argument = [KernelValue::Number(LegacyNumber::Integer(3))];
        let expected = KernelValue::Vector(LegacyVector::from_components(expected));
        for _ in 0..2 {
            assert_eq!(
                compiled
                    .program()
                    .execute_with_arguments(&argument, KernelExecutionBudget::default())
                    .expect("Source vector execution")
                    .value,
                expected
            );
        }
        assert_eq!(argument[0], KernelValue::Number(LegacyNumber::Integer(3)));
    }
    for text in [
        "main(){return(1,2,3)==(1,2,3);}",
        "main(){return(1,1,1)==1;}",
    ] {
        let compiled = function_compile_support::compile_primitive(
            text.as_bytes(),
            FunctionCompilationBudget::default(),
        )
        .expect("Vector comparison source");
        assert_eq!(
            compiled
                .program()
                .execute(KernelExecutionBudget::default())
                .expect("Source comparison")
                .value,
            KernelValue::Number(LegacyNumber::Integer(1))
        );
    }
}

#[test]
fn source_vector_conversion_and_parameter_float_bits_preserve_the_legacy_domain()
-> Result<(), String> {
    let compiled = function_compile_support::compile_primitive(
        b"main(){return(-2147483648,16777217,-0.0);}",
        FunctionCompilationBudget::default(),
    )
    .expect("Signed vector literals");
    let KernelValue::Vector(value) = compiled
        .program()
        .execute(KernelExecutionBudget::default())
        .expect("Converted source vector")
        .value
    else {
        return Err("Expected vector".into());
    };
    assert_eq!(
        value.components().map(f32::to_bits),
        [0xcf00_0000, 0x4b80_0000, 0x8000_0000]
    );
    let compiled = function_compile_support::compile_primitive(
        b"main(a){return(a,2,3);}",
        FunctionCompilationBudget::default(),
    )
    .expect("Parameter constructor");
    let input = f32::from_bits(0x7f80_0001);
    let KernelValue::Vector(value) = compiled
        .program()
        .execute_with_arguments(
            &[KernelValue::Number(LegacyNumber::Float(input))],
            KernelExecutionBudget::default(),
        )
        .expect("Float bits")
        .value
    else {
        return Err("Expected vector".into());
    };
    assert_eq!(
        value.components().map(f32::to_bits),
        [0x7f80_0001, 0x4000_0000, 0x4040_0000]
    );
    Ok(())
}
