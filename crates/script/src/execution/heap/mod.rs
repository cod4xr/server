// SPDX-License-Identifier: AGPL-3.0-or-later
mod kernel_heap_accounting;
mod kernel_heap_collection;
mod kernel_heap_context;
mod kernel_heap_error;
mod kernel_heap_operations;
mod kernel_heap_usage;
mod kernel_literal_cache;
mod kernel_object_allocation;
mod kernel_object_collection;
mod kernel_object_error;
mod kernel_object_roots;
mod kernel_object_validation;
mod kernel_string_access;
mod kernel_string_allocation;
mod kernel_string_operations;

pub use kernel_heap_context::KernelHeapContext;
pub(super) use kernel_heap_operations::{apply, validate_arguments};
pub use kernel_heap_usage::KernelHeapUsage;
