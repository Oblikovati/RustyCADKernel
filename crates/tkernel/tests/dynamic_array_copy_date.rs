//! Defined copyDate success and recoverable unpublished-block failure states.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray, DynamicArrayCopyFailure};
use rustycad_tkernel::standard::Failure;
use std::{cell::Cell, rc::Rc};
mod dynamic_array_copy_date {
    pub mod contracts;
    pub mod oracle;
}
#[derive(Default)]
struct CopyObservations {
    attempts: Cell<usize>,
    drops: Cell<usize>,
}
struct CopyDateItem {
    value: i32,
    observations: Rc<CopyObservations>,
}
impl CopyDateItem {
    fn copy_construct(&self, fail_at: usize) -> Result<Self, Failure> {
        let attempt = self.observations.attempts.get() + 1;
        self.observations.attempts.set(attempt);
        if attempt == fail_at {
            return Err(Failure::with_type("Standard_Failure", "copy failure", ""));
        }
        Ok(Self {
            value: self.value,
            observations: Rc::clone(&self.observations),
        })
    }
}
impl Drop for CopyDateItem {
    fn drop(&mut self) {
        self.observations
            .drops
            .set(self.observations.drops.get() + 1);
    }
}
fn source_items(count: i32) -> (DynamicArray<CopyDateItem>, Rc<CopyObservations>) {
    let observations = Rc::new(CopyObservations::default());
    let mut source = DynamicArray::with_increment(2);
    for value in 1..=count {
        source.append(CopyDateItem {
            value,
            observations: Rc::clone(&observations),
        });
    }
    source
        .storage_extension()
        .ensure_block_count(3, size_of::<CopyDateItem>())
        .unwrap();
    (source, observations)
}
fn target_items() -> (DynamicArray<CopyDateItem>, BaseAllocator) {
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let target = DynamicArray::with_allocator(2, Some(allocator.clone()));
    (target, allocator)
}
fn live_allocations(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count as usize)
        .sum()
}
