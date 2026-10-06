//! Raw lifetime contracts keep detached allocation ownership separate from typed Vec blocks.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray, DynamicArrayStorageExtension};
use std::{
    cell::Cell,
    fmt::Write,
    panic::{AssertUnwindSafe, catch_unwind},
};
mod dynamic_array_raw_lifetime {
    pub mod contracts;
    pub mod oracle;
}
thread_local! {
    static RAW_LIFETIMES: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}
struct RawItem {
    value: i32,
}
impl RawItem {
    fn new(value: i32) -> Self {
        RAW_LIFETIMES.with(|counts| {
            let (attempts, destroyed) = counts.get();
            counts.set((attempts + 1, destroyed));
        });
        assert_ne!(value, 99, "constructor value99: expected injected failure");
        Self { value }
    }
}
impl Drop for RawItem {
    fn drop(&mut self) {
        RAW_LIFETIMES.with(|counts| {
            let (attempts, destroyed) = counts.get();
            counts.set((attempts, destroyed + 1));
        });
    }
}
struct RawAllocationObservations {
    live: usize,
    free: usize,
    allocations: usize,
    frees: usize,
    bytes: usize,
}
fn allocator_state(allocator: &BaseAllocator) -> (usize, usize) {
    let chain = allocator.accumulating_chain();
    let live = chain.iter().map(|block| block.alloc_count as usize).sum();
    let free = chain.iter().map(|block| block.free_size).sum();
    (live, free)
}
impl RawAllocationObservations {
    fn new(allocator: &BaseAllocator) -> Self {
        let (live, free) = allocator_state(allocator);
        Self {
            live,
            free,
            allocations: 0,
            frees: 0,
            bytes: 0,
        }
    }
    fn observe(&mut self, allocator: &BaseAllocator) {
        let (live, free) = allocator_state(allocator);
        if live > self.live {
            self.allocations += live - self.live;
            self.bytes += self.free - free;
        } else {
            self.frees += self.live - live;
        }
        self.live = live;
        self.free = free;
    }
}
fn output_row(output: &mut String, name: &str, values: &[usize]) {
    output.push_str(name);
    for value in values {
        write!(output, " {value}").unwrap();
    }
    output.push('\n');
}
fn raw_snapshot(
    array: &mut DynamicArray<RawItem>,
    allocator: &BaseAllocator,
    counts: &mut RawAllocationObservations,
) -> Vec<usize> {
    counts.observe(allocator);
    let (attempts, destroyed) = RAW_LIFETIMES.with(Cell::get);
    vec![
        array.size(),
        array.storage_extension().block_count(),
        counts.allocations,
        counts.frees,
        counts.bytes,
        attempts,
        destroyed,
    ]
}
fn raw_values(array: &DynamicArray<RawItem>) -> Vec<usize> {
    array.iter().map(|item| item.value as usize).collect()
}
