// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite generated include graphs explicitly admit wide namespace and alias scopes."
)]
use std::fmt::Write;

use crate::package_compile_support::compile;
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::FunctionCompileErrorKind;
use cod4x_script::compilation::ScriptPackageBudget;
use cod4x_script::execution::{KernelCallBudget, KernelValue};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn seventy_thousand_imported_names_and_global_targets_preserve_full_indices() {
    let mut owned = Vec::new();
    let mut root = String::new();
    for module in 0..70 {
        writeln!(&mut root, "#include lib{module};").expect("Directive");
        let mut source = String::new();
        for function in 0..1000 {
            let value = i32::from(module == 69 && function == 999) * 42;
            write!(&mut source, "f{module}_{function}(){{return {value};}}").expect("Function");
        }
        owned.push((format!("lib{module}"), source));
    }
    root.push_str("main(){return f69_999();}");
    owned.insert(0, ("root".into(), root));
    let inputs: Vec<_> = owned
        .iter()
        .map(|(name, source)| (name.as_bytes(), source.as_bytes()))
        .collect();
    let package =
        compile(&inputs, Disabled, ScriptPackageBudget::default()).expect("Wide alias inventory");
    assert_eq!(package.usage().include_edges, 70);
    assert_eq!(package.usage().imported_functions, 70_000);
    assert_eq!(package.functions().len(), 70_001);
    assert_eq!(package.function_for_declaration(70, 999), Some(70_000));
    assert_eq!(
        package
            .instruction_location(70_000, 0)
            .expect("Original target map")
            .module,
        70
    );
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Wide imported call")
            .value,
        KernelValue::Number(LegacyNumber::Integer(42))
    );
}

#[test]
fn seventy_thousand_include_edges_have_no_native_precache_or_short_count_ceiling() {
    let mut root = String::new();
    let mut names = Vec::new();
    for index in 0..70_000 {
        let name = format!("lib{index:05}");
        writeln!(&mut root, "#include {name};").expect("Wide directive");
        names.push(name);
    }
    root.push_str("main(){return 23;}");
    let mut inputs = vec![(b"root".as_slice(), root.as_bytes())];
    inputs.extend(names.iter().map(|name| (name.as_bytes(), b"".as_slice())));
    let mut budget = ScriptPackageBudget {
        max_modules: 70_001,
        max_include_edges: 70_000,
        max_imported_functions: 0,
        ..ScriptPackageBudget::default()
    };
    budget.compilation.max_steps = 128_000_000;
    let package = compile(&inputs, Disabled, budget).expect("Wide flat include graph");
    assert_eq!(package.usage().include_edges, 70_000);
    assert_eq!(package.usage().imported_functions, 0);
    assert_eq!(package.modules().len(), 70_001);
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Wide graph entry")
            .value,
        KernelValue::Number(LegacyNumber::Integer(23))
    );
}

#[test]
fn seventy_thousand_byte_imported_function_names_keep_legacy_long_name_case_identity() {
    let lower = "a".repeat(70_000);
    let upper = lower.to_ascii_uppercase();
    let root = format!("#include lib;main(){{return {lower}()+{upper}();}}");
    let library = format!("{lower}(){{return 7;}}{upper}(){{return 9;}}");
    let package = compile(
        &[(b"root", root.as_bytes()), (b"lib", library.as_bytes())],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Complete distinct long alias keys");
    assert_eq!(package.usage().imported_functions, 2);
    assert_eq!(
        package
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Exact long key identity")
            .value,
        KernelValue::Number(LegacyNumber::Integer(16))
    );
    let mixed = format!("{}A", &lower[..lower.len() - 1]);
    let root = format!("#include lib;main(){{return {mixed}();}}");
    let error = compile(
        &[(b"root", root.as_bytes()), (b"lib", library.as_bytes())],
        Disabled,
        ScriptPackageBudget::default(),
    )
    .expect_err("No long-name global case folding");
    assert_eq!(error.cause.kind, FunctionCompileErrorKind::UnknownFunction);
}
