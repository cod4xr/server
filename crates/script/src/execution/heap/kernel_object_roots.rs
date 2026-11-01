// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelScriptRoots, KernelValue};
use crate::heap::{ObjectId, StringId};

pub(super) struct ObjectRoots {
    pub(super) objects: Vec<ObjectId>,
    pub(super) strings: Vec<StringId>,
    pub(super) bytes: u64,
}

pub(super) fn gather(
    context: &mut KernelHeapContext<'_>,
    values: impl Iterator<Item = KernelValue> + Clone,
) -> Result<ObjectRoots, KernelErrorKind> {
    let script_objects = context
        .script_roots
        .map(KernelScriptRoots::ordinary_objects);
    let mut objects = (context.external_objects.len() as u64)
        .checked_add(script_objects.map_or(0, |roots| roots.len() as u64))
        .ok_or(KernelErrorKind::SizeOverflow)?;
    let mut strings = (context.external_roots.len() as u64)
        .checked_add(context.literals.active.len() as u64)
        .ok_or(KernelErrorKind::SizeOverflow)?;
    for value in values.clone() {
        context.charge(1)?;
        match value {
            KernelValue::Object(_) => {
                objects = objects
                    .checked_add(1)
                    .ok_or(KernelErrorKind::SizeOverflow)?;
            }
            KernelValue::String(_) | KernelValue::LocalizedString(_) => {
                strings = strings
                    .checked_add(1)
                    .ok_or(KernelErrorKind::SizeOverflow)?;
            }
            KernelValue::Undefined
            | KernelValue::Number(_)
            | KernelValue::Vector(_)
            | KernelValue::Function(_)
            | KernelValue::Array(_) => {}
        }
    }
    let bytes = objects
        .checked_mul(size_of::<ObjectId>() as u64)
        .zip(strings.checked_mul(size_of::<StringId>() as u64))
        .and_then(|(objects, strings)| objects.checked_add(strings))
        .ok_or(KernelErrorKind::SizeOverflow)?;
    context.scratch(bytes)?;
    let mut roots = ObjectRoots {
        objects: reserve(objects)?,
        strings: reserve(strings)?,
        bytes,
    };
    for value in values {
        context.charge(1)?;
        match value {
            KernelValue::Object(id) => roots.objects.push(id),
            KernelValue::String(id) | KernelValue::LocalizedString(id) => roots.strings.push(id),
            KernelValue::Undefined
            | KernelValue::Number(_)
            | KernelValue::Vector(_)
            | KernelValue::Function(_)
            | KernelValue::Array(_) => {}
        }
    }
    for id in context.external_objects.iter().copied() {
        context.charge(1)?;
        roots.objects.push(id);
    }
    for id in script_objects.into_iter().flatten() {
        context.charge(1)?;
        roots.objects.push(id);
    }
    for id in context.external_roots.iter().copied() {
        context.charge(1)?;
        roots.strings.push(id);
    }
    for index in 0..context.literals.active.len() {
        context.charge(1)?;
        roots.strings.push(context.literals.active_id(index)?);
    }
    if roots.objects.len() as u64 != objects || roots.strings.len() as u64 != strings {
        return Err(KernelErrorKind::InvalidExecutionState);
    }
    Ok(roots)
}

fn reserve<T>(count: u64) -> Result<Vec<T>, KernelErrorKind> {
    let count = usize::try_from(count).map_err(|_| KernelErrorKind::SizeOverflow)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| KernelErrorKind::AllocationFailed)?;
    Ok(values)
}
