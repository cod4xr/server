// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::HeapWorkBudget;
use crate::heap::objects::object_allocation_plan::ObjectAllocationPlan;
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::objects::{
    ObjectAllocation, ObjectHeap, ObjectHeapError, ObjectHeapErrorKind, ObjectId,
};

pub(super) fn allocate(
    heap: &mut ObjectHeap,
    budget: HeapWorkBudget,
) -> Result<ObjectAllocation, ObjectHeapError> {
    let mut work = ObjectWork::new(budget);
    let id = insert(heap, &mut work).map_err(|kind| work.error(kind))?;
    Ok(ObjectAllocation {
        id,
        usage: work.usage(),
    })
}

fn insert(heap: &mut ObjectHeap, work: &mut ObjectWork) -> Result<ObjectId, ObjectHeapErrorKind> {
    let mut plan = ObjectAllocationPlan::new(heap, work)?;
    plan.reserve(heap, work)?;
    Ok(plan.publish(heap))
}
