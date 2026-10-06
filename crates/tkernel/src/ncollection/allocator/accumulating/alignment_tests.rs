//! Checked reservation geometry must not mutate ownership on failure.
use super::AccEngine;

#[test]
fn unrepresentable_alignment_slack_keeps_current_graph() {
    let mut engine = AccEngine::new(128).unwrap();
    let before = engine.chain();
    assert!(engine.allocate_aligned_block(usize::MAX, 64).is_none());
    assert_eq!(engine.chain(), before);
    assert!(engine.allocate_aligned(usize::MAX, 8).is_none());
    assert_eq!(engine.chain(), before);
}
#[test]
fn oversized_cursor_subtraction_cannot_wrap_into_owned_block() {
    let engine = AccEngine::new(128).unwrap();
    let slot = &engine.blocks[&engine.last_block.unwrap()];
    assert_eq!(slot.reservation(usize::MAX, 8), None);
    assert_eq!(slot.reservation(256, 8), None);
    assert_eq!(slot.reservation(32, 8), Some(32));
    assert_eq!(slot.descriptor.alloc_count, 0);
}
#[test]
fn fresh_small_block_honors_strict_alignment_without_extra_count() {
    let mut engine = AccEngine::new(16).unwrap();
    let pointer = engine.allocate_aligned(64, 64).unwrap();
    assert_eq!(pointer.as_ptr().addr() % 64, 0);
    assert_eq!(engine.chain().len(), 2);
    assert_eq!(
        engine
            .chain()
            .iter()
            .map(|block| block.alloc_count)
            .sum::<i32>(),
        1
    );
    // Native findBlock has only a two-key window; owned release uses the retained block extent.
    engine.free_owned(pointer.as_ptr().addr()).unwrap();
    assert_eq!(engine.chain().len(), 1);
}
