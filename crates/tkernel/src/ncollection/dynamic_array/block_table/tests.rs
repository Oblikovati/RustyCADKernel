use super::super::super::{BaseAllocator, DynamicArray};
use std::{cell::RefCell, rc::Rc};
mod metadata;
mod phases;

const EXPECTED: &str =
    include_str!("../../../../../../tools/oracle/tkernel_dynamic_block_table.expected");
fn table_allocator() -> BaseAllocator {
    BaseAllocator::accumulating(4096).unwrap()
}
fn live_blocks(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count as usize)
        .sum()
}
fn table_row<T>(name: &str, target: &DynamicArray<T>, allocator: &BaseAllocator) -> String {
    format!(
        "{name} {} {} {} {} {}\n",
        target.size(),
        target.blocks.len(),
        target.blocks.capacity(),
        target.blocks.len() * target.increment,
        live_blocks(allocator)
    )
}
fn expected_group(prefixes: &[&str]) -> String {
    EXPECTED
        .lines()
        .filter(|line| prefixes.iter().any(|prefix| line.starts_with(prefix)))
        .map(|line| format!("{line}\n"))
        .collect()
}
#[test]
fn descriptor_capacity_and_release_rows_match_native() {
    assert_eq!(metadata::descriptor_rows(), expected_group(&["table_"]));
}
#[test]
fn copy_and_move_descriptor_capacity_rows_match_native() {
    assert_eq!(metadata::copy_rows(), expected_group(&["copy_", "move_"]));
}
#[test]
fn append_failure_retains_published_empty_block_and_table() {
    assert_eq!(metadata::failure_rows(), expected_group(&["append_"]));
}
#[test]
fn all_destructors_precede_any_block_release() {
    let rows = phases::clear_case("clear_retain_phases", false, false)
        + &phases::clear_case("clear_release_phases", true, false)
        + &phases::clear_case("destructor_phases", true, true);
    assert_eq!(rows, expected_group(&["clear_", "destructor_"]));
}
