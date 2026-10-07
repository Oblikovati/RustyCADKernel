use super::{assert_native, caught, event_text, policy::*};
use crate::allocation_trace::*;
use rustycad_tkernel::ncollection::{BaseAllocator, BoundingVolume, UBTree, UBTreeFiller};
fn append<O: QueuePayload, B: QueuePayload + BoundingVolume>(
    filler: &mut UBTreeFiller<'_, O, B>,
    value: i32,
) {
    filler.add_ref_with_constructor(&O::item(value), &B::item(value * 10), construct_pair);
}
fn add_phase<O: QueuePayload, B: QueuePayload + BoundingVolume>(
    filler: &mut UBTreeFiller<'_, O, B>,
    arena: &BaseAllocator,
    rejection: &str,
) -> (bool, String) {
    let input = (O::item(9), B::item(90));
    start(arena, rejection_tuple(rejection));
    let failed = caught(|| filler.add_ref_with_constructor(&input.0, &input.1, construct_pair));
    (failed, event_text())
}
fn reset_phase<O: QueuePayload, B: QueuePayload + BoundingVolume>(
    filler: &mut UBTreeFiller<'_, O, B>,
    arena: &BaseAllocator,
) -> String {
    start(arena, ('X', 'X', 0));
    filler.reset();
    let events = event_text();
    assert_eq!(filler.fill(), 0);
    events
}
pub fn queue_case<O: QueuePayload, B: QueuePayload + BoundingVolume>(
    moving: bool,
    before: bool,
    rejection: &str,
) {
    let arena = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = UBTree::default();
    let mut filler = UBTreeFiller::new(&mut tree, Some(arena.clone()), true);
    if before {
        append::<O, B>(&mut filler, 1);
    }
    let phases = queued_phases(&mut filler, &arena, rejection);
    drop(filler);
    let prefix = format!(
        "queued {} {} {rejection} ",
        u8::from(moving),
        u8::from(before)
    );
    assert_after_drop(&prefix, &phases, &arena, tree.is_empty());
}
fn assert_after_drop(prefix: &str, phases: &str, arena: &BaseAllocator, tree_empty: bool) {
    let row = format!(
        "{prefix}{phases} {} {}",
        allocations(arena),
        u8::from(tree_empty)
    );
    assert_native(prefix, &row);
}

fn queued_phases<O: QueuePayload, B: QueuePayload + BoundingVolume>(
    filler: &mut UBTreeFiller<'_, O, B>,
    arena: &BaseAllocator,
    rejection: &str,
) -> String {
    let (failed, added) = add_phase(filler, arena, rejection);
    let after = allocations(arena);
    let cleared = reset_phase(filler, arena);
    let remaining = allocations(arena);
    format!("{} {added} {after} {cleared} {remaining}", u8::from(failed))
}
