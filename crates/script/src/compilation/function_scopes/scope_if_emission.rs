// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use super::scope_emission::with_plan;
use super::{ScopeControl, invalid};
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;

pub(in crate::compilation) fn begin_if(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::If {
            parent,
            consequence,
            ..
        } = plan.control(builder, span.clone())?
        else {
            return Err(invalid(span));
        };
        plan.enter(builder, parent, consequence, span)
    })
}

pub(in crate::compilation) fn end_consequence(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::If {
            parent,
            consequence,
            ..
        } = plan.control(builder, span.clone())?
        else {
            return Err(invalid(span));
        };
        if plan.current != consequence {
            return Err(invalid(span));
        }
        if builder.reachable {
            plan.clear_current(builder, span)?;
        }
        plan.current = parent;
        Ok(())
    })
}

pub(in crate::compilation) fn begin_else(
    builder: &mut FunctionBuilder<'_, '_>,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::If {
            parent,
            alternative: Some(alternative),
            ..
        } = plan.control(builder, span.clone())?
        else {
            return Err(invalid(span));
        };
        plan.enter(builder, parent, alternative, span)
    })
}

pub(in crate::compilation) fn end_if(
    builder: &mut FunctionBuilder<'_, '_>,
    consequence_reaches: bool,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    with_plan(builder, |plan, builder| {
        let ScopeControl::If {
            parent,
            consequence,
            alternative: Some(alternative),
        } = plan.control(builder, span.clone())?
        else {
            return Err(invalid(span));
        };
        if plan.current != alternative {
            return Err(invalid(span));
        }
        let alternative_reaches = builder.reachable;
        if alternative_reaches {
            plan.clear_current(builder, span.clone())?;
        }
        let created = match (consequence_reaches, alternative_reaches) {
            (true, true) => plan
                .block(consequence, span.clone())?
                .created
                .min(plan.block(alternative, span.clone())?.created),
            (true, false) => plan.block(consequence, span.clone())?.created,
            (false, true) => plan.block(alternative, span.clone())?.created,
            (false, false) => plan.block(parent, span.clone())?.created,
        };
        plan.block_mut(parent, span)?.created = created;
        plan.current = parent;
        Ok(())
    })
}
