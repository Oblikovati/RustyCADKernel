//! Regression cases for BOPDS_FaceInfo maps and native partial Clear behavior.
use rustycad_tkbo::bop_ds::{FaceInfo, PaveBlock, pave_block_handle};
use std::sync::Arc;

fn block(edge: i32) -> rustycad_tkbo::bop_ds::PaveBlockHandle {
    let mut value = PaveBlock::default();
    value.set_original_edge(edge);
    pave_block_handle(value)
}

#[test]
fn empty_state_and_map_insertions_keep_native_unique_keys() {
    let mut info = FaceInfo::default();
    assert_eq!(info.index(), -1);
    assert_eq!(info.pave_blocks_in().extent(), 0);
    assert_eq!(info.vertices_in().extent(), 0);
    assert_eq!(info.pave_blocks_on().extent(), 0);
    assert_eq!(info.vertices_on().extent(), 0);
    assert_eq!(info.pave_blocks_section().extent(), 0);
    assert_eq!(info.vertices_section().extent(), 0);
    let first = block(4);
    let second = block(7);
    assert_eq!(info.change_pave_blocks_in().add(Arc::clone(&first)), 1);
    assert_eq!(info.change_pave_blocks_in().add(Arc::clone(&second)), 2);
    assert_eq!(info.change_pave_blocks_in().add(Arc::clone(&first)), 1);
    assert!(info.pave_blocks_in().contains(&first));
    assert!(Arc::ptr_eq(
        info.pave_blocks_in().find_key(2).unwrap(),
        &second
    ));
    assert_eq!(info.pave_blocks_in().extent(), 2);
    assert!(info.change_vertices_in().add(9));
    assert!(!info.change_vertices_in().add(9));
    assert!(info.vertices_in().contains(9));
    assert_eq!(info.vertices_in().extent(), 1);
}

#[test]
fn clear_empties_in_and_on_but_retains_section_maps_and_index() {
    let mut info = FaceInfo::default();
    let first = block(1);
    let second = block(2);
    let third = block(3);
    info.set_index(12);
    info.change_pave_blocks_in().add(Arc::clone(&first));
    info.change_vertices_in().add(4);
    info.change_pave_blocks_on().add(Arc::clone(&second));
    info.change_vertices_on().add(5);
    info.change_pave_blocks_section().add(Arc::clone(&third));
    info.change_vertices_section().add(6);
    info.clear();
    assert_eq!(info.index(), 12);
    assert_eq!(info.pave_blocks_in().extent(), 0);
    assert_eq!(info.vertices_in().extent(), 0);
    assert_eq!(info.pave_blocks_on().extent(), 0);
    assert_eq!(info.vertices_on().extent(), 0);
    assert_eq!(info.pave_blocks_section().extent(), 1);
    assert!(info.pave_blocks_section().contains(&third));
    assert_eq!(info.vertices_section().extent(), 1);
    assert!(info.vertices_section().contains(6));
}
