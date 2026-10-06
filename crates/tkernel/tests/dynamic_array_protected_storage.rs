//! Protected block probes use real owned allocations and never read uninitialized capacity.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray, DynamicArrayStorageExtension};
use std::{cell::Cell, fmt::Write};
mod dynamic_array_protected_storage {
    pub mod contracts;
    pub mod oracle;
}
thread_local! {
    static ELEMENT_LIFETIMES: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}
struct EngineItem {
    value: i32,
}
impl EngineItem {
    fn new(value: i32) -> Self {
        ELEMENT_LIFETIMES.with(|counts| {
            let (constructed, destroyed) = counts.get();
            counts.set((constructed + 1, destroyed));
        });
        Self { value }
    }
}
impl Drop for EngineItem {
    fn drop(&mut self) {
        ELEMENT_LIFETIMES.with(|counts| {
            let (constructed, destroyed) = counts.get();
            counts.set((constructed, destroyed + 1));
        });
    }
}
#[derive(Default)]
struct AllocationObservations {
    live: usize,
    allocations: usize,
    frees: usize,
}
impl AllocationObservations {
    fn observe(&mut self, allocator: &BaseAllocator) {
        let live = live_allocations(allocator);
        if live >= self.live {
            self.allocations += live - self.live;
        } else {
            self.frees += self.live - live;
        }
        self.live = live;
    }
}
fn output_values(output: &mut String, name: &str, values: &[usize]) {
    output.push_str(name);
    for value in values {
        write!(output, " {value}").unwrap();
    }
    output.push('\n');
}
fn snapshot(
    array: &mut DynamicArray<EngineItem>,
    allocator: &BaseAllocator,
    allocations: &mut AllocationObservations,
) -> Vec<usize> {
    allocations.observe(allocator);
    let size = array.size();
    let storage = array.storage_extension();
    let (constructed, destroyed) = ELEMENT_LIFETIMES.with(Cell::get);
    vec![
        storage.block_count(),
        storage.available_size(),
        size,
        constructed,
        destroyed,
        allocations.allocations,
        allocations.frees,
    ]
}

fn live_allocations(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count as usize)
        .sum()
}
