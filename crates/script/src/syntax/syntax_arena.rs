// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::syntax::legacy_syntax_error;
use crate::syntax::syntax_ledger::SyntaxLedger;
use crate::syntax::syntax_node::{Node, NodeData};
use crate::syntax::syntax_stack::StackEntry;
use crate::syntax::{LegacySyntaxErrorKind, LegacySyntaxResource};

pub(super) struct SyntaxArena<'source> {
    pub(super) nodes: Vec<Node<'source>>,
    pub(super) edges: Vec<usize>,
}

impl<'source> SyntaxArena<'source> {
    pub(super) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub(super) fn leaf(
        &mut self,
        data: NodeData<'source>,
        span: Range<u64>,
        ledger: &mut SyntaxLedger,
    ) -> Result<usize, LegacySyntaxErrorKind> {
        self.admit(0, ledger)?;
        let index = self.nodes.len();
        self.nodes.push(Node {
            data,
            span,
            children: self.edges.len()..self.edges.len(),
        });
        Ok(index)
    }

    pub(super) fn production(
        &mut self,
        rule: u16,
        rhs: &[StackEntry],
        anchor: u64,
        ledger: &mut SyntaxLedger,
    ) -> Result<usize, LegacySyntaxErrorKind> {
        let mut span = anchor..anchor;
        for (position, entry) in rhs.iter().enumerate() {
            let index = entry.node.ok_or(LegacySyntaxErrorKind::InvalidTable)?;
            let node = self
                .nodes
                .get(index)
                .ok_or(LegacySyntaxErrorKind::InvalidTable)?;
            if position == 0 {
                span.start = node.span.start;
            }
            span.end = node.span.end;
        }
        if span.start > span.end {
            return Err(LegacySyntaxErrorKind::InvalidTable);
        }
        self.admit(rhs.len(), ledger)?;
        let start = self.edges.len();
        for entry in rhs {
            self.edges
                .push(entry.node.ok_or(LegacySyntaxErrorKind::InvalidTable)?);
        }
        let index = self.nodes.len();
        self.nodes.push(Node {
            data: NodeData::Production(rule),
            span,
            children: start..self.edges.len(),
        });
        Ok(index)
    }

    fn admit(
        &mut self,
        edges: usize,
        ledger: &mut SyntaxLedger,
    ) -> Result<(), LegacySyntaxErrorKind> {
        let nodes = self
            .nodes
            .len()
            .checked_add(1)
            .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        let total_edges = self
            .edges
            .len()
            .checked_add(edges)
            .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        legacy_syntax_error::check(
            LegacySyntaxResource::Nodes,
            nodes as u64,
            ledger.budget.max_nodes,
        )?;
        legacy_syntax_error::check(
            LegacySyntaxResource::ChildEdges,
            total_edges as u64,
            ledger.budget.max_child_edges,
        )?;
        let bytes = edges
            .checked_mul(size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(size_of::<Node<'source>>()))
            .ok_or(LegacySyntaxErrorKind::SizeOverflow)?;
        ledger.admit_metadata(bytes)?;
        self.nodes
            .try_reserve(1)
            .map_err(|_| LegacySyntaxErrorKind::AllocationFailed)?;
        self.edges
            .try_reserve(edges)
            .map_err(|_| LegacySyntaxErrorKind::AllocationFailed)
    }
}
