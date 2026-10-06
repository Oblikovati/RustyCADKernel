//! Native allocation identity and reallocation ownership, rather than a Vec capacity simulation.
use rustycad_tkernel::standard::memory::{self, StandardMemory};

fn assert_storage_row(actual: &str) {
    let fixture = include_str!("../../../tools/oracle/tkernel_standard_memory_storage.expected");
    assert!(
        fixture.lines().any(|row| row == actual),
        "missing native row: {actual}"
    );
}

fn zero_owner(optimal: bool) -> StandardMemory {
    if optimal {
        return memory::allocate_optimal(0).unwrap();
    }
    memory::allocate(0).unwrap()
}

#[test]
fn simultaneous_zero_byte_allocations_are_real_distinct_aligned_blocks() {
    for optimal in [false, true] {
        let first = zero_owner(optimal);
        let second = zero_owner(optimal);
        let (left, right) = (
            first.as_slice().as_ptr().addr(),
            second.as_slice().as_ptr().addr(),
        );
        assert_storage_row(&format!(
            "zero-{} {} {} {} {} {}",
            u8::from(optimal),
            u8::from(left != 0),
            u8::from(right != 0),
            u8::from(left != right),
            u8::from(left % 16 == 0),
            u8::from(right % 16 == 0)
        ));
    }
}

#[test]
fn zero_byte_aligned_owners_follow_the_same_real_allocation_contract() {
    for alignment in [1, 2, 4, 8, 16, 64] {
        let first = memory::allocate_aligned(0, alignment).unwrap();
        let second = memory::allocate_aligned(0, alignment).unwrap();
        let expected = alignment.max(16);
        assert_storage_row(&format!(
            "aligned-zero-{alignment} {} {} {} {}",
            u8::from(first.address() != 0),
            u8::from(first.address() != second.address()),
            u8::from(first.address().is_multiple_of(expected)),
            u8::from(second.address().is_multiple_of(expected))
        ));
    }
}

#[test]
fn reallocation_of_null_to_zero_keeps_a_distinct_live_owner() {
    let (mut first, mut second) = (None, None);
    memory::reallocate(&mut first, 0).unwrap();
    memory::reallocate(&mut second, 0).unwrap();
    let (left, right) = (first.unwrap(), second.unwrap());
    let left = left.as_slice().as_ptr().addr();
    let right = right.as_slice().as_ptr().addr();
    assert_storage_row(&format!(
        "realloc-null-zero {} {} {}",
        u8::from(left != 0),
        u8::from(left != right),
        u8::from(left % 16 == 0)
    ));
}

#[test]
fn resizing_preserves_initialized_prefix_through_growth_and_shrink() {
    let mut slot = Some(memory::allocate(8).unwrap());
    slot.as_mut()
        .unwrap()
        .as_mut_slice()
        .copy_from_slice(b"abcdefgh");
    memory::reallocate(&mut slot, 32).unwrap();
    let growth = &slot.as_ref().unwrap().as_slice()[..8] == b"abcdefgh";
    memory::reallocate(&mut slot, 3).unwrap();
    assert_storage_row(&format!(
        "realloc-prefix {} {}",
        u8::from(growth),
        u8::from(slot.as_ref().unwrap().as_slice() == b"abc")
    ));
}

#[test]
fn positive_allocation_failure_preserves_old_pointer_and_bytes() {
    let mut slot = Some(memory::allocate(8).unwrap());
    slot.as_mut()
        .unwrap()
        .as_mut_slice()
        .copy_from_slice(b"abcdefgh");
    let before = slot.as_ref().unwrap().as_slice().as_ptr();
    let failure = memory::reallocate(&mut slot, usize::MAX).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfMemory");
    assert_eq!(slot.as_ref().unwrap().as_slice().as_ptr(), before);
    assert_storage_row(&format!(
        "realloc-positive-failure 1 {}",
        u8::from(slot.as_ref().unwrap().as_slice() == b"abcdefgh")
    ));
}

#[test]
fn existing_zero_size_reallocation_releases_owner_before_reporting_native_error() {
    let mut slot = Some(memory::allocate(8).unwrap());
    let failure = memory::reallocate(&mut slot, 0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfMemory");
    assert_storage_row(&format!(
        "realloc-existing-zero 1 {}",
        u8::from(slot.is_none())
    ));
}

#[test]
fn selected_native_manager_and_purge_policy_remain_unchanged() {
    let kind = memory::get_allocator_type() as i32;
    let purged = memory::purge();
    assert_storage_row(&format!("manager {kind} {purged}"));
}

#[path = "standard_memory_storage/ownership.rs"]
mod ownership;
