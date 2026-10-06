#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::SparseArrayBase;
use std::{cell::RefCell, ffi::c_void};
pub unsafe fn discard_integer(_: *mut c_void) {}
pub unsafe fn create_integer(item: *mut c_void, source: *mut c_void) {
    unsafe {
        item.cast::<i32>().write(*source.cast::<i32>());
    }
}
pub unsafe fn copy_integer(item: *mut c_void, source: *mut c_void) {
    unsafe {
        std::ptr::copy(source.cast::<i32>(), item.cast::<i32>(), 1);
    }
}
pub fn insert_integer(base: &SparseArrayBase, index: usize, value: i32) {
    let mut value = value;
    unsafe {
        base.set_value(
            index,
            std::ptr::from_mut(&mut value).cast(),
            create_integer,
            copy_integer,
        );
    }
}
pub fn assert_native_row(actual: &str) {
    let expected = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/oracle/tkernel_sparse_array_base_contracts.expected"
    ));
    assert!(
        expected.lines().any(|row| row == actual),
        "row {actual:?}; expected exact native output"
    );
}
pub struct SparseCallbackTrace<'a> {
    pub owner: &'a SparseArrayBase,
    pub events: RefCell<String>,
}
impl<'a> SparseCallbackTrace<'a> {
    pub fn new(owner: &'a SparseArrayBase) -> Self {
        Self {
            owner,
            events: RefCell::new(String::new()),
        }
    }
    pub fn record(&self, event: char, item: *mut c_void) {
        let index = self.index_of(item);
        let block = unsafe {
            self.owner
                .get_block(self.owner.block_address(index / self.owner.block_size()))
        };
        self.events.borrow_mut().push_str(&format!(
            "{event}{index}:{}:{}:{},",
            self.owner.size(),
            unsafe { *block.count },
            u8::from(self.owner.has_value(index))
        ));
    }
    fn index_of(&self, item: *mut c_void) -> usize {
        for number in 0..self.owner.block_slots() {
            if self.owner.block_address(number).is_null() {
                continue;
            }
            let block = unsafe { self.owner.get_block(self.owner.block_address(number)) };
            for offset in 0..self.owner.block_size() {
                if unsafe { self.owner.get_item(block, offset) } == item {
                    return number * self.owner.block_size() + offset;
                }
            }
        }
        panic!("item {item:p}; expected a slot owned by callback observer");
    }
    pub fn create(&self, item: *mut c_void, source: *mut c_void) {
        self.record('C', item);
        unsafe { create_integer(item, source) };
    }
    pub fn copy(&self, item: *mut c_void, source: *mut c_void) {
        self.record(if item == source { 'S' } else { 'A' }, item);
        unsafe { copy_integer(item, source) };
    }
}

#[derive(Clone, Copy)]
pub struct DestructorCounters {
    pub first: *const std::cell::Cell<usize>,
    pub second: *const std::cell::Cell<usize>,
}
pub unsafe fn copy_counters(item: *mut c_void, source: *mut c_void) {
    unsafe {
        item.cast::<DestructorCounters>()
            .write(*source.cast::<DestructorCounters>());
    }
}
pub unsafe fn destroy_first(item: *mut c_void) {
    let counter = unsafe { &*(*item.cast::<DestructorCounters>()).first };
    counter.set(counter.get() + 1);
}
pub unsafe fn destroy_second(item: *mut c_void) {
    let counter = unsafe { &*(*item.cast::<DestructorCounters>()).second };
    counter.set(counter.get() + 1);
}

#[derive(Clone, Copy)]
pub struct ObservedSparseDrop {
    pub state: *const rustycad_tkernel::ncollection::SparseArrayBaseState,
    pub trace: *const RefCell<Vec<(usize, usize)>>,
}
pub unsafe fn copy_observed_drop(item: *mut c_void, source: *mut c_void) {
    unsafe {
        item.cast::<ObservedSparseDrop>()
            .write(*source.cast::<ObservedSparseDrop>());
    }
}
pub unsafe fn destroy_observed_drop(item: *mut c_void) {
    let record = unsafe { *item.cast::<ObservedSparseDrop>() };
    let state = unsafe { &*record.state };
    let block_count = unsafe { *(*state.table).cast::<usize>() };
    unsafe { &*record.trace }
        .borrow_mut()
        .push((state.size, block_count));
}

pub fn populated_sparse_base(
    stride: usize,
    increment: usize,
    values: &[(usize, i32)],
) -> SparseArrayBase {
    let base = SparseArrayBase::new(stride, increment, discard_integer);
    for &(index, value) in values {
        insert_integer(&base, index, value);
    }
    base
}
impl SparseCallbackTrace<'_> {
    pub fn insert(&self, index: usize, mut value: i32) {
        let source = std::ptr::from_mut(&mut value).cast();
        unsafe {
            self.owner.set_value_with(
                index,
                source,
                |a, b| self.create(a, b),
                |a, b| self.copy(a, b),
            );
        }
    }
    pub fn self_copy(&self, index: usize) {
        let source = self.owner.get_value(index).unwrap();
        unsafe {
            self.owner.set_value_with(
                index,
                source,
                |a, b| self.create(a, b),
                |a, b| self.copy(a, b),
            );
        }
    }
    pub fn reject_create(&self, index: usize) {
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            self.owner.set_value_with(
                index,
                std::ptr::null_mut(),
                |a, _| {
                    self.record('C', a);
                    panic!("rejected44");
                },
                |_, _| unreachable!(),
            );
        }));
        assert!(failure.is_err());
    }
}
pub fn native_layout_row(base: &SparseArrayBase) -> String {
    let address = base.block_address(0);
    let block = unsafe { base.get_block(address) };
    let array_offset = unsafe { block.array.cast::<u8>().offset_from(address.cast()) };
    let bit_offset = unsafe { block.bits.offset_from(address.cast()) };
    let item_offset = unsafe {
        base.get_item(block, 8)
            .cast::<u8>()
            .offset_from(block.array.cast())
    };
    let count = unsafe { *block.count };
    let bit0 = unsafe { *block.bits };
    let bit1 = unsafe { *block.bits.add(1) };
    format!(
        "layout {} {} {array_offset} {bit_offset} {count} {bit0} {bit1} {item_offset}",
        base.block_slots(),
        base.capacity()
    )
}
pub fn native_exchange_row(first: &SparseArrayBase, second: &SparseArrayBase) -> String {
    format!(
        "exchange_layout {} {} {} {} {} {}",
        first.item_size(),
        first.block_size(),
        first.size(),
        second.item_size(),
        second.block_size(),
        second.size()
    )
}
pub fn assert_self_assignment_noop(base: &SparseArrayBase) {
    unsafe {
        base.assign_with(
            base,
            |_, _| panic!("self create"),
            |_| panic!("self destroy"),
            |_, _| panic!("self copy"),
        );
    }
}
pub fn assign_rejecting_twenty(target: &SparseArrayBase, source: &SparseArrayBase) {
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        target.assign_with(
            source,
            |a, b| {
                assert_ne!(*b.cast::<i32>(), 20);
                create_integer(a, b);
            },
            |_| {},
            |a, b| copy_integer(a, b),
        );
    }));
    assert!(failure.is_err());
}
pub fn drop_exchanged_counters(record: &mut DestructorCounters) {
    let mut first = SparseArrayBase::new(size_of::<DestructorCounters>(), 4, destroy_first);
    let mut second = SparseArrayBase::new(size_of::<DestructorCounters>(), 9, destroy_second);
    let source = std::ptr::from_mut(record).cast();
    unsafe {
        first.set_value(0, source, copy_counters, copy_counters);
        second.set_value(8, source, copy_counters, copy_counters);
        second.set_value(9, source, copy_counters, copy_counters);
        first.exchange(&mut second);
    }
}
