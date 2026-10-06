use super::*;
#[repr(C, align(64))]
#[derive(Clone)]
struct Aligned(u8);
#[test]
#[cfg(target_pointer_width = "64")]
fn object_first_node_matches_native_scalar_and_overaligned_layouts() {
    assert_layout::<u8>("u8");
    assert_layout::<i32>("i32");
    assert_layout::<Aligned>("align64");
}
fn assert_layout<T>(label: &str) {
    let expected = native_layout(label);
    assert_eq!(
        [
            std::mem::size_of::<CellObjectNode<T>>(),
            std::mem::align_of::<CellObjectNode<T>>(),
            std::mem::offset_of!(CellObjectNode<T>, object),
            std::mem::offset_of!(CellObjectNode<T>, next)
        ],
        expected[..4]
    );
}
#[test]
#[cfg(target_pointer_width = "64")]
fn actual_cell_index_fields_close_native_cell_and_map_node_size_gaps() {
    // Real ten-integer inline storage closes the former SmallVec byte gap without padding.
    let native = native_layout("i32");
    assert_eq!(std::mem::size_of::<super::super::CellIndex>(), native[10]);
    assert_eq!(std::mem::size_of::<CellRecord<i32>>(), native[4]);
    assert_eq!(std::mem::offset_of!(CellRecord<i32>, objects), native[7]);
    assert_eq!(std::mem::size_of::<CellMapNode<i32>>(), native[8]);
}
#[test]
fn filter_preserves_payload_thread_traits_and_unpin() {
    fn send_sync<T: Send + Sync>() {}
    fn unpin<T: Unpin>() {}
    type Geometry = crate::ncollection::cell_filter::index_storage_tests::IndexGeometry;
    send_sync::<CellFilter<Geometry>>();
    unpin::<CellFilter<Geometry>>();
}

#[test]
fn typed_incremental_nodes_honor_overalignment_and_release_reset_protection() {
    let arena = BaseAllocator::incremental(1024);
    let mut cells = super::super::CellMap::new(arena.clone());
    let index = super::super::CellIndex::from_slice(&[1]);
    cells.add(&index, &Aligned(7));
    let (_, cell) = cells.items().next().unwrap();
    assert_eq!(cell.objects.addr() % 64, 0);
    cells
        .try_retain(&index, |value| {
            assert_eq!(value.0, 7);
            Ok::<_, ()>(false)
        })
        .unwrap();
    assert!(arena.reset(true).is_ok());
}

fn native_layout(label: &str) -> Vec<usize> {
    include_str!("../../../../../../../tools/oracle/tkernel_cell_filter_node_layout.expected")
        .lines()
        .find(|row| row.split_whitespace().next() == Some(label))
        .unwrap()
        .split_whitespace()
        .filter_map(|word| word.parse().ok())
        .collect()
}

#[test]
#[cfg(target_pointer_width = "64")]
fn actual_cell_map_consumes_native_node_allocation_byte_requests() {
    let arena = BaseAllocator::incremental(1024);
    let mut cells = super::super::CellMap::new(arena.clone());
    let first = super::super::CellIndex::from_slice(&[0]);
    cells.add(&first, &7);
    let (map, object) = node_positions(&cells, &first);
    cells.add(&first, &8);
    let (_, second_object) = node_positions(&cells, &first);
    let second = super::super::CellIndex::from_slice(&[1]);
    cells.add(&second, &9);
    let (second_map, third_object) = node_positions(&cells, &second);
    assert_allocation_strides([
        object - map,
        second_object - object,
        second_map - second_object,
        third_object - second_map,
    ]);
}
fn node_positions(
    cells: &super::super::CellMap<i32>,
    index: &super::super::CellIndex,
) -> (usize, usize) {
    let (_, node) = cells.lookup_index(index.as_slice()).unwrap();
    let record = unsafe { super::super::CellMapNode::<i32>::cell_ptr(node.cast()) };
    (node.addr(), unsafe { (*record).objects.addr() })
}
fn assert_allocation_strides(strides: [usize; 4]) {
    let expected = include_str!(
        "../../../../../../../tools/oracle/tkernel_cell_index_native_storage.expected"
    );
    let last_object = std::mem::size_of::<CellObjectNode<i32>>();
    let actual = format!(
        "requests {} {} {} {} {last_object}",
        strides[0], strides[1], strides[2], strides[3]
    );
    // For these8-aligned sizes, observed arena consumption equals each native Allocate request.
    assert_eq!(actual, expected.lines().nth(1).unwrap());
}
