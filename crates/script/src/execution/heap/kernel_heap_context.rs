// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::execution::heap::KernelHeapUsage;
use crate::execution::heap::array_owners::KernelArrayOwners;
use crate::execution::heap::kernel_heap_error;
use crate::execution::heap::kernel_literal_cache::KernelLiteralCache;
use crate::execution::heap::kernel_object_validation;
use crate::execution::{KernelErrorKind, KernelScriptRoots, KernelValue};
use crate::heap::{
    HeapErrorKind, HeapWorkBudget, ObjectHeap, ObjectHeapUsage, ObjectId, StringHeap,
    StringHeapUsage, StringId,
};

/// Caller-owned string and optional shared-object context for the iterative call tree.
///
/// Copied IDs do not own storage. Supply all host handles that must survive VM
/// overwrites as external roots. Returned IDs stay in this borrowed heap and need
/// rooting in subsequent invocations. Invocation failures can retain allocations
/// or successful reclamation; individual store operations remain atomic.
/// Work accumulates when this context is reused; it never resets per callee.
/// Each invocation prepares a fresh admitted literal cache; cached-only strings
/// remain roots until the next invocation. Cache, owned-array registries, root
/// IDs and mark flags share the scratch ceiling. Dropping a cache does not drop payload.
///
/// Array arguments borrow host owners; entry cells clone separate managed
/// locations. Local/operand copies clone and moves transfer those locations.
/// Callee return releases other callee locations before the caller resumes.
/// A successful array result transfers one owner to the host; release it with
/// [`ObjectHeap::release_array_value`]. Raw Rust handle copies create no owner.
/// Error cleanup uses work reserved before publication, even when execution or
/// heap fuel is exhausted. Read-only indexing and size accept checked host-origin
/// arrays and independently constructed empty data (ADR 0087). Indexed writes and
/// stored array fields remain unsupported, so independent data cannot gain payload
/// edges. This profile does not implement nested/object logical lifetimes; another
/// populated origin requires a new ownership review.
/// Fixed script roots are opt-in through [`Self::with_script_roots`]. Ordinary
/// root-only graphs survive collection; game reads borrow the host binding and
/// create separately owned VM copies. Missing roots never fabricate world state.
pub struct KernelHeapContext<'heap> {
    pub(super) script_roots: Option<KernelScriptRoots>,
    pub(super) array_owners: KernelArrayOwners,
    pub(super) literals: KernelLiteralCache,
    pub(super) heap: &'heap mut StringHeap,
    pub(super) external_roots: &'heap [StringId],
    pub(super) objects: Option<&'heap mut ObjectHeap>,
    pub(super) external_objects: &'heap [ObjectId],
    pub(super) budget: HeapWorkBudget,
    pub(super) usage: KernelHeapUsage,
}

impl<'heap> KernelHeapContext<'heap> {
    /// Borrows storage and a complete external root inventory without allocation.
    /// Entry validation charges/checks roots before allocating execution frames.
    #[must_use]
    pub fn new(
        heap: &'heap mut StringHeap,
        external_roots: &'heap [StringId],
        budget: HeapWorkBudget,
    ) -> Self {
        Self {
            script_roots: None,
            array_owners: KernelArrayOwners::default(),
            literals: KernelLiteralCache::empty(),
            heap,
            external_roots,
            objects: None,
            external_objects: &[],
            budget,
            usage: KernelHeapUsage::default(),
        }
    }

    /// Adds shared struct storage and complete host object roots without allocation.
    /// Entry validation checks the bound string store and every supplied reference.
    /// Pressure reclamation traces all frame values and external object graphs,
    /// then strings; a later failure may retain a completed object collection.
    /// Backs checked structs and managed array transport. All live array locations
    /// already root their data in the object store, including host-owned locations.
    /// Nested array fields, source array operations and scheduling remain separate.
    #[must_use]
    pub fn with_objects(
        mut self,
        objects: &'heap mut ObjectHeap,
        external: &'heap [ObjectId],
    ) -> Self {
        self.objects = Some(objects);
        self.external_objects = external;
        self
    }

    /// Returns optional logical object-store usage without identities or payloads.
    #[must_use]
    pub fn object_heap_usage(&self) -> Option<ObjectHeapUsage> {
        self.objects.as_ref().map(|objects| objects.usage())
    }

    /// Returns charged global work/collection/scratch accounting, including failures.
    #[must_use]
    pub fn usage(&self) -> KernelHeapUsage {
        self.usage
    }

    /// Returns retained logical string-store usage without payload or identities.
    #[must_use]
    pub fn heap_usage(&self) -> StringHeapUsage {
        self.heap.usage()
    }

    /// Borrows explicit result bytes for a checked ID; this host read is not VM work.
    ///
    /// # Errors
    /// Rejects stale/foreign handles. No mutation or encoding conversion occurs.
    pub fn get(&self, id: StringId) -> Result<&[u8], HeapErrorKind> {
        self.heap.get(id)
    }

    pub(in crate::execution) fn validate(
        &mut self,
        arguments: &[KernelValue],
    ) -> Result<(), KernelErrorKind> {
        kernel_object_validation::binding(self)?;
        self.validate_script_roots()?;
        for value in arguments {
            self.charge(1)?;
            if let KernelValue::Object(id) = value {
                kernel_object_validation::reference(self, *id)?;
            }
            if let KernelValue::Array(id) = value {
                let budget = self.remaining(0)?;
                let result = self
                    .objects
                    .as_deref()
                    .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
                    .array_data(*id, budget);
                match result {
                    Ok(result) => self.record(result.usage, 0)?,
                    Err(error) => {
                        return Err(crate::execution::heap::kernel_object_error::record(
                            self, error, 0,
                        )?);
                    }
                }
            }
            if let Some(id) = value.string_id() {
                self.heap.get(id).map_err(kernel_heap_error::map)?;
            }
        }
        for index in 0..self.external_objects.len() {
            self.charge(1)?;
            kernel_object_validation::reference(self, self.external_objects[index])?;
        }
        for index in 0..self.external_roots.len() {
            self.charge(1)?;
            self.heap
                .get(self.external_roots[index])
                .map_err(kernel_heap_error::map)?;
        }
        Ok(())
    }

    pub(super) fn equal(
        &mut self,
        left: StringId,
        right: StringId,
    ) -> Result<bool, KernelErrorKind> {
        self.charge(4)?;
        if left == right {
            self.heap.get(left).map_err(kernel_heap_error::map)?;
            return Ok(true);
        }
        let left_length = self.heap.get(left).map_err(kernel_heap_error::map)?.len();
        let right_length = self.heap.get(right).map_err(kernel_heap_error::map)?.len();
        let visits = if left_length == right_length {
            left_length as u64
        } else {
            0
        };
        self.charge(visits)?;
        Ok(self.heap.get(left).map_err(kernel_heap_error::map)?
            == self.heap.get(right).map_err(kernel_heap_error::map)?)
    }
}

impl fmt::Debug for KernelHeapContext<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KernelHeapContext")
            .field("usage", &self.usage)
            .field("heap_usage", &self.heap.usage())
            .field("object_heap_usage", &self.object_heap_usage())
            .finish_non_exhaustive()
    }
}
