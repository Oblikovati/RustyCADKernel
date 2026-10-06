//! Numeric descriptors never invent provenance; live allocations use their owned block pointer.
use rustycad_tkernel::ncollection::{AccAlignedSize, AccBlock, BaseAllocator};

#[test]
fn numeric_block_descriptor_keeps_defined_address_arithmetic() {
    let mut block = AccBlock::new(0x1001, 16);
    assert_eq!(block.free_size(), 15);
    assert_eq!(
        block.allocate(AccAlignedSize::new(3)).as_ptr().addr(),
        0x100c
    );
    assert_eq!(block.free_size(), 11);
    assert_eq!(block.alloc_count, 1);
    block.free();
    assert!(block.is_empty());
    // Synthetic wrapping is a Rust descriptor extension, not native pointer-arithmetic evidence.
    let wrapped = AccBlock::new(usize::MAX - 3, 12);
    assert_eq!(wrapped.alloc_start.as_ptr().addr(), 8);
    assert_eq!(wrapped.free_size(), 12);
}

#[test]
fn accumulating_owned_payloads_retain_real_block_provenance() {
    let allocator = BaseAllocator::accumulating(64).unwrap();
    let mut first = allocator.allocate_value([1_u32, 2, 3]);
    let mut second = allocator.allocate_value([4_u32, 5, 6]);
    first[1] = 11;
    second[1] = 22;
    assert_eq!(*first, [1, 11, 3]);
    drop(first);
    assert_eq!(*second, [4, 22, 6]);
    drop(second);
    assert_eq!(*allocator.allocate_value([7_u32, 8, 9]), [7, 8, 9]);
}
