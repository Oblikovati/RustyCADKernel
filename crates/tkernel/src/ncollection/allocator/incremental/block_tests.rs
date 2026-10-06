//! Exact LP64 native layout and constructor/reset rows on the actual allocated header.
use super::{IncAllocatorBlock, IncBlockLink};
use std::mem::{align_of, offset_of, size_of};

fn assert_block_row(actual: &str) {
    let fixture = include_str!(
        "../../../../../../tools/oracle/tkernel_inc_allocator_block_operations.expected"
    );
    assert!(
        fixture.lines().any(|expected| expected == actual),
        "missing native row: {actual}"
    );
}

#[test]
fn actual_header_matches_measured_native_layout() {
    let row = format!(
        "layout {} {} {} {} {} {}",
        size_of::<IncAllocatorBlock>(),
        align_of::<IncAllocatorBlock>(),
        offset_of!(IncAllocatorBlock, current),
        offset_of!(IncAllocatorBlock, available),
        offset_of!(IncAllocatorBlock, next),
        offset_of!(IncAllocatorBlock, ordered_next)
    );
    assert_block_row(&row);
}

#[test]
fn native_constructor_only_computes_pointer_and_initial_fields() {
    let mut buffer = [0_u8; 160];
    for size in [0, 1, 128] {
        let header = IncAllocatorBlock::new(buffer.as_mut_ptr(), size);
        let fields = header.observation();
        let expected_start = buffer.as_ptr().addr() + size_of::<IncAllocatorBlock>();
        assert_block_row(&format!(
            "construct-{size} {} {} {} {}",
            u8::from(fields.current == expected_start),
            fields.available,
            u8::from(fields.next == 0),
            u8::from(fields.ordered_next == 0)
        ));
    }
}

#[test]
fn allocated_header_reset_preserves_both_intrusive_links() {
    let block = IncBlockLink::allocate(128).unwrap();
    let first = block.bump(31);
    block.set_next(Some(block));
    block.set_ordered_next(Some(block));
    block.reset();
    let fields = block.observation();
    assert_block_row(&format!(
        "reset-placement {} {} {} {}",
        u8::from(fields.current == first.as_ptr().addr()),
        fields.available,
        u8::from(fields.next == fields.address),
        u8::from(fields.ordered_next == fields.address)
    ));
    // SAFETY: no payload leases exist and no copied link is used after this release.
    unsafe { block.release() };
}

#[test]
fn stable_header_allocation_contains_exact_native_header_and_payload_bytes() {
    for size in [0, 1, 128, 2048] {
        let block = IncBlockLink::allocate(size).unwrap();
        // SAFETY: the block handle retains the exact raw_heap allocation base, never a payload reborrow.
        let allocated = unsafe { super::super::super::raw_heap::heap_payload_size(block.0.cast()) };
        assert_eq!(allocated, size_of::<IncAllocatorBlock>() + size);
        assert_eq!(block.current().as_ptr().addr() % 16, 0);
        // SAFETY: this allocation was never handed to any payload owner or copied outside this test.
        unsafe { block.release() };
    }
}

#[test]
fn standalone_header_accepts_opaque_pointer_values_without_dereferencing() {
    for address in [0, 1, usize::MAX] {
        let pointer = std::ptr::without_provenance_mut(address);
        let header = IncAllocatorBlock::new(pointer, 17);
        assert_eq!(
            header.observation().current,
            address.wrapping_add(size_of::<IncAllocatorBlock>())
        );
        assert_eq!(header.observation().available, 17);
    }
}
