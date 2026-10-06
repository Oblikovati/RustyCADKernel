//! Protected raw arithmetic follows native unsigned wrapping without relaxing typed layout checks.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray};
fn allocator_totals(allocator: &BaseAllocator) -> (usize, usize) {
    let chain = allocator.accumulating_chain();
    let live = chain.iter().map(|block| block.alloc_count as usize).sum();
    let free = chain.iter().map(|block| block.free_size).sum();
    (live, free)
}
fn byte_checksum(pointer: *mut u8, bytes: usize) -> usize {
    let mut checksum = 0;
    for offset in 0..bytes {
        // The detached request owns exactly this writable byte range until deallocation below.
        unsafe {
            pointer.add(offset).write(0x5a);
            checksum += usize::from(pointer.add(offset).read());
        }
    }
    checksum
}
fn allocation_row(stride: usize, item_size: usize) -> String {
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut array = DynamicArray::<i32>::with_allocator(stride, Some(allocator.clone()));
    let before = allocator_totals(&allocator);
    let pointer = array.storage_extension().allocate_block(item_size).unwrap();
    let allocated = allocator_totals(&allocator);
    let bytes = before.1 - allocated.1;
    let checksum = byte_checksum(pointer, bytes);
    // The pointer belongs to this raw allocator and contains bytes only, with no typed owner.
    unsafe {
        array.storage_extension().deallocate_block(pointer).unwrap();
    }
    assert_eq!(allocated.0 - before.0, 1);
    assert_eq!(allocator_totals(&allocator).0, before.0);
    assert_eq!(
        (array.size(), array.storage_extension().block_count()),
        (0, 0)
    );
    format!("raw {stride} {item_size} {bytes} 1 1 0 0 {checksum}\n")
}
fn zero_allocation_row(stride: usize, item_size: usize) -> String {
    let mut array =
        DynamicArray::<i32>::with_allocator(stride, Some(BaseAllocator::fresh_common()));
    let pointer = array.storage_extension().allocate_block(item_size).unwrap();
    // Native common allocation permits freeing a zero-byte allocation; AccAllocator differs here.
    unsafe {
        array.storage_extension().deallocate_block(pointer).unwrap();
    }
    assert_eq!(
        (array.size(), array.storage_extension().block_count()),
        (0, 0)
    );
    format!("raw {stride} {item_size} 0 1 1 0 0 0\n")
}
fn noop_row(used: i32, reserved: usize) -> String {
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut array = noop_array(used, reserved, &allocator);
    let first = array.storage_extension().block_address(0).ok();
    let before = allocator_totals(&allocator);
    array
        .storage_extension()
        .ensure_storage_for_index(usize::MAX, size_of::<i32>())
        .unwrap();
    format!(
        "noop {used} {reserved} {} {} {} {} {}\n",
        array.size(),
        array.storage_extension().block_count(),
        allocator_totals(&allocator).0 - before.0,
        u8::from(first == array.storage_extension().block_address(0).ok()),
        array.iter().sum::<i32>()
    )
}
fn noop_array(used: i32, reserved: usize, allocator: &BaseAllocator) -> DynamicArray<i32> {
    let mut array = DynamicArray::with_allocator(1, Some(allocator.clone()));
    for value in 1..=used {
        array.append(value);
    }
    array
        .storage_extension()
        .ensure_block_count(reserved, size_of::<i32>())
        .unwrap();
    array
}
const RAW_CASES: &[(usize, usize)] = &[
    (1, 24),
    (2, usize::MAX / 2 + 5),
    (4, usize::MAX / 4 + 5),
    (8, usize::MAX / 8 + 5),
    (256, usize::MAX / 256 + 2),
];
#[test]
fn native_raw_wrapping_and_maximum_index_noop_rows_match() {
    let mut actual = String::new();
    for &(stride, item_size) in RAW_CASES {
        actual.push_str(&allocation_row(stride, item_size));
    }
    for (stride, item_size) in [(2, usize::MAX / 2 + 1), (usize::MAX, 7)] {
        actual.push_str(&zero_allocation_row(stride, item_size));
    }
    for (used, reserved) in [(0, 0), (0, 3), (1, 3), (3, 3)] {
        actual.push_str(&noop_row(used, reserved));
    }
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkernel_dynamic_storage_wrapping.expected")
    );
}
#[test]
fn overflowing_item_size_does_not_escape_typed_storage_checks() {
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut array = DynamicArray::<i32>::with_allocator(2, Some(allocator.clone()));
    let before = allocator_totals(&allocator);
    let item_size = usize::MAX / 2 + 5;
    let mut storage = array.storage_extension();
    assert!(storage.ensure_storage_for_index(0, item_size).is_err());
    assert!(storage.ensure_block_count(1, item_size).is_err());
    assert!(storage.expand_block(item_size).is_err());
    assert_eq!(storage.block_count(), 0);
    assert_eq!(allocator_totals(&allocator), before);
}
