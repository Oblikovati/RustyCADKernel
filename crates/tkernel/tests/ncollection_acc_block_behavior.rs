//! Defined value behavior of the protected AccAllocator block descriptor.

use rustycad_tkernel::ncollection::{AccAlignedSize, AccBlock};

#[test]
fn protected_block_trace_matches_captured_native_output() {
    assert_eq!(
        native_block_trace(),
        include_str!("../../../tools/oracle/ncollection_acc_block.expected")
    );
}

#[test]
fn block_initial_state_and_previous_identity_match_native() {
    let block = AccBlock::with_previous(0x1004, 24, Some(0x2000));

    assert_eq!(block.free_size(), 24);
    assert_eq!(block.alloc_count, 0);
    assert!(block.is_empty());
    assert_eq!(block.prev_block, Some(0x2000));
}

#[test]
fn block_set_free_size_and_allocation_match_native_cursor() {
    let mut block = AccBlock::new(0x1004, 24);
    block.set_free_size(13);
    assert_eq!(block.free_size(), 12);
    assert_eq!(block.alloc_start.as_ptr() as usize, block.address + 12);

    let allocated = block.allocate(AccAlignedSize::new(5));
    assert_eq!(allocated.as_ptr() as usize - block.address, 4);
    assert_eq!(block.free_size(), 4);
    assert_eq!(block.alloc_count, 1);
    assert!(!block.is_empty());
}

#[test]
fn block_free_decrements_count_and_restores_empty_state() {
    let mut block = AccBlock::new(0x1004, 24);
    block.allocate(AccAlignedSize::new(5));
    block.free();

    assert_eq!(block.alloc_count, 0);
    assert!(block.is_empty());
}

fn native_block_trace() -> String {
    let previous = AccBlock::new(0x2000, 4);
    let mut block = AccBlock::with_previous(0x1004, 24, Some(previous.address));
    let mut trace = initial_block_row(&block, previous.address);
    block.set_free_size(13);
    trace.push_str(&cursor_block_row(&block));
    trace.push_str(&allocated_block_row(&mut block));
    trace.push_str(&emptied_block_row(&mut block));
    trace
}

fn initial_block_row(block: &AccBlock, previous_address: usize) -> String {
    format!(
        "{},{},{},{}\n",
        block.free_size(),
        block.alloc_count,
        block.is_empty() as u8,
        (block.prev_block == Some(previous_address)) as u8
    )
}

fn cursor_block_row(block: &AccBlock) -> String {
    format!("{}\n", block.free_size())
}

fn allocated_block_row(block: &mut AccBlock) -> String {
    let offset = block.allocate(AccAlignedSize::new(5)).as_ptr() as usize - block.address;
    format!(
        "{offset},{},{},{}\n",
        block.free_size(),
        block.alloc_count,
        block.is_empty() as u8
    )
}

fn emptied_block_row(block: &mut AccBlock) -> String {
    block.free();
    format!("{},{}\n", block.alloc_count, block.is_empty() as u8)
}
