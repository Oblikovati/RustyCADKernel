//! OCCT 3d097a computes Block::Size with native unsigned wraparound.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    BaseAllocator, SparseArray, SparseArrayBase, SparseArrayBlock, SparseArrayBlockCell,
};
use std::{
    cell::Cell,
    ffi::{c_char, c_void},
    panic::{AssertUnwindSafe, catch_unwind},
};

const NATIVE_OVERFLOW_SIZE: usize = SparseArrayBlock::size(usize::MAX, 0);

#[test]
fn sparse_block_size_wraps_native_rounding_without_allocating() {
    assert_eq!(SparseArrayBlock::size(usize::MAX, 0), size_of::<usize>());
    assert_eq!(
        SparseArrayBlock::size(usize::MAX, 1),
        size_of::<usize>() - 1
    );
    assert_eq!(NATIVE_OVERFLOW_SIZE, size_of::<usize>());
}

#[test]
fn sparse_cell_alias_has_primitive_identity() {
    let mask: SparseArrayBlockCell = u8::MAX;
    let primitive: u8 = mask;
    assert_eq!(primitive, 255);
    assert_eq!(size_of::<SparseArrayBlockCell>(), 1);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn native_size_cases_match_oracle_without_allocation() {
    let expected =
        include_str!("../../../tools/oracle/tkernel_sparse_block_native_surface.expected");
    for line in expected.lines().filter(|line| line.starts_with("size ")) {
        let values: Vec<usize> = line
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(
            SparseArrayBlock::size(values[0], values[1]),
            values[2],
            "{line}"
        );
    }
}

#[test]
fn native_array_adapter_ignores_extreme_dimensions() {
    let mut storage = [0usize; 3];
    let address = storage.as_mut_ptr().cast::<c_void>();
    let expected = unsafe { address.cast::<usize>().add(1).cast::<c_char>() };
    let ordinary = unsafe { SparseArrayBlock::native_to_array(address, 4, 4) };
    let extreme = unsafe { SparseArrayBlock::native_to_array(address, usize::MAX, usize::MAX) };
    let zero = unsafe { SparseArrayBlock::native_to_array(address, 0, 0) };
    assert_eq!((ordinary, extreme, zero), (expected, expected, expected));
    unsafe { ordinary.write(42) };
    assert_eq!(unsafe { (extreme.read(), zero.read()) }, (42, 42));
    assert_eq!(
        unsafe { SparseArrayBlock::to_array(address) },
        ordinary.cast()
    );
}

unsafe fn discard_unconstructed(_: *mut c_void) {
    panic!("no item should reach destruction after rejected layout");
}

fn live_resource_slots(resource: &BaseAllocator) -> i32 {
    resource
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count)
        .sum()
}

struct RejectedErasedStorage {
    base: SparseArrayBase,
    resource: BaseAllocator,
    callbacks: Cell<usize>,
}
impl RejectedErasedStorage {
    fn new(items: usize, stride: usize) -> Self {
        let resource = BaseAllocator::accumulating(128).unwrap();
        let base =
            SparseArrayBase::with_allocator(stride, items, discard_unconstructed, resource.clone());
        Self {
            base,
            resource,
            callbacks: Cell::new(0),
        }
    }
    fn rejected_mutation(&self, index: usize) -> String {
        let error = catch_unwind(AssertUnwindSafe(|| unsafe {
            self.base.set_value_with(
                index,
                std::ptr::null_mut(),
                |_, _| self.callbacks.set(1),
                |_, _| self.callbacks.set(2),
            );
        }))
        .unwrap_err();
        panic_text(error)
    }
    fn assert_unpublished(&self) {
        let shape = (
            self.base.size(),
            self.base.block_slots(),
            self.callbacks.get(),
        );
        assert_eq!(shape, (0, 0, 0));
        assert_eq!(live_resource_slots(&self.resource), 0);
        assert!(self.base.block_address(0).is_null());
    }
}

fn rejected_layout_message(items: usize, stride: usize) -> String {
    format!("SparseArray block {items} items of stride {stride}; expected addressable layout")
}

fn reject_erased_layout(items: usize, stride: usize) {
    let probe = RejectedErasedStorage::new(items, stride);
    let resource = probe.resource.clone();
    assert_eq!(
        probe.rejected_mutation(0),
        rejected_layout_message(items, stride)
    );
    probe.assert_unpublished();
    drop(probe);
    assert_eq!(live_resource_slots(&resource), 0);
}

#[test]
fn erased_overflowing_and_unaddressable_blocks_do_not_allocate_or_invoke_callbacks() {
    for (items, stride) in [
        (1, usize::MAX),
        (usize::MAX, 1),
        (2, usize::MAX),
        (1, isize::MAX as usize),
    ] {
        reject_erased_layout(items, stride);
    }
}

fn panic_text(error: Box<dyn std::any::Any + Send>) -> String {
    match error.downcast::<String>() {
        Ok(message) => *message,
        Err(error) => (*error.downcast::<&str>().unwrap()).to_owned(),
    }
}

struct RejectedIncoming<'a>(&'a Cell<usize>);
impl Drop for RejectedIncoming<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn typed_overflow_preserves_empty_storage_and_drops_incoming_value() {
    let drops = Cell::new(0);
    let mut values = SparseArray::new(usize::MAX);
    let error = catch_unwind(AssertUnwindSafe(|| {
        values.set_value(0, RejectedIncoming(&drops));
    }))
    .unwrap_err();
    let stride = size_of::<RejectedIncoming<'_>>();
    assert_eq!(
        panic_text(error),
        rejected_layout_message(usize::MAX, stride)
    );
    assert_eq!((values.size(), values.capacity(), drops.get()), (0, 0, 1));
    drop(values);
    assert_eq!(drops.get(), 1);
}

#[test]
fn erased_table_overflow_does_not_allocate_or_publish_callbacks() {
    let probe = RejectedErasedStorage::new(1, 4);
    let index = usize::MAX / size_of::<*mut c_void>() + 1;
    assert!(
        probe
            .rejected_mutation(index)
            .contains("expected addressable pointer span")
    );
    probe.assert_unpublished();
}
