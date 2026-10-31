// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::declaration::declaration_ledger::DeclarationLedger;
use crate::declaration::{
    DeclarationBudget, DeclarationError, DeclarationErrorKind, DeclarationResource,
    LegacyFunctionSyntax, LegacyModuleItem, LegacyModuleSyntax,
};
use crate::lexical::{LegacyLexeme, LegacyLexemeKind};
use crate::syntax::{LegacySyntaxMode, LegacySyntaxNode, LegacySyntaxNodeKind, LegacySyntaxTree};

pub(super) fn lower<'tree, 'source>(
    tree: &'tree LegacySyntaxTree<'source>,
    budget: DeclarationBudget,
) -> Result<LegacyModuleSyntax<'tree, 'source>, DeclarationError> {
    if tree.mode() != LegacySyntaxMode::Script {
        return Err(DeclarationError {
            kind: DeclarationErrorKind::ExpectedScript,
            offset: 0,
        });
    }
    let mut ledger = DeclarationLedger::new(budget);
    let mut includes = Vec::new();
    let mut items = Vec::new();
    for node in tree.nodes() {
        let result = (|| {
            ledger.visit()?;
            let LegacySyntaxNodeKind::Production(code) = node.kind() else {
                return Ok(());
            };
            match code.as_u16() {
                131 => {
                    ledger.admit(
                        DeclarationResource::Includes,
                        size_of::<LegacyLexeme<'source>>(),
                    )?;
                    includes
                        .try_reserve(1)
                        .map_err(|_| DeclarationErrorKind::AllocationFailed)?;
                    let path = child(node, 1, 2)?;
                    if !matches!(production(path), Some(28 | 29)) {
                        return Err(DeclarationErrorKind::InvalidStructure);
                    }
                    includes.push(lexeme(child(path, 0, 1)?)?);
                }
                125..=128 => {
                    ledger.admit(
                        DeclarationResource::Items,
                        size_of::<LegacyModuleItem<'tree, 'source>>(),
                    )?;
                    items
                        .try_reserve(1)
                        .map_err(|_| DeclarationErrorKind::AllocationFailed)?;
                    items.push(item(node, code.as_u16(), &mut ledger)?);
                }
                _ => {}
            }
            Ok(())
        })();
        result.map_err(|kind| DeclarationError {
            kind,
            offset: node.span().start,
        })?;
    }
    Ok(LegacyModuleSyntax {
        includes,
        items,
        usage: ledger.usage,
    })
}

fn item<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    rule: u16,
    ledger: &mut DeclarationLedger,
) -> Result<LegacyModuleItem<'tree, 'source>, DeclarationErrorKind> {
    Ok(match rule {
        125 => LegacyModuleItem::Function(function(node, ledger)?),
        126 => LegacyModuleItem::UsingAnimTree(lexeme(child(node, 2, 5)?)?),
        127 => LegacyModuleItem::DeveloperBegin {
            offset: lexeme(child(node, 0, 1)?)?.span().start,
        },
        128 => LegacyModuleItem::DeveloperEnd {
            offset: lexeme(child(node, 0, 1)?)?.span().start,
        },
        _ => return Err(DeclarationErrorKind::InvalidStructure),
    })
}

fn function<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    ledger: &mut DeclarationLedger,
) -> Result<LegacyFunctionSyntax<'tree, 'source>, DeclarationErrorKind> {
    let name = identifier(child(node, 0, 7)?)?;
    let list = child(node, 2, 7)?;
    let body = child(node, 5, 7)?;
    if !matches!(production(body), Some(109 | 110)) {
        return Err(DeclarationErrorKind::InvalidStructure);
    }
    let mut parameters = Vec::new();
    let mut current = list;
    loop {
        ledger.visit()?;
        match production(current) {
            Some(118) => {
                if current.children().len() != 0 {
                    return Err(DeclarationErrorKind::InvalidStructure);
                }
                break;
            }
            Some(117) => current = child(current, 0, 1)?,
            Some(rule @ (115 | 116)) => {
                let terminal = if rule == 115 {
                    child(current, 2, 3)?
                } else {
                    child(current, 0, 1)?
                };
                let parameter = identifier(terminal)?;
                ledger.admit(
                    DeclarationResource::Parameters,
                    size_of::<LegacyLexeme<'source>>(),
                )?;
                parameters
                    .try_reserve(1)
                    .map_err(|_| DeclarationErrorKind::AllocationFailed)?;
                parameters.push(parameter);
                if rule == 116 {
                    break;
                }
                current = child(current, 0, 3)?;
            }
            _ => return Err(DeclarationErrorKind::InvalidStructure),
        }
    }
    parameters.reverse();
    Ok(LegacyFunctionSyntax {
        name,
        parameters,
        body,
        span: node.span(),
    })
}

fn child<'tree, 'source>(
    node: LegacySyntaxNode<'tree, 'source>,
    index: usize,
    count: usize,
) -> Result<LegacySyntaxNode<'tree, 'source>, DeclarationErrorKind> {
    if node.children().len() != count {
        return Err(DeclarationErrorKind::InvalidStructure);
    }
    node.children()
        .nth(index)
        .ok_or(DeclarationErrorKind::InvalidStructure)
}

fn production(node: LegacySyntaxNode<'_, '_>) -> Option<u16> {
    match node.kind() {
        LegacySyntaxNodeKind::Production(code) => Some(code.as_u16()),
        _ => None,
    }
}

fn lexeme<'source>(
    node: LegacySyntaxNode<'_, 'source>,
) -> Result<LegacyLexeme<'source>, DeclarationErrorKind> {
    node.lexeme().ok_or(DeclarationErrorKind::InvalidStructure)
}

fn identifier<'source>(
    node: LegacySyntaxNode<'_, 'source>,
) -> Result<LegacyLexeme<'source>, DeclarationErrorKind> {
    let lexeme = lexeme(node)?;
    if !matches!(lexeme.kind(), LegacyLexemeKind::Token(code) if code.as_u16() == 0x102) {
        return Err(DeclarationErrorKind::InvalidStructure);
    }
    Ok(lexeme)
}
