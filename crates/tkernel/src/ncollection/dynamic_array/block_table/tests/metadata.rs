use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(super) fn descriptor_rows() -> String {
    let allocator = table_allocator();
    let mut target = DynamicArray::with_allocator(2, Some(allocator.clone()));
    let mut output = table_row("table_empty", &target, &allocator);
    for value in 1..=5 {
        target.append(value);
        if value != 4 {
            output += &table_row(&format!("table_append{value}"), &target, &allocator);
        }
    }
    output += &reserve_rows(&mut target, &allocator);
    output += &release_rows(&mut target, &allocator);
    output
}
fn ensure_blocks<T>(target: &mut DynamicArray<T>, count: usize) {
    target
        .storage_extension()
        .ensure_block_count(count, size_of::<T>())
        .unwrap();
}
fn reserve_rows(target: &mut DynamicArray<i32>, allocator: &BaseAllocator) -> String {
    ensure_blocks(target, 6);
    let mut output = table_row("table_reserve6", target, allocator);
    ensure_blocks(target, 9);
    output += &table_row("table_reserve9", target, allocator);
    target.clear(false);
    output += &table_row("table_clear_retain", target, allocator);
    ensure_blocks(target, 5);
    output += &table_row("table_smaller_reserve", target, allocator);
    target.append(42);
    output + &table_row("table_reuse", target, allocator)
}
fn release_rows(target: &mut DynamicArray<i32>, allocator: &BaseAllocator) -> String {
    target.clear(true);
    let mut output = table_row("table_clear_release", target, allocator);
    target
        .storage_extension()
        .expand_block(size_of::<i32>())
        .unwrap();
    output += &table_row("table_expand_empty", target, allocator);
    target.storage_extension().clear_blocks();
    output += &table_row("table_clear_blocks", target, allocator);
    ensure_blocks(target, 6);
    output + &table_row("table_initial_reserve6", target, allocator)
}
pub(super) fn copy_rows() -> String {
    let allocator = table_allocator();
    let mut source = DynamicArray::with_allocator(2, Some(allocator.clone()));
    ensure_blocks(&mut source, 7);
    source.append(7_i32);
    let mut output = table_row("copy_source", &source, &allocator);
    let mut copied = source.clone();
    output += &table_row("copy_constructed", &copied, &allocator);
    copied.clear(true);
    output += &table_row("copy_cleared", &copied, &allocator);
    ensure_blocks(&mut copied, 10);
    output += &table_row("copy_destination_reserved", &copied, &allocator);
    copied.assign(&source, true);
    output += &table_row("copy_assigned", &copied, &allocator);
    output + &move_rows(&mut copied, &allocator)
}
fn move_rows(copied: &mut DynamicArray<i32>, allocator: &BaseAllocator) -> String {
    let mut moved = DynamicArray::default();
    moved.move_from(copied);
    table_row("move_constructed", &moved, allocator) + &table_row("move_source", copied, allocator)
}
pub(super) fn failure_rows() -> String {
    let allocator = table_allocator();
    let mut target = DynamicArray::with_allocator(2, Some(allocator.clone()));
    let failure = catch_unwind(AssertUnwindSafe(|| {
        target.emplace_append(|| panic!("constructor"));
    }));
    assert!(failure.is_err());
    let mut output = table_row("append_constructor_failed", &target, &allocator);
    target.emplace_append(|| 5_i32);
    output += &table_row("append_constructor_retry", &target, &allocator);
    target.clear(true);
    output + &table_row("append_constructor_cleared", &target, &allocator)
}
