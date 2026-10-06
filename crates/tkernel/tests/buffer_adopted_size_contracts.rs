//! Native adopted buffers expose a logical prefix while retaining the whole allocation.
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};

fn adopted_row(size: usize) -> String {
    let allocator = BaseAllocator::common();
    let mut allocation = allocator.allocate(16).unwrap();
    allocation.as_mut_slice().fill(7);
    let address = allocation.as_ptr();
    let mut buffer = Buffer::with_data_size(Some(allocator), allocation, size).unwrap();
    assert_eq!(buffer.as_slice(), vec![7; size]);
    buffer.as_mut_slice().fill(3);
    adopted_state_row(size, address, &buffer)
}

fn adopted_state_row(size: usize, address: *const u8, buffer: &Buffer) -> String {
    format!(
        "{size} {} {} {} {} {}\n",
        buffer.size(),
        buffer.is_empty() as u8,
        (buffer.data_ptr() == address) as u8,
        buffer.data_end_ptr().addr() - buffer.data_ptr().addr(),
        buffer
            .as_slice()
            .iter()
            .map(|byte| usize::from(*byte))
            .sum::<usize>()
    )
}

#[test]
fn adopted_prefix_size_endpoints_and_writes_match_native_rows() {
    let rows: String = [0, 1, 4, 16].into_iter().map(adopted_row).collect();
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkernel_buffer_adopted_size.expected")
    );
}

#[test]
fn adopted_size_follows_free_allocate_and_allocator_replacement() {
    let allocator = BaseAllocator::common();
    let allocation = allocator.allocate(16).unwrap();
    let mut buffer = Buffer::with_data_size(Some(allocator), allocation, 4).unwrap();
    buffer.free().unwrap();
    assert_eq!(buffer.size(), 0);
    assert!(buffer.data_ptr().is_null());
    assert!(buffer.allocate(9).unwrap());
    assert_eq!(buffer.size(), 9);
    buffer.set_allocator(None).unwrap();
    assert_eq!(buffer.size(), 0);
    assert!(!buffer.allocate(0).unwrap());
}

#[test]
fn adopted_prefix_rejects_size_beyond_initialized_allocation() {
    let allocation = BaseAllocator::common().allocate(4).unwrap();
    let failure = Buffer::with_data_size(None, allocation, 5).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(
        failure
            .to_string()
            .contains("buffer size 5 exceeds allocation extent 4; expected 0..=4")
    );
}

#[test]
fn zero_extent_and_null_selected_allocator_keep_owned_identity() {
    let allocator = BaseAllocator::common();
    let allocation = allocator.allocate(0).unwrap();
    let address = allocation.as_ptr();
    let mut buffer = Buffer::with_data_size(None, allocation, 0).unwrap();
    assert_eq!(buffer.data_ptr(), address);
    assert_eq!(buffer.data_end_ptr(), address);
    assert!(!buffer.is_empty());
    assert!(buffer.as_mut_slice().is_empty());
    buffer.free().unwrap();
    assert!(buffer.is_empty());
}

#[test]
fn dump_reports_logical_adopted_size() {
    let allocator = BaseAllocator::common();
    let allocation = allocator.allocate(16).unwrap();
    let buffer = Buffer::with_data_size(Some(allocator), allocation, 4).unwrap();
    let mut output = rustycad_tkernel::standard::dump::DumpStream::default();
    buffer.dump_json(&mut output, -1).unwrap();
    assert!(output.text().contains("\"Size\": 4"));
}
