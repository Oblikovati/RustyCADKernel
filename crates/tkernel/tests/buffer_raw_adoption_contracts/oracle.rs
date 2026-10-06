use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};
pub fn allocator(policy: usize) -> BaseAllocator {
    match policy {
        0 => BaseAllocator::common(),
        1 => BaseAllocator::global_heap(),
        2 => BaseAllocator::incremental(4096),
        3 => BaseAllocator::accumulating(256).unwrap(),
        4 => BaseAllocator::aligned(32),
        _ => panic!("allocator policy {policy}; expected 0..=4"),
    }
}
pub fn initialized(owner: &BaseAllocator, optimal: bool, size: usize) -> *mut u8 {
    let pointer = if optimal {
        owner.allocate_optimal_raw(size)
    } else {
        owner.allocate_raw(size)
    }
    .unwrap();
    // SAFETY: the raw allocation contains size exclusively owned bytes, initialized before adoption.
    unsafe { pointer.write_bytes(7, size) };
    pointer.as_ptr()
}
pub fn adopted(owner: &BaseAllocator, size: usize) -> Buffer {
    let pointer = initialized(owner, false, 16);
    // SAFETY: initialized raw storage belongs to owner; all ownership transfers into the buffer.
    unsafe { Buffer::from_raw_data(owner.clone(), size, pointer) }.unwrap()
}
fn prefix(policy: usize, optimal: bool, size: usize) -> String {
    let owner = allocator(policy);
    let pointer = initialized(&owner, optimal, 16);
    // SAFETY: the requested prefix fits the initialized raw allocation and its exact allocator.
    let buffer = unsafe { Buffer::from_raw_data(owner, size, pointer) }.unwrap();
    format!(
        "prefix {policy} {} {size} {} {} {} {} {}\n",
        u8::from(optimal),
        buffer.size(),
        u8::from(buffer.is_empty()),
        u8::from(buffer.data_ptr() == pointer),
        buffer.data_end_ptr().addr() - buffer.data_ptr().addr(),
        checksum(&buffer)
    )
}
fn checksum(buffer: &Buffer) -> usize {
    buffer
        .as_slice()
        .iter()
        .map(|byte| usize::from(*byte))
        .sum()
}
fn null_pointer(policy: usize, size: usize) -> String {
    // SAFETY: null requests allocation and transfers no external memory.
    match unsafe { Buffer::from_raw_data(allocator(policy), size, std::ptr::null_mut()) } {
        Ok(buffer) => format!(
            "null {policy} {size} {} {}\n",
            buffer.size(),
            u8::from(buffer.is_empty())
        ),
        Err(error) => format!("null {policy} {size} {}\n", error.exception_type()),
    }
}
fn replace(policy: usize) -> String {
    let owner = allocator(policy);
    let replacement = BaseAllocator::common();
    let mut buffer = adopted(&owner, 4);
    buffer.set_allocator(Some(replacement.clone())).unwrap();
    format!(
        "replace {policy} {} {} {}\n",
        buffer.size(),
        u8::from(buffer.is_empty()),
        u8::from(buffer.allocator() == Some(&replacement))
    )
}
fn allocate_again(policy: usize) -> String {
    let mut buffer = adopted(&allocator(policy), 4);
    let success = buffer.allocate(6).unwrap();
    buffer.as_mut_slice().fill(3);
    format!(
        "allocate {policy} {} {} {}\n",
        u8::from(success),
        buffer.size(),
        checksum(&buffer)
    )
}
fn free_buffer(policy: usize) -> String {
    let owner = allocator(policy);
    let mut buffer = adopted(&owner, 4);
    buffer.free().unwrap();
    format!(
        "free {policy} {} {} {}\n",
        buffer.size(),
        u8::from(buffer.is_empty()),
        u8::from(buffer.allocator() == Some(&owner))
    )
}
fn zero_allocation(policy: usize) -> String {
    let owner = allocator(policy);
    let pointer = owner.allocate_raw(0).unwrap().as_ptr();
    // SAFETY: the live non-null raw allocation has an empty initialized extent.
    let buffer = unsafe { Buffer::from_raw_data(owner, 0, pointer) }.unwrap();
    format!(
        "zero {policy} {} {} {}\n",
        buffer.size(),
        u8::from(buffer.is_empty()),
        u8::from(buffer.data_ptr() == pointer)
    )
}
fn policy_rows(policy: usize) -> String {
    let mut rows = String::new();
    for optimal in [false, true] {
        for size in [0, 1, 4, 16] {
            rows.push_str(&prefix(policy, optimal, size));
        }
    }
    for size in [0, 4] {
        rows.push_str(&null_pointer(policy, size));
    }
    rows.push_str(&replace(policy));
    rows.push_str(&allocate_again(policy));
    if policy != 3 {
        rows.push_str(&free_buffer(policy));
        rows.push_str(&zero_allocation(policy));
    }
    rows
}
fn accumulating_count() -> String {
    let owner = allocator(3);
    let pointer = initialized(&owner, false, 16);
    let count = || {
        owner
            .accumulating_chain()
            .iter()
            .map(|b| b.alloc_count)
            .sum::<i32>()
    };
    let before = count();
    // SAFETY: initialized raw prefix and allocator match; buffer becomes the sole owner.
    let buffer = unsafe { Buffer::from_raw_data(owner.clone(), 4, pointer) }.unwrap();
    let during = count();
    drop(buffer);
    format!("acc_count {before} {during} {}\n", count())
}
fn incremental_cursor() -> String {
    let owner = allocator(2);
    let pointer = initialized(&owner, false, 16);
    // SAFETY: the live initialized raw allocation is transferred exactly once.
    let _buffer = unsafe { Buffer::from_raw_data(owner.clone(), 4, pointer) }.unwrap();
    let next = owner.allocate_raw(1).unwrap();
    let offset = next.as_ptr().addr() - pointer.addr();
    // SAFETY: next is a distinct raw allocation not owned by the buffer.
    unsafe { owner.free_raw(next) }.unwrap();
    format!("inc_cursor {offset}\n")
}
fn invalid_alignment() -> String {
    // SAFETY: null means no external allocation; the invalid alignment returns allocation failure.
    let buffer =
        unsafe { Buffer::from_raw_data(BaseAllocator::aligned(3), 4, std::ptr::null_mut()) }
            .unwrap();
    format!(
        "invalid_alignment {} {}\n",
        buffer.size(),
        u8::from(buffer.is_empty())
    )
}
#[test]
fn raw_constructor_and_followup_methods_match_all_native_policy_rows() {
    let mut rows: String = (0..5).map(policy_rows).collect();
    rows.push_str(&accumulating_count());
    rows.push_str(&incremental_cursor());
    rows.push_str(&invalid_alignment());
    assert_eq!(
        rows,
        include_str!("../../../../tools/oracle/tkernel_buffer_raw_adoption.expected")
    );
}
