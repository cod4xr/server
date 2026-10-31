// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::lexical::{LegacyLexemeKind, LegacyLexer};
use crate::source::SourceText;
use crate::syntax::grammar::{self, Action};
use crate::syntax::syntax_arena::SyntaxArena;
use crate::syntax::syntax_ledger::SyntaxLedger;
use crate::syntax::syntax_lookahead::Lookahead;
use crate::syntax::syntax_node::NodeData;
use crate::syntax::syntax_stack::SyntaxStack;
use crate::syntax::{
    LegacySyntaxBudget, LegacySyntaxError, LegacySyntaxErrorKind, LegacySyntaxMode,
    LegacySyntaxTree, LegacySyntaxUsage,
};

struct Parser<'source> {
    lexer: LegacyLexer<'source>,
    lookahead: Option<Lookahead<'source>>,
    ledger: SyntaxLedger,
    stack: SyntaxStack,
    arena: SyntaxArena<'source>,
    source_tokens: u64,
    reductions: u64,
    length: u64,
}

pub(super) fn parse<'source>(
    source: &SourceText<'source>,
    mode: LegacySyntaxMode,
    budget: LegacySyntaxBudget,
) -> Result<LegacySyntaxTree<'source>, LegacySyntaxError> {
    let mut ledger = SyntaxLedger::new(budget);
    let stack =
        SyntaxStack::new(&mut ledger).map_err(|kind| LegacySyntaxError { kind, offset: 0 })?;
    let mut parser = Parser {
        lexer: LegacyLexer::new(source, budget.lexical),
        lookahead: Some(Lookahead::Selector(mode)),
        ledger,
        stack,
        arena: SyntaxArena::new(),
        source_tokens: 0,
        reductions: 0,
        length: source.bytes().len() as u64,
    };
    loop {
        let result = parser.advance();
        match result {
            Ok(false) => {}
            Ok(true) => return parser.finish(source.bytes(), mode),
            Err(kind) => {
                return Err(LegacySyntaxError {
                    kind,
                    offset: parser.offset(),
                });
            }
        }
    }
}

impl<'source> Parser<'source> {
    fn offset(&self) -> u64 {
        self.lookahead
            .map_or(self.lexer.usage().consumed_bytes, Lookahead::offset)
    }

    fn token(&mut self) -> Result<Lookahead<'source>, LegacySyntaxErrorKind> {
        if let Some(token) = self.lookahead {
            return Ok(token);
        }
        for result in self.lexer.by_ref() {
            match result {
                Ok(lexeme) => {
                    if let LegacyLexemeKind::Token(code) = lexeme.kind() {
                        let token = Lookahead::Source { code, lexeme };
                        self.lookahead = Some(token);
                        return Ok(token);
                    }
                }
                Err(error) => {
                    // Retain the scanner's exact error offset, including an opening
                    // comment or lookahead byte beyond successfully consumed input.
                    self.lookahead = Some(Lookahead::Eof(error.offset));
                    return Err(LegacySyntaxErrorKind::Lexical(error.kind));
                }
            }
        }
        let token = Lookahead::Eof(self.length);
        self.lookahead = Some(token);
        Ok(token)
    }

    fn advance(&mut self) -> Result<bool, LegacySyntaxErrorKind> {
        self.ledger.step()?;
        let state = self.stack.state()?;
        // Default reductions must run before asking the scanner for another token.
        let code = if grammar::needs_token(state)? {
            self.token()?.code()
        } else {
            0
        };
        match grammar::action(state, code)? {
            Action::Shift(state) => self.shift(state)?,
            Action::Reduce(rule) => self.reduce(rule)?,
            Action::Accept => return Ok(true),
            Action::Error => return Err(LegacySyntaxErrorKind::UnexpectedToken),
        }
        Ok(false)
    }

    fn shift(&mut self, state: usize) -> Result<(), LegacySyntaxErrorKind> {
        let token = self.lookahead.ok_or(LegacySyntaxErrorKind::InvalidTable)?;
        let (data, span) = match token {
            Lookahead::Selector(mode) => (NodeData::Selector(mode), 0..0),
            Lookahead::Source { code, lexeme } => {
                (NodeData::Terminal { code, lexeme }, lexeme.span())
            }
            Lookahead::Eof(offset) => (NodeData::EndOfInput, offset..offset),
        };
        let node = self.arena.leaf(data, span, &mut self.ledger)?;
        self.stack.push(state, Some(node), &mut self.ledger)?;
        if matches!(token, Lookahead::Source { .. }) {
            self.source_tokens = self
                .source_tokens
                .checked_add(1)
                .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        }
        // The original parser caches EOF even after shifting it.
        if !matches!(token, Lookahead::Eof(_)) {
            self.lookahead = None;
        }
        Ok(())
    }

    fn reduce(&mut self, rule: u16) -> Result<(), LegacySyntaxErrorKind> {
        let count = grammar::production_length(rule)?;
        let node = self.arena.production(
            rule,
            self.stack.rhs(count)?,
            self.offset(),
            &mut self.ledger,
        )?;
        self.stack.pop(count, &mut self.ledger)?;
        let state = grammar::goto(self.stack.state()?, rule)?;
        self.stack.push(state, Some(node), &mut self.ledger)?;
        self.reductions = self
            .reductions
            .checked_add(1)
            .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        Ok(())
    }

    fn finish(
        self,
        bytes: &'source [u8],
        mode: LegacySyntaxMode,
    ) -> Result<LegacySyntaxTree<'source>, LegacySyntaxError> {
        // Accept follows an EOF shift; the preceding entry production is the root.
        let root = self.entry_root().map_err(|kind| LegacySyntaxError {
            kind,
            offset: self.offset(),
        })?;
        let usage = LegacySyntaxUsage {
            source_tokens: self.source_tokens,
            reductions: self.reductions,
            nodes: self.arena.nodes.len() as u64,
            child_edges: self.arena.edges.len() as u64,
            steps: self.ledger.steps,
            peak_stack_entries: self.ledger.peak_stack,
            peak_metadata_bytes: self.ledger.peak_metadata,
            lexical: self.lexer.usage(),
        };
        Ok(LegacySyntaxTree {
            bytes,
            mode,
            arena: self.arena,
            root,
            usage,
        })
    }

    fn entry_root(&self) -> Result<usize, LegacySyntaxErrorKind> {
        let eof = self.stack.node()?;
        if !matches!(
            self.arena.nodes.get(eof).map(|node| node.data),
            Some(NodeData::EndOfInput)
        ) {
            return Err(LegacySyntaxErrorKind::InvalidTable);
        }
        let index = self
            .stack
            .rhs(2)?
            .first()
            .and_then(|entry| entry.node)
            .ok_or(LegacySyntaxErrorKind::InvalidTable)?;
        if !matches!(
            self.arena.nodes.get(index).map(|node| node.data),
            Some(NodeData::Production(1..=4))
        ) {
            return Err(LegacySyntaxErrorKind::InvalidTable);
        }
        Ok(index)
    }
}
