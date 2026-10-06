use super::super::{DynamicArray, DynamicArrayStorageExtension};
use crate::ncollection::{BaseAllocator, OccAllocator};
fn row(name: &str, array: &mut DynamicArray<i32>) -> String {
    let available = array.storage_extension().available_size();
    format!(
        "{name} {} {} {} {} {} {} {} {} {} {} {} {}",
        array.increment,
        DynamicArrayStorageExtension::<i32>::log2_pow2(array.increment),
        array.increment.wrapping_sub(1),
        array.size(),
        array.length(),
        array.lower(),
        array.upper(),
        usize::from(array.is_empty()),
        array.blocks.len(),
        available,
        usize::from(array.allocator().is_some()),
        usize::from(
            array.legacy_cursor::<false>()
                == super::super::DynamicArrayIterator::<i32, false>::new(array, array.size())
        )
    )
}
fn constructor_rows() -> Vec<String> {
    let high = 1_usize << (usize::BITS - 1);
    let values = [
        0,
        1,
        2,
        3,
        high - 1,
        high,
        high + 1,
        usize::MAX - 1,
        usize::MAX,
    ];
    let allocator = BaseAllocator::fresh_common();
    let typed = OccAllocator::new(allocator.clone());
    let mut rows = Vec::new();
    for (index, value) in values.into_iter().enumerate() {
        rows.push(row(
            &format!("usize_plain_{index}"),
            &mut DynamicArray::with_increment(value),
        ));
        rows.push(row(
            &format!("usize_handle_{index}"),
            &mut DynamicArray::with_allocator(value, Some(allocator.clone())),
        ));
        rows.push(row(
            &format!("usize_typed_{index}"),
            &mut DynamicArray::with_occ_allocator(value, &typed),
        ));
        let mut changed = DynamicArray::with_increment(3);
        changed.set_increment(value);
        rows.push(row(&format!("set_{index}"), &mut changed));
    }
    rows.extend(signed_rows(&allocator, &typed));
    rows
}
fn signed_rows(allocator: &BaseAllocator, typed: &OccAllocator<i32>) -> Vec<String> {
    let mut rows = Vec::new();
    for (index, value) in [i32::MIN, -1, 0, 1, 3, i32::MAX].into_iter().enumerate() {
        rows.push(row(
            &format!("signed_plain_{index}"),
            &mut DynamicArray::with_increment_i32(value),
        ));
        rows.push(row(
            &format!("signed_handle_{index}"),
            &mut DynamicArray::with_allocator_i32(value, Some(allocator.clone())),
        ));
        rows.push(row(
            &format!("signed_typed_{index}"),
            &mut DynamicArray::with_occ_allocator_i32(value, typed),
        ));
    }
    rows
}
fn transfer_rows() -> Vec<String> {
    let mut source =
        DynamicArray::<i32>::with_allocator(usize::MAX, Some(BaseAllocator::fresh_common()));
    let mut copied = source.clone();
    let mut rows = vec![row("copy_zero", &mut copied)];
    let mut moved = DynamicArray::from_moved(&mut source);
    rows.push(row("move_zero", &mut moved));
    rows.push(row("moved_source_zero", &mut source));
    let mut assigned = DynamicArray::with_increment(2);
    assigned.assign(&copied, false);
    rows.push(row("assign_zero", &mut assigned));
    let mut receiver = DynamicArray::with_increment(2);
    receiver.move_from(&mut assigned);
    rows.push(row("move_assign_zero", &mut receiver));
    rows.push(row("move_assign_source_zero", &mut assigned));
    rows
}
fn retained_rows() -> Vec<String> {
    let mut array = DynamicArray::with_increment(2);
    for value in [1, 2, 3] {
        array.append(value);
    }
    array.clear(false);
    array.set_increment(usize::MAX);
    let mut rows = vec![row("retained_zero", &mut array)];
    array.set_increment(2);
    array.append(77);
    rows.push(row("retained_reset", &mut array));
    array.set_increment(usize::MAX);
    rows.push(row("live_unchanged", &mut array));
    let mut empty = DynamicArray::with_increment(usize::MAX);
    empty.set_increment(0);
    empty.append(9);
    rows.push(row("zero_reset", &mut empty));
    rows
}
#[test]
fn native_unsigned_and_signed_constructor_metadata_rows() {
    let mut rows = constructor_rows();
    rows.extend(transfer_rows());
    rows.extend(retained_rows());
    let expected =
        include_str!("../../../../../../tools/oracle/tkernel_dynamic_increment_overflow.expected");
    assert_eq!(rows.join("\n") + "\n", expected);
}
