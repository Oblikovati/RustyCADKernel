use super::super::*;
fn trivial_source(used: i32, blocks: usize, allocator: &BaseAllocator) -> DynamicArray<i32> {
    let mut source = DynamicArray::with_allocator(2, Some(allocator.clone()));
    for value in 1..=used {
        source.append(value);
    }
    source
        .storage_extension()
        .ensure_block_count(blocks, size_of::<i32>())
        .unwrap();
    source
}
fn trivial_row(
    used: i32,
    blocks: usize,
    mode: usize,
    target: &mut DynamicArray<i32>,
    allocator: &BaseAllocator,
) -> String {
    format!(
        "{used},{blocks},{mode} size={} blocks={} source={} values={}\n",
        target.size(),
        target.storage_extension().block_count(),
        u8::from(target.allocator() == Some(allocator)),
        target
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn trivial_rows(used: i32, blocks: usize) -> String {
    let allocator = BaseAllocator::fresh_common();
    let source = trivial_source(used, blocks, &allocator);
    let mut copied = DynamicArray::from_copied_trivial(&source).unwrap();
    let mut output = trivial_row(used, blocks, 0, &mut copied, &allocator);
    for mode in 1..=2 {
        let mut target = DynamicArray::with_increment(4);
        target.append(99);
        target.assign_trivial(&source, mode == 1).unwrap();
        output.push_str(&trivial_row(used, blocks, mode, &mut target, &allocator));
    }
    output
}
#[test]
fn public_trivial_native_rows_copy_empty_retained_and_partial_last_blocks() {
    let mut actual = String::new();
    for (used, blocks) in [(0, 0), (0, 3), (3, 3)] {
        actual.push_str(&trivial_rows(used, blocks));
    }
    assert_eq!(
        actual,
        include_str!("../../../../tools/oracle/tkernel_dynamic_public_copy_trivial.expected")
    );
}
