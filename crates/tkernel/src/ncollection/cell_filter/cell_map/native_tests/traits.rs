//! Auto traits and over-aligned targets. The native node layout and arena byte rows describe
//! allocator-owned list nodes (plan D1 classes B5 and B6) and are not compared.
use super::*;
#[repr(C, align(64))]
#[derive(Clone)]
struct Aligned(u8);
#[test]
fn filter_preserves_payload_thread_traits_and_unpin() {
    fn send_sync<T: Send + Sync>() {}
    fn unpin<T: Unpin>() {}
    type Geometry = crate::ncollection::cell_filter::index_storage_tests::IndexGeometry;
    send_sync::<CellFilter<Geometry>>();
    unpin::<CellFilter<Geometry>>();
}
#[test]
fn overaligned_targets_are_stored_aligned_and_purged() {
    let arena = BaseAllocator::incremental(1024);
    let mut cells = super::super::CellMap::new(arena.clone());
    let index = super::super::CellIndex::from_slice(&[1]);
    cells.add(&index, &Aligned(7));
    let (_, cell) = cells.items().next().unwrap();
    let target = cell.targets().next().unwrap();
    assert_eq!(std::ptr::from_ref(target).addr() % 64, 0);
    cells
        .try_retain(&index, |value| {
            assert_eq!(value.0, 7);
            Ok::<_, ()>(false)
        })
        .unwrap();
    assert!(cells.is_empty());
    assert!(arena.reset(true).is_ok());
}
