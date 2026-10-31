// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::lexical::legacy_tables;
use crate::lexical::{LegacyLexError, LegacyLexErrorKind, LegacyLexResource};

#[derive(Clone, Copy)]
pub(super) enum ScanMode {
    Code,
    BlockComment,
}

pub(super) fn first(
    bytes: &[u8],
    mode: ScanMode,
    maximum: u64,
    remaining_steps: &mut u64,
) -> Result<(u8, usize), LegacyLexError> {
    let mut state = match mode {
        ScanMode::Code => legacy_tables::NORMAL_STATE,
        ScanMode::BlockComment => legacy_tables::COMMENT_STATE,
    };
    let mut accepted = None;
    for (position, byte) in bytes.iter().copied().enumerate() {
        if byte == 0 {
            return Err(error(LegacyLexErrorKind::NulByte, position));
        }
        let rule = word(&legacy_tables::ACCEPT, state).map_err(|kind| error(kind, position))?;
        if rule != 0 {
            accepted = Some((rule, position));
        }
        let symbol = usize::from(legacy_tables::EQUIVALENCE[usize::from(byte)]);
        state = transition(state, symbol, remaining_steps).map_err(|kind| error(kind, position))?;
        if word(&legacy_tables::BASE, state).map_err(|kind| error(kind, position))?
            == usize::from(legacy_tables::JAM_BASE)
        {
            let rule = word(&legacy_tables::ACCEPT, state).map_err(|kind| error(kind, position))?;
            let result = if rule == 0 {
                accepted
            } else {
                Some((rule, position + 1))
            };
            return finish(result, maximum, position);
        }
        if position as u64 + 1 > maximum {
            return Err(error(
                LegacyLexErrorKind::BudgetExceeded(LegacyLexResource::LexemeBytes),
                position,
            ));
        }
    }
    let rule = word(&legacy_tables::ACCEPT, state).map_err(|kind| error(kind, bytes.len()))?;
    finish(
        if rule == 0 {
            accepted
        } else {
            Some((rule, bytes.len()))
        },
        maximum,
        bytes.len(),
    )
}

fn finish(
    result: Option<(usize, usize)>,
    maximum: u64,
    position: usize,
) -> Result<(u8, usize), LegacyLexError> {
    let (rule, length) = result
        .filter(|(_, length)| *length != 0)
        .ok_or_else(|| error(LegacyLexErrorKind::InvalidByte, position))?;
    if length as u64 > maximum {
        return Err(error(
            LegacyLexErrorKind::BudgetExceeded(LegacyLexResource::LexemeBytes),
            position,
        ));
    }
    let rule = u8::try_from(rule).map_err(|_| error(LegacyLexErrorKind::InvalidTable, position))?;
    Ok((rule, length))
}

fn transition(
    mut state: usize,
    mut symbol: usize,
    remaining: &mut u64,
) -> Result<usize, LegacyLexErrorKind> {
    for _ in 0..legacy_tables::BASE.len() {
        *remaining = remaining
            .checked_sub(1)
            .ok_or(LegacyLexErrorKind::BudgetExceeded(LegacyLexResource::Steps))?;
        let index = word(&legacy_tables::BASE, state)?
            .checked_add(symbol)
            .ok_or(LegacyLexErrorKind::InvalidTable)?;
        if word(&legacy_tables::CHECK, index)? == state {
            return word(&legacy_tables::NEXT, index);
        }
        state = word(&legacy_tables::DEFAULT, state)?;
        if state > 0x101 {
            symbol = usize::from(
                *legacy_tables::META
                    .get(symbol)
                    .ok_or(LegacyLexErrorKind::InvalidTable)?,
            );
        }
    }
    Err(LegacyLexErrorKind::InvalidTable)
}

fn word(table: &[u16], index: usize) -> Result<usize, LegacyLexErrorKind> {
    table
        .get(index)
        .copied()
        .map(usize::from)
        .ok_or(LegacyLexErrorKind::InvalidTable)
}

fn error(kind: LegacyLexErrorKind, offset: usize) -> LegacyLexError {
    LegacyLexError {
        kind,
        offset: offset as u64,
    }
}
