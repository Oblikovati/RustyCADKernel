//! Actual flat node offsets match pinned OCCT for equivalent scalar and aligned payloads.
use rustycad_tkernel::ncollection::{BaseAllocator, IndexedDataMapNode, IndexedMapNode};
#[path = "indexed_node_layout_contracts/mod.rs"]
mod fixture;
use fixture::{Wide, native_layout_columns};

fn map_layout<K: Default>(label: &str) {
    let arena = BaseAllocator::incremental(4096);
    let _prefix = arena.allocate(16).unwrap();
    let mut node = IndexedMapNode::allocate_with(&arena, 1, std::ptr::null_mut(), K::default);
    let adjacent = IndexedMapNode::allocate_with(&arena, 2, std::ptr::null_mut(), K::default);
    let start = std::ptr::from_ref(&*node).addr();
    let key = std::ptr::from_mut(node.key1()).addr() - start;
    let index = std::ptr::from_mut(node.index()).addr() - start;
    let size = size_of_val(&*node);
    assert_eq!(std::ptr::from_ref(&*adjacent).addr() - start, size);
    assert_eq!(start % align_of_val(&*node), 0);
    native_layout_columns(label, &[size, align_of_val(&*node), key, index, size]);
}

fn data_layout<K: Default, V: Default>(label: &str) {
    let arena = BaseAllocator::incremental(4096);
    let _prefix = arena.allocate(16).unwrap();
    let make = || {
        IndexedDataMapNode::allocate_with(&arena, 1, std::ptr::null_mut(), K::default, V::default)
    };
    let (mut node, adjacent) = (make(), make());
    let start = std::ptr::from_ref(&*node).addr();
    let key = std::ptr::from_mut(node.key1()).addr() - start;
    let value = std::ptr::from_ref(node.value()).addr() - start;
    let index = std::ptr::from_mut(node.index()).addr() - start;
    let size = size_of_val(&*node);
    assert_eq!(std::ptr::from_ref(&*adjacent).addr() - start, size);
    assert_eq!(start % align_of_val(&*node), 0);
    native_layout_columns(
        label,
        &[size, align_of_val(&*node), key, value, index, size],
    );
}

#[test]
fn indexed_key_layout_and_allocation_spacing_match_native() {
    map_layout::<u8>("map_u8");
    map_layout::<i32>("map_i32");
    map_layout::<Wide>("map_wide");
}

#[test]
fn indexed_scalar_data_layout_and_allocation_spacing_match_native() {
    data_layout::<u8, u8>("data_u8_u8");
    data_layout::<i32, u8>("data_i32_u8");
    data_layout::<u8, i32>("data_u8_i32");
    data_layout::<i32, i32>("data_i32_i32");
}

#[test]
fn indexed_aligned_data_layout_and_allocation_spacing_match_native() {
    data_layout::<Wide, u8>("data_wide_u8");
    data_layout::<u8, Wide>("data_u8_wide");
    data_layout::<Wide, Wide>("data_wide_wide");
}
