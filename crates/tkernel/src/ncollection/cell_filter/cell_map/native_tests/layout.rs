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
fn existing_local_array_adapter_cell_layout_gap_is_explicit() {
    // Native CellIndex56/Cell64/MapNode72; SmallVec index48 produces Cell56/MapNode64.
    let native = native_layout("i32");
    assert_eq!(
        std::mem::size_of::<super::super::CellIndex>() + 8,
        native[10]
    );
    assert_eq!(std::mem::size_of::<CellRecord<i32>>() + 8, native[4]);
    assert_eq!(
        std::mem::offset_of!(CellRecord<i32>, objects) + 8,
        native[7]
    );
    assert_eq!(std::mem::size_of::<CellMapNode<i32>>() + 8, native[8]);
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
