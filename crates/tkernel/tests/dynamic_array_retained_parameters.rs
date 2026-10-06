//! Retained parameter copying preserves physical bytes and transfers initialized-range obligations.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray, DynamicArrayStorageExtension};
use std::{cell::Cell, rc::Rc};
mod dynamic_array_retained_parameters {
    pub mod contracts;
    pub mod oracle;
}
#[derive(Default)]
struct ParameterCounts {
    constructed: Cell<usize>,
    destroyed: Cell<usize>,
}
struct ParameterItem {
    value: i32,
    counts: Rc<ParameterCounts>,
}
impl ParameterItem {
    fn new(value: i32, counts: &Rc<ParameterCounts>) -> Self {
        counts.constructed.set(counts.constructed.get() + 1);
        Self {
            value,
            counts: Rc::clone(counts),
        }
    }
}
impl Drop for ParameterItem {
    fn drop(&mut self) {
        self.counts.destroyed.set(self.counts.destroyed.get() + 1);
    }
}
fn allocator_live(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count as usize)
        .sum()
}
fn retained_values(target: &DynamicArray<ParameterItem>) -> Vec<i32> {
    target.iter().map(|item| item.value).collect()
}
