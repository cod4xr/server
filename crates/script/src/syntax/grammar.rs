// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::syntax::LegacySyntaxErrorKind;
use crate::syntax::legacy_actions;
use crate::syntax::legacy_productions;

pub(super) enum Action {
    Shift(usize),
    Reduce(u16),
    Accept,
    Error,
}

pub(super) fn needs_token(state: usize) -> Result<bool, LegacySyntaxErrorKind> {
    Ok(at(&legacy_actions::PACT, state)? != legacy_actions::UNSET)
}

pub(super) fn action(state: usize, code: u16) -> Result<Action, LegacySyntaxErrorKind> {
    let base = at(&legacy_actions::PACT, state)?;
    let value = if base == legacy_actions::UNSET {
        -at(&legacy_actions::DEFAULT_REDUCE, state)?
    } else {
        let symbol = at(&legacy_actions::TRANSLATE, usize::from(code))?;
        let index = i32::from(base) + i32::from(symbol);
        if let Ok(index) = usize::try_from(index)
            && index <= legacy_actions::LAST
            && at(&legacy_actions::CHECK, index)? == symbol
        {
            at(&legacy_actions::ACTION, index)?
        } else {
            -at(&legacy_actions::DEFAULT_REDUCE, state)?
        }
    };
    if value == 0 || value == legacy_actions::UNSET {
        return Ok(Action::Error);
    }
    if value > 0 {
        let target = usize::try_from(value).map_err(|_| LegacySyntaxErrorKind::InvalidTable)?;
        if target >= legacy_actions::PACT.len() {
            return Err(LegacySyntaxErrorKind::InvalidTable);
        }
        return Ok(if target == legacy_actions::FINAL {
            Action::Accept
        } else {
            Action::Shift(target)
        });
    }
    let rule = u16::try_from(-i32::from(value)).map_err(|_| LegacySyntaxErrorKind::InvalidTable)?;
    production_length(rule)?;
    Ok(Action::Reduce(rule))
}

pub(super) fn production_length(rule: u16) -> Result<usize, LegacySyntaxErrorKind> {
    if rule == 0 {
        return Err(LegacySyntaxErrorKind::InvalidTable);
    }
    usize::try_from(at(&legacy_productions::LENGTH, usize::from(rule))?)
        .map_err(|_| LegacySyntaxErrorKind::InvalidTable)
}

pub(super) fn goto(state: usize, rule: u16) -> Result<usize, LegacySyntaxErrorKind> {
    let lhs = at(&legacy_productions::LHS, usize::from(rule))? - 91;
    let lhs = usize::try_from(lhs).map_err(|_| LegacySyntaxErrorKind::InvalidTable)?;
    let base = at(&legacy_productions::GOTO_OFFSET, lhs)?;
    let native = i32::try_from(state).map_err(|_| LegacySyntaxErrorKind::InvalidTable)?;
    let index = i32::from(base) + native;
    let value = if let Ok(index) = usize::try_from(index)
        && index <= legacy_actions::LAST
        && i32::from(at(&legacy_actions::CHECK, index)?) == native
    {
        at(&legacy_actions::ACTION, index)?
    } else {
        at(&legacy_productions::DEFAULT_GOTO, lhs)?
    };
    let target = usize::try_from(value).map_err(|_| LegacySyntaxErrorKind::InvalidTable)?;
    if target >= legacy_actions::PACT.len() {
        return Err(LegacySyntaxErrorKind::InvalidTable);
    }
    Ok(target)
}

fn at(values: &[i16], index: usize) -> Result<i16, LegacySyntaxErrorKind> {
    values
        .get(index)
        .copied()
        .ok_or(LegacySyntaxErrorKind::InvalidTable)
}
