// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare real aggregate frontend work/storage against exact admission."
)]
use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

use super::frontend::ScriptPackageFrontend;
use super::phases;
use crate::app_error::AppError;

const SOURCE: &[u8] = b"main(a){return a;}";
type Resource = fn(&mut ScriptPackageFrontend) -> &mut u64;

fn exact() -> ScriptPackageFrontend {
    let source = SourceText::index(SOURCE, SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )
    .expect("Tree");
    let module = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Module");
    let symbols = LegacyModuleSymbols::bind(
        &module,
        LegacyDeveloperMode::Disabled,
        SymbolBudget::default(),
    )
    .expect("Symbols");
    let mut ledger = ScriptPackageFrontend::default();
    ledger.source.max_source_bytes = 2 * SOURCE.len() as u64;
    ledger.source.max_lines = 2 * source.line_count();
    ledger.source.max_metadata_bytes = 2 * source.metadata_bytes();
    ledger.syntax.max_nodes = 2 * tree.usage().nodes;
    ledger.syntax.max_child_edges = 2 * tree.usage().child_edges;
    ledger.syntax.max_steps = 2 * tree.usage().steps;
    ledger.syntax.max_metadata_bytes = 2 * tree.usage().peak_metadata_bytes;
    ledger.syntax.lexical.max_steps = 2 * tree.usage().lexical.steps;
    ledger.syntax.lexical.max_lexemes = 2 * tree.usage().lexical.attempted_lexemes;
    ledger.declaration.max_items = 2 * module.usage().items;
    ledger.declaration.max_parameters = 2 * module.usage().parameters;
    ledger.declaration.max_steps = 2 * module.usage().steps;
    ledger.declaration.max_metadata_bytes = 2 * module.usage().metadata_bytes;
    ledger.symbol.max_functions = 2 * symbols.usage().functions;
    ledger.symbol.max_name_bytes = 2 * symbols.usage().name_bytes;
    ledger.symbol.max_steps = 2 * symbols.usage().steps;
    ledger.symbol.max_metadata_bytes = 2 * symbols.usage().metadata_bytes;
    ledger
}

fn prepare(ledger: &mut ScriptPackageFrontend) -> Result<(), AppError> {
    let bytes = [SOURCE.to_vec(), SOURCE.to_vec()];
    let sources = phases::index(&bytes, ledger)?;
    let trees = phases::parse(&sources, ledger)?;
    let modules = phases::declare(&sources, &trees, ledger)?;
    let symbols = phases::bind(&sources, &modules, LegacyDeveloperMode::Disabled, ledger)?;
    assert_eq!(symbols.len(), 2);
    Ok(())
}

#[test]
fn each_frontend_phase_admits_exact_aggregate_resources_and_rejects_one_less() {
    let resources: [Resource; 17] = [
        |ledger| &mut ledger.source.max_source_bytes,
        |ledger| &mut ledger.source.max_lines,
        |ledger| &mut ledger.source.max_metadata_bytes,
        |ledger| &mut ledger.syntax.max_nodes,
        |ledger| &mut ledger.syntax.max_child_edges,
        |ledger| &mut ledger.syntax.max_steps,
        |ledger| &mut ledger.syntax.max_metadata_bytes,
        |ledger| &mut ledger.syntax.lexical.max_steps,
        |ledger| &mut ledger.syntax.lexical.max_lexemes,
        |ledger| &mut ledger.declaration.max_items,
        |ledger| &mut ledger.declaration.max_parameters,
        |ledger| &mut ledger.declaration.max_steps,
        |ledger| &mut ledger.declaration.max_metadata_bytes,
        |ledger| &mut ledger.symbol.max_functions,
        |ledger| &mut ledger.symbol.max_name_bytes,
        |ledger| &mut ledger.symbol.max_steps,
        |ledger| &mut ledger.symbol.max_metadata_bytes,
    ];
    let exact = exact();
    let mut admitted = exact;
    prepare(&mut admitted).expect("Exact aggregate frontend resources");
    for (index, resource) in resources.into_iter().enumerate() {
        assert_eq!(*resource(&mut admitted), 0);
        let mut insufficient = exact;
        *resource(&mut insufficient) -= 1;
        assert!(
            prepare(&mut insufficient).is_err(),
            "Resource {index} must span both files"
        );
    }
}
