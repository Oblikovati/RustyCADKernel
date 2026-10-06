use super::super::*;
const ITEM_SIZE: usize = size_of::<EngineItem>();
fn numeric_rows(output: &mut String) {
    let high = 1_usize << (usize::BITS - 1);
    let rounds = [0, 1, 3, 8, 9, high, high + 1, usize::MAX];
    let logs = [0, 1, 3, 8, 9, high, usize::MAX];
    let rounded = rounds.map(DynamicArrayStorageExtension::<EngineItem>::round_up_pow2);
    let logarithms = logs.map(DynamicArrayStorageExtension::<EngineItem>::log2_pow2);
    output_values(output, "round", &rounded);
    output_values(output, "log", &logarithms);
}
fn snapshot_row(
    output: &mut String,
    name: &str,
    array: &mut DynamicArray<EngineItem>,
    allocator: &BaseAllocator,
    counts: &mut AllocationObservations,
) {
    output_values(output, name, &snapshot(array, allocator, counts));
}
fn reserve_rows(
    output: &mut String,
    array: &mut DynamicArray<EngineItem>,
    allocator: &BaseAllocator,
    counts: &mut AllocationObservations,
) {
    array
        .storage_extension()
        .ensure_block_count(0, ITEM_SIZE)
        .unwrap();
    snapshot_row(output, "reserve_zero", array, allocator, counts);
    array
        .storage_extension()
        .ensure_storage_for_index(8, ITEM_SIZE)
        .unwrap();
    snapshot_row(output, "reserve_index8", array, allocator, counts);
    address_row(output, array);
    reserve_smaller_row(output, array, allocator, counts);
}
fn reserve_smaller_row(
    output: &mut String,
    array: &mut DynamicArray<EngineItem>,
    allocator: &BaseAllocator,
    counts: &mut AllocationObservations,
) {
    array
        .storage_extension()
        .ensure_block_count(2, ITEM_SIZE)
        .unwrap();
    snapshot_row(output, "reserve_smaller", array, allocator, counts);
}
fn address_row(output: &mut String, array: &mut DynamicArray<EngineItem>) {
    let mut storage = array.storage_extension();
    let first = storage.change_block_address(0).unwrap();
    let constant = storage.block_address(0).unwrap();
    let mutable_typed = storage.change_block_at(0).unwrap().cast::<()>();
    let constant_typed = storage.block_at(0).unwrap().cast::<()>();
    let second = storage.block_address(1).unwrap();
    let flags = [
        std::ptr::eq(first, constant),
        std::ptr::eq(first, mutable_typed),
        std::ptr::eq(first, constant_typed),
        !std::ptr::eq(first, second),
    ];
    output_values(output, "addresses", &flags.map(usize::from));
}
fn expand_rows(
    output: &mut String,
    array: &mut DynamicArray<EngineItem>,
    allocator: &BaseAllocator,
    counts: &mut AllocationObservations,
) {
    let first = array.storage_extension().block_address(0).unwrap();
    let last = array.storage_extension().expand_block(ITEM_SIZE).unwrap();
    snapshot_row(output, "expand", array, allocator, counts);
    let storage = array.storage_extension();
    let flags = [
        std::ptr::eq(last, storage.block_address(3).unwrap()),
        std::ptr::eq(first, storage.block_address(0).unwrap()),
    ];
    output_values(output, "expand_identity", &flags.map(usize::from));
}
fn initialized_rows(
    output: &mut String,
    array: &mut DynamicArray<EngineItem>,
    allocator: &BaseAllocator,
    counts: &mut AllocationObservations,
) {
    array.emplace_append(|| EngineItem::new(11));
    array.storage_extension().change_at(0).unwrap().value = 12;
    let value = array.storage_extension().at(0).unwrap().value as usize;
    output_values(output, "initialized_at", &[value]);
    snapshot_row(output, "one_initialized", array, allocator, counts);
}
fn retained_rows(
    output: &mut String,
    array: &mut DynamicArray<EngineItem>,
    allocator: &BaseAllocator,
    counts: &mut AllocationObservations,
) {
    let first = array.storage_extension().block_address(0).unwrap();
    array.clear(false);
    snapshot_row(output, "clear_retain", array, allocator, counts);
    array.set_increment(2);
    snapshot_row(output, "decrease_stride", array, allocator, counts);
    let same = std::ptr::eq(first, array.storage_extension().block_address(0).unwrap());
    output_values(output, "retained_identity", &[usize::from(same)]);
    array.storage_extension().clear_blocks();
    snapshot_row(output, "clear_empty_blocks", array, allocator, counts);
}
fn clear_live_rows(
    output: &mut String,
    array: &mut DynamicArray<EngineItem>,
    allocator: &BaseAllocator,
    counts: &mut AllocationObservations,
) {
    array.emplace_append(|| EngineItem::new(22));
    snapshot_row(output, "before_clear_live_blocks", array, allocator, counts);
    array.storage_extension().clear_blocks();
    snapshot_row(output, "clear_live_blocks", array, allocator, counts);
}
#[test]
fn native_storage_rows_match_typed_runtime_observations() {
    ELEMENT_LIFETIMES.with(|counts| counts.set((0, 0)));
    let allocator = BaseAllocator::accumulating(4096).unwrap();
    let mut array = DynamicArray::with_allocator(3, Some(allocator.clone()));
    let mut counts = AllocationObservations::default();
    let mut actual = String::new();
    numeric_rows(&mut actual);
    reserve_rows(&mut actual, &mut array, &allocator, &mut counts);
    expand_rows(&mut actual, &mut array, &allocator, &mut counts);
    initialized_rows(&mut actual, &mut array, &allocator, &mut counts);
    retained_rows(&mut actual, &mut array, &allocator, &mut counts);
    clear_live_rows(&mut actual, &mut array, &allocator, &mut counts);
    assert_eq!(
        actual,
        include_str!("../../../../tools/oracle/tkernel_dynamic_protected_storage.expected")
    );
}
