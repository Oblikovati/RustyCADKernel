use super::super::{DynamicArray, DynamicArrayIterator, DynamicArrayStorageExtension};
use crate::ncollection::{BaseAllocator, OccAllocator};
type Array = DynamicArray<i32>;
const HIGH_BIT: usize = 1_usize << (usize::BITS - 1);
const UNSIGNED_BOUNDARIES: [usize; 9] = [
    0,
    1,
    2,
    3,
    HIGH_BIT - 1,
    HIGH_BIT,
    HIGH_BIT + 1,
    usize::MAX - 1,
    usize::MAX,
];
const SIGNED_BOUNDARIES: [i32; 6] = [i32::MIN, -1, 0, 1, 3, i32::MAX];
fn row(name: &str, array: &mut Array) -> String {
    let stride = format!(
        "{} {} {}",
        array.increment,
        DynamicArrayStorageExtension::<i32>::log2_pow2(array.increment),
        array.increment.wrapping_sub(1)
    );
    let bounds = format!(
        "{} {} {} {}",
        array.size(),
        array.length(),
        array.lower(),
        array.upper()
    );
    format!("{name} {stride} {bounds} {}", metadata_flags(array))
}
fn metadata_flags(array: &mut Array) -> String {
    let available = array.storage_extension().available_size();
    let at_end = array.legacy_cursor::<false>()
        == DynamicArrayIterator::<i32, false>::new(array, array.size());
    format!(
        "{} {} {} {} {}",
        usize::from(array.is_empty()),
        array.blocks.len(),
        available,
        usize::from(array.allocator().is_some()),
        usize::from(at_end)
    )
}
fn constructor_rows() -> Vec<String> {
    let allocator = BaseAllocator::fresh_common();
    let mut rows = Vec::new();
    for (index, value) in UNSIGNED_BOUNDARIES.into_iter().enumerate() {
        let arrays = unsigned_arrays(value, &allocator);
        rows.extend(family_rows("usize", index, arrays));
        let mut changed = Array::with_increment(3);
        changed.set_increment(value);
        rows.push(row(&format!("set_{index}"), &mut changed));
    }
    for (index, value) in SIGNED_BOUNDARIES.into_iter().enumerate() {
        let arrays = signed_arrays(value, &allocator);
        rows.extend(family_rows("signed", index, arrays));
    }
    rows
}
fn family_rows(prefix: &str, index: usize, arrays: [Array; 3]) -> Vec<String> {
    ["plain", "handle", "typed"]
        .into_iter()
        .zip(arrays)
        .map(|(family, mut array)| row(&format!("{prefix}_{family}_{index}"), &mut array))
        .collect()
}
fn unsigned_arrays(value: usize, allocator: &BaseAllocator) -> [Array; 3] {
    let typed = OccAllocator::new(allocator.clone());
    let plain = Array::with_increment(value);
    let handled = Array::with_allocator(value, Some(allocator.clone()));
    let typed = Array::with_occ_allocator(value, &typed);
    [plain, handled, typed]
}
fn signed_arrays(value: i32, allocator: &BaseAllocator) -> [Array; 3] {
    let typed = OccAllocator::new(allocator.clone());
    let plain = Array::with_increment_i32(value);
    let handled = Array::with_allocator_i32(value, Some(allocator.clone()));
    let typed = Array::with_occ_allocator_i32(value, &typed);
    [plain, handled, typed]
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
