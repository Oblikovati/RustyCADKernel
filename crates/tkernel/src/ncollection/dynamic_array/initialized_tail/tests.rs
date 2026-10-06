use super::super::{BaseAllocator, DynamicArray};
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
mod native;
mod panics;
mod transitions;

fn assert_tail_state<T>(target: &DynamicArray<T>) -> usize {
    let exact = exact_tail(target);
    let mut inspected = 0;
    let lengths = target
        .blocks
        .iter()
        .map(|block| block.len())
        .inspect(|_| inspected += 1);
    assert_eq!(target.initialized_tail_in(lengths), exact);
    assert_eq!(target.has_initialized_tail(), exact);
    if !target.needs_tail_scan {
        assert_eq!(inspected, 0);
    }
    inspected
}
fn reserve_blocks<T>(target: &mut DynamicArray<T>, count: usize) {
    target
        .storage_extension()
        .ensure_block_count(count, size_of::<T>())
        .unwrap();
}
fn live_allocations(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count as usize)
        .sum()
}

fn exact_tail<T>(target: &DynamicArray<T>) -> bool {
    target.blocks.iter().enumerate().any(|(index, block)| {
        block.len()
            > target
                .used
                .saturating_sub(index * target.increment)
                .min(target.increment)
    })
}
