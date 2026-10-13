// SPDX-License-Identifier: GPL-2.0-only
//! Benchmark-only call accounting; allocation policy remains the system allocator.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
static CALLS: AtomicU64 = AtomicU64::new(0);
pub struct WindingBenchmarkAllocator;
// SAFETY: this wrapper preserves every System layout/pointer contract and only
// observes calls. It does not access allocated objects or alter their lifetimes.
// This narrow benchmark-only exception is required by the System delegation contract.
#[allow(unsafe_code)]
unsafe impl GlobalAlloc for WindingBenchmarkAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        CALLS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the caller supplies the unchanged GlobalAlloc layout contract.
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        CALLS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: preserve System's actual zeroed allocation policy as well.
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: this pointer and layout are forwarded unchanged to System.
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        CALLS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the caller's original System allocation provenance is unchanged.
        unsafe { System.realloc(pointer, layout, size) }
    }
}
pub fn current_calls() -> u64 {
    CALLS.load(Ordering::Relaxed)
}
