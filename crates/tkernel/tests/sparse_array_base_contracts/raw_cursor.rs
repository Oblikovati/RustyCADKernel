//! Protected native value() exposes cached raw addresses independently from occupied value().
use super::fixture::{discard_integer, populated_sparse_base};
use rustycad_tkernel::ncollection::{SparseArrayBase, SparseArrayBaseIterator, SparseArrayBlock};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};

fn raw_cursor_oracle(actual: &str) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_sparse_raw_iterator_addresses.expected");
    assert!(
        expected.lines().any(|line| line == actual),
        "row {actual:?}; expected native raw iterator row"
    );
}

fn raw_cursor_live_row(label: &str, cursor: &SparseArrayBaseIterator<'_>) {
    assert!(cursor.more());
    let value = unsafe { *cursor.native_value_address().unwrap().cast::<i32>() };
    raw_cursor_oracle(&format!(
        "{label} {} {} {value}",
        u8::from(cursor.more()),
        cursor.index()
    ));
}

fn raw_cursor_address_row(
    label: &str,
    cursor: &SparseArrayBaseIterator<'_>,
    expected: *mut c_void,
) {
    let address = unsafe { cursor.native_value_address() }.unwrap();
    raw_cursor_oracle(&format!(
        "{label} {} {} {}",
        u8::from(cursor.more()),
        cursor.index(),
        u8::from(address == expected)
    ));
}

#[test]
fn raw_cursor_exhaustion_keeps_bitmap_boundary_address() {
    let base = populated_sparse_base(4, 4, &[(5, 50)]);
    let mut cursor = SparseArrayBaseIterator::from_optional(Some(&base));
    raw_cursor_live_row("live", &cursor);
    let block = unsafe { base.get_block(base.block_address(1)) };
    cursor.next();
    assert!(cursor.value().is_none());
    raw_cursor_address_row("exhausted_address", &cursor, block.bits.cast());
    cursor.next();
    raw_cursor_address_row("exhausted_noop", &cursor, block.bits.cast());
    cursor.restart();
    raw_cursor_live_row("restart_address", &cursor);
}

#[test]
fn raw_cursor_supports_null_construction_detachment_and_rebinding() {
    let base = populated_sparse_base(4, 4, &[(5, 50)]);
    let mut cursor = SparseArrayBaseIterator::from_optional(None);
    assert!(unsafe { cursor.native_value_address() }.is_none());
    raw_cursor_oracle(&format!("detached {}", u8::from(cursor.more())));
    cursor.initialize_optional(Some(&base));
    cursor.next();
    cursor.initialize_optional(None);
    cursor.next();
    cursor.restart();
    assert!(unsafe { cursor.native_value_address() }.is_none());
    raw_cursor_oracle(&format!("detached_again {}", u8::from(cursor.more())));
    cursor.initialize_optional(Some(&base));
    raw_cursor_live_row("rebound", &cursor);
}

#[test]
fn raw_cursor_empty_rebinding_retains_previous_live_block_cache() {
    let base = populated_sparse_base(4, 4, &[(5, 50)]);
    let empty = SparseArrayBase::new(4, 4, discard_integer);
    let block = unsafe { base.get_block(base.block_address(1)) };
    let mut cursor = SparseArrayBaseIterator::new(&base);
    cursor.initialize_optional(Some(&empty));
    assert!(cursor.value().is_none());
    raw_cursor_address_row("empty_retains_cache", &cursor, block.array);
}

#[test]
fn raw_cursor_failed_construction_still_has_cached_block_address() {
    let base = SparseArrayBase::new(4, 4, discard_integer);
    fail_raw_cursor_construction(&base);
    let cursor = SparseArrayBaseIterator::new(&base);
    let block = unsafe { base.get_block(base.block_address(2)) };
    assert!(cursor.value().is_none());
    raw_cursor_address_row("failed_block_address", &cursor, block.bits.cast());
}

fn fail_raw_cursor_construction(base: &SparseArrayBase) {
    let failure = catch_unwind(AssertUnwindSafe(|| unsafe {
        base.set_value_with(
            9,
            std::ptr::null_mut(),
            |_, _| panic!("rejected"),
            |_, _| unreachable!(),
        );
    }));
    assert!(failure.is_err());
}

#[test]
fn raw_cursor_cross_block_traversal_updates_cached_address() {
    let base = populated_sparse_base(4, 4, &[(5, 50), (9, 90)]);
    let mut cursor = SparseArrayBaseIterator::new(&base);
    cursor.next();
    raw_cursor_live_row("cross_block", &cursor);
    cursor.next();
    let block = unsafe { base.get_block(base.block_address(2)) };
    raw_cursor_address_row("cross_block_end", &cursor, block.bits.cast());
}

#[test]
fn private_native_block_members_are_reachable_through_inferred_return_type() {
    let base = populated_sparse_base(4, 4, &[(0, 10)]);
    let address = base.block_address(0);
    let block = unsafe { base.get_block(address) };
    let projected = unsafe { SparseArrayBlock::new(address, 4, 4) };
    let (set, present, unset) = unsafe { (block.set(1), block.is_set(1), block.unset(1)) };
    raw_cursor_oracle(&format!(
        "private_block_reachable {} {} {} {} {} {} {} {set} {present} {unset} {}",
        unsafe { *block.count },
        u8::from(block.array == projected.array),
        u8::from(block.bits == projected.bits),
        size_of::<u8>(),
        SparseArrayBlock::bits_per_cell(),
        SparseArrayBlock::size(4, 4),
        u8::from(unsafe { SparseArrayBlock::to_array(address) } == block.array),
        unsafe { block.is_set(1) }
    ));
}
