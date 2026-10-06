//! Native erased storage, callback observations and pointer-table contracts.
#![allow(unsafe_code)]
#[path = "sparse_array_base_contracts/exchange.rs"]
mod exchange_contracts;
#[path = "sparse_array_base_contracts/fixture.rs"]
mod fixture;
#[path = "sparse_array_base_contracts/resources.rs"]
mod resource_contracts;
use fixture::*;
use rustycad_tkernel::ncollection::{
    BaseAllocator, SparseArrayBase, SparseArrayBaseIterator, SparseArrayBlock,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn raw_block_layout_matches_native_stride_and_bitmap_byte_boundary() {
    let base = populated_sparse_base(12, 9, &[(8, 88), (9, 99)]);
    assert_native_row(&native_layout_row(&base));
    let address = base.block_address(0);
    let block = unsafe { base.get_block(address) };
    assert_eq!(SparseArrayBlock::bits_per_cell(), 8);
    assert_eq!(SparseArrayBlock::size(9, 12), 118);
    assert_eq!(unsafe { SparseArrayBlock::to_array(address) }, block.array);
}

#[test]
fn callbacks_observe_native_set_self_copy_and_unset_publication_order() {
    let base = SparseArrayBase::new(4, 4, discard_integer);
    let trace = SparseCallbackTrace::new(&base);
    trace.insert(1, 7);
    trace.insert(1, 9);
    trace.self_copy(1);
    unsafe {
        base.unset_with(1, |a| trace.record('D', a));
    }
    assert_native_row(&format!(
        "set_unset_states {} {} {}",
        trace.events.borrow(),
        base.size(),
        u8::from(base.block_address(0).is_null())
    ));
}

#[test]
fn assignment_distinguishes_slot_removal_and_whole_block_removal_timing() {
    let source = populated_sparse_base(4, 4, &[(0, 10), (8, 80)]);
    let target = populated_sparse_base(4, 4, &[(0, 1), (1, 11), (4, 4), (8, 8), (12, 12)]);
    let trace = SparseCallbackTrace::new(&target);
    unsafe {
        target.assign_with(
            &source,
            |a, b| trace.create(a, b),
            |a| trace.record('D', a),
            |a, b| trace.copy(a, b),
        );
    }
    assert_native_row(&format!(
        "assign_states {} {}",
        trace.events.borrow(),
        target.size()
    ));
}

#[test]
fn clear_preserves_block_count_and_bits_during_destruction() {
    let base = SparseArrayBase::new(4, 4, discard_integer);
    for (i, v) in [(0, 1), (1, 2), (5, 5)] {
        insert_integer(&base, i, v);
    }
    let trace = SparseCallbackTrace::new(&base);
    unsafe {
        base.clear_with(|a| trace.record('D', a));
    }
    assert_native_row(&format!(
        "clear_states {} {} {}",
        trace.events.borrow(),
        base.size(),
        base.capacity()
    ));
    unsafe {
        base.clear_items(discard_integer);
    }
}

#[test]
fn failed_create_retains_empty_block_and_unpublished_occupancy() {
    let base = SparseArrayBase::new(4, 4, discard_integer);
    let trace = SparseCallbackTrace::new(&base);
    trace.reject_create(9);
    let block = unsafe { base.get_block(base.block_address(2)) };
    assert_native_row(&format!(
        "failed_set_state {} {} {} {} {}",
        trace.events.borrow(),
        base.size(),
        base.capacity(),
        unsafe { *block.count },
        unsafe { *block.bits }
    ));
}

#[test]
fn exchange_moves_geometry_and_item_pointers() {
    let mut first = populated_sparse_base(4, 4, &[(0, 1)]);
    let mut second = populated_sparse_base(12, 9, &[(8, 8), (9, 9)]);
    let address = second.get_value(8).unwrap();
    unsafe {
        first.exchange(&mut second);
    }
    assert_eq!(first.get_value(8).unwrap(), address);
    assert_native_row(&native_exchange_row(&first, &second));
}

#[test]
fn table_growth_retains_item_addresses_and_zeroes_unconstructed_storage() {
    let base = SparseArrayBase::new(4, 4, discard_integer);
    insert_integer(&base, 0, 10);
    let address = base.get_value(0).unwrap();
    insert_integer(&base, 100, 100);
    assert_eq!(base.capacity(), 128);
    assert_eq!(base.get_value(0).unwrap(), address);
    let block = unsafe { base.get_block(base.block_address(0)) };
    assert_eq!(unsafe { *base.get_item(block, 1).cast::<i32>() }, 0);
    assert!(base.block_address(1).is_null());
    assert!(base.block_address(100).is_null());
    assert!(base.get_value(2).is_err());
}

#[test]
fn raw_bitmap_operations_return_native_masks_without_extra_count_changes() {
    let base = SparseArrayBase::new(4, 9, discard_integer);
    insert_integer(&base, 0, 10);
    let block = unsafe { base.get_block(base.block_address(0)) };
    unsafe {
        assert_eq!(block.set(8), 1);
        assert_eq!(block.set(8), 0);
        assert_eq!(block.is_set(8), 1);
        assert_eq!(block.unset(8), 1);
        assert_eq!(block.unset(8), 0);
        assert_eq!(block.set(7), 1);
        assert_eq!(block.is_set(7), 128);
        assert_eq!(block.unset(7), 128);
        assert_eq!(*block.count, 1);
    }
}

#[test]
fn assignment_of_different_block_geometry_reconstructs_and_self_assignment_is_noop() {
    let source = SparseArrayBase::new(4, 2, discard_integer);
    let target = SparseArrayBase::new(4, 4, discard_integer);
    insert_integer(&source, 0, 10);
    insert_integer(&target, 1, 1);
    unsafe {
        target.assign(&source, create_integer, discard_integer, copy_integer);
    }
    assert_eq!(
        (target.block_size(), target.capacity(), target.size()),
        (2, 4, 1)
    );
    assert_self_assignment_noop(&target);
    assert_eq!(unsafe { *target.get_value(0).unwrap().cast::<i32>() }, 10);
}

#[test]
fn failed_assignment_keeps_earlier_successful_copies_and_later_original_items() {
    let source = populated_sparse_base(4, 4, &[(0, 10), (2, 20)]);
    let target = populated_sparse_base(4, 4, &[(3, 33)]);
    assign_rejecting_twenty(&target, &source);
    assert_eq!(target.size(), 2);
    assert!(target.has_value(0));
    assert!(!target.has_value(2));
    assert!(target.has_value(3));
}

#[test]
fn legacy_base_iterator_traverses_raw_blocks_and_preserves_exhausted_offset() {
    let base = SparseArrayBase::new(4, 4, discard_integer);
    insert_integer(&base, 5, 50);
    let mut cursor = SparseArrayBaseIterator::new(&base);
    assert_eq!(cursor.index(), 5);
    assert_eq!(unsafe { *cursor.value().unwrap().cast::<i32>() }, 50);
    cursor.next();
    assert!(!cursor.more());
    assert_eq!(cursor.index(), 20);
    assert!(cursor.value().is_none());
    cursor.next();
    assert_eq!(cursor.index(), 20);
    cursor.restart();
    assert_eq!(cursor.index(), 5);
    let empty = SparseArrayBase::new(4, 0, discard_integer);
    cursor.initialize(&empty);
    assert_eq!(cursor.index(), 0);
    assert!(!cursor.more());
}

#[test]
fn retained_empty_table_and_zero_constructor_support_defined_queries() {
    let base = SparseArrayBase::with_allocator(4, 4, discard_integer, BaseAllocator::common());
    insert_integer(&base, 5, 50);
    unsafe {
        assert!(base.unset_value(5, discard_integer));
        assert!(!base.unset_value(5, discard_integer));
    }
    assert_eq!(SparseArrayBaseIterator::new(&base).index(), 16);
    let zero = SparseArrayBase::new(4, 0, discard_integer);
    assert_eq!((zero.size(), zero.capacity()), (0, 0));
    assert!(SparseArrayBaseIterator::default().value().is_none());
    unsafe {
        assert!((*zero.native_state()).table.is_null());
        zero.clear_items(discard_integer);
    }
}

#[test]
fn exchanged_owners_invoke_the_original_stored_destructor() {
    let first_count = std::cell::Cell::new(0);
    let second_count = std::cell::Cell::new(0);
    let mut record = DestructorCounters {
        first: &first_count,
        second: &second_count,
    };
    drop_exchanged_counters(&mut record);
    assert_native_row(&format!(
        "exchange_destructors {} {}",
        first_count.get(),
        second_count.get()
    ));
}

#[test]
fn drop_callbacks_can_observe_native_intermediate_counts() {
    let trace = std::cell::RefCell::new(Vec::new());
    {
        let base = SparseArrayBase::new(size_of::<ObservedSparseDrop>(), 4, destroy_observed_drop);
        let mut item = ObservedSparseDrop {
            state: unsafe { base.native_state() },
            trace: &trace,
        };
        let source = std::ptr::from_mut(&mut item).cast();
        unsafe {
            base.set_value(0, source, copy_observed_drop, copy_observed_drop);
            base.set_value(1, source, copy_observed_drop, copy_observed_drop);
        }
    }
    assert_eq!(*trace.borrow(), [(2, 2), (1, 2)]);
}

#[test]
fn assigned_empty_allocated_source_block_is_preserved_in_destination() {
    let source = SparseArrayBase::new(4, 4, discard_integer);
    let target = SparseArrayBase::new(4, 4, discard_integer);
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        source.set_value_with(
            9,
            std::ptr::null_mut(),
            |_, _| panic!("rejected"),
            |_, _| unreachable!(),
        );
    }));
    unsafe {
        target.assign(&source, create_integer, discard_integer, copy_integer);
    }
    assert_eq!((target.size(), target.capacity()), (0, 16));
    assert!(!target.block_address(2).is_null());
    assert_eq!(SparseArrayBaseIterator::new(&target).index(), 20);
}

#[test]
fn failed_copy_into_new_block_retains_successful_prefix() {
    let source = populated_sparse_base(4, 4, &[(0, 10), (2, 20)]);
    let target = SparseArrayBase::new(4, 4, discard_integer);
    assign_rejecting_twenty(&target, &source);
    assert_eq!((target.size(), target.capacity()), (1, 16));
    assert!(target.has_value(0));
    assert!(!target.has_value(2));
}
