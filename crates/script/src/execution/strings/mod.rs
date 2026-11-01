// SPDX-License-Identifier: AGPL-3.0-or-later
mod kernel_heap_collection;
mod kernel_heap_context;
mod kernel_heap_error;
mod kernel_heap_usage;
mod kernel_literal_cache;
mod kernel_string_allocation;
mod kernel_string_operations;

pub use kernel_heap_context::KernelHeapContext;
pub use kernel_heap_usage::KernelHeapUsage;
pub(super) use kernel_string_operations::{apply, validate_arguments};
