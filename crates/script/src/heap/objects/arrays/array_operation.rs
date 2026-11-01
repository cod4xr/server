// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::arrays::ArrayResult;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{HeapWorkBudget, ObjectHeapError, ObjectHeapErrorKind};

pub(super) fn run<T>(
    budget: HeapWorkBudget,
    apply: impl FnOnce(&mut ObjectWork) -> Result<T, ObjectHeapErrorKind>,
) -> Result<ArrayResult<T>, ObjectHeapError> {
    let mut work = ObjectWork::new(budget);
    let value = apply(&mut work).map_err(|kind| work.error(kind))?;
    Ok(ArrayResult {
        value,
        usage: work.usage(),
    })
}
