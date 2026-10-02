//! Regression cases for BOPDS_CommonBlock ordering, shared identity, faces and dump text.
use rustycad_tkbo::bop_ds::{CommonBlock, PaveBlock, PaveBlockHandle, pave_block_handle};
use std::sync::Arc;

fn block(original_edge: i32, edge: i32) -> PaveBlockHandle {
    let mut block = PaveBlock::default();
    block.set_original_edge(original_edge);
    block.set_edge(edge);
    pave_block_handle(block)
}

fn original_edges(group: &CommonBlock) -> Vec<i32> {
    group
        .pave_blocks()
        .iter()
        .map(|block| block.lock().unwrap().original_edge())
        .collect()
}

#[test]
fn add_and_set_blocks_preserve_native_first_minimum_ordering() {
    let first = block(9, 20);
    let second = block(3, 30);
    let third = block(5, 50);
    let mut group = CommonBlock::new();
    group.add_pave_block(Arc::clone(&first));
    group.add_pave_block(Arc::clone(&second));
    group.add_pave_block(Arc::clone(&third));
    assert_eq!(original_edges(&group), [3, 9, 5]);
    assert!(Arc::ptr_eq(group.pave_block1().unwrap(), &second));
    group.set_pave_blocks(&[third, first, second]);
    assert_eq!(original_edges(&group), [3, 5, 9]);
}

#[test]
fn edge_queries_mutations_and_real_block_selection_keep_handle_identity() {
    let first = block(9, 20);
    let second = block(3, 30);
    let third = block(5, 50);
    let mut group = CommonBlock::new();
    group.set_pave_blocks(&[first.clone(), second.clone(), third.clone()]);
    assert!(group.is_pave_block_on_edge(5));
    assert!(!group.is_pave_block_on_edge(8));
    assert!(group.contains_pave_block(&third));
    assert!(!group.contains_pave_block(&block(5, 50)));
    assert!(Arc::ptr_eq(&group.pave_block_on_edge(5).unwrap(), &third));
    assert!(group.pave_block_on_edge(88).is_none());
    group.set_edge(42);
    assert_eq!(group.edge().unwrap(), 42);
    assert!(
        group
            .pave_blocks()
            .iter()
            .all(|block| block.lock().unwrap().edge() == 42)
    );
    group.set_real_pave_block(&third);
    assert!(Arc::ptr_eq(group.pave_block1().unwrap(), &third));
    group.set_real_pave_block(&block(100, 100));
    assert!(Arc::ptr_eq(group.pave_block1().unwrap(), &third));
}

#[test]
fn face_mutations_retain_duplicates_and_append_moves_the_input() {
    let mut group = CommonBlock::new();
    assert!(!group.contains_face(4));
    group.add_face(4);
    group.set_faces(&[6, 6]);
    let mut appended = vec![8, 9];
    group.append_faces(&mut appended);
    assert!(appended.is_empty());
    assert_eq!(group.faces(), [6, 6, 8, 9]);
    assert!(group.contains_face(8));
    assert!(group.is_pave_block_on_face(8));
    assert!(!group.is_pave_block_on_face(7));
}

#[test]
fn empty_state_tolerance_and_dump_match_native_layout() {
    let mut group = CommonBlock::default();
    assert_eq!(group.tolerance(), 0.0);
    assert_eq!(
        group.edge().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        group.pave_block1().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    group.set_tolerance(1.0e-7);
    assert_eq!(group.tolerance(), 1.0e-7);
    group.add_pave_block(block(2, 7));
    group.add_face(11);
    let mut dump = String::new();
    group.dump(&mut dump).unwrap();
    assert_eq!(
        dump,
        " -- CB:\n PB:{ E:7 orE:2 Pave1: { -1 99.000 } Pave2: { -1 99.000 } }\n Faces: 11\n"
    );
}
