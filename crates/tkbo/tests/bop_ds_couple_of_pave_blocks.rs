//! Regression cases for BOPDS_CoupleOfPaveBlocks nullable handles and indices.
use rustycad_tkbo::bop_ds::{CoupleOfPaveBlocks, PaveBlock, PaveBlockHandle, pave_block_handle};
use std::sync::Arc;

fn pave_block(edge: i32) -> PaveBlockHandle {
    let mut block = PaveBlock::default();
    block.set_original_edge(edge);
    pave_block_handle(block)
}

#[test]
fn defaults_and_scalar_setters_match_native_sentinels() {
    let mut value = CoupleOfPaveBlocks::new();
    assert_eq!(value.index(), -1);
    assert_eq!(value.interference_index(), -1);
    assert_eq!(value.tolerance(), 0.0);
    assert!(value.pave_block1().is_none());
    assert!(value.pave_block2().is_none());
    value.set_index(17);
    value.set_interference_index(23);
    value.set_tolerance(-0.25);
    assert_eq!(value.index(), 17);
    assert_eq!(value.interference_index(), 23);
    assert_eq!(value.tolerance(), -0.25);
}

#[test]
fn constructors_and_setters_copy_nullable_shared_handles() {
    let first = pave_block(3);
    let second = pave_block(8);
    let mut value =
        CoupleOfPaveBlocks::with_pave_blocks(Some(Arc::clone(&first)), Some(Arc::clone(&second)));
    assert!(Arc::ptr_eq(value.pave_block1().unwrap(), &first));
    assert!(Arc::ptr_eq(value.pave_block2().unwrap(), &second));
    let mut output_first = None;
    let mut output_second = None;
    value.pave_blocks_into(&mut output_first, &mut output_second);
    assert!(Arc::ptr_eq(output_first.as_ref().unwrap(), &first));
    assert!(Arc::ptr_eq(output_second.as_ref().unwrap(), &second));
    value.set_pave_block1(None);
    value.set_pave_block2(Some(Arc::clone(&first)));
    assert!(value.pave_block1().is_none());
    assert!(Arc::ptr_eq(value.pave_block2().unwrap(), &first));
    value.set_pave_blocks(Some(Arc::clone(&second)), None);
    assert!(Arc::ptr_eq(value.pave_block1().unwrap(), &second));
    assert!(value.pave_block2().is_none());
}
