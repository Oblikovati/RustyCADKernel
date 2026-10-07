use super::{assert_native, caught, event_text};
use crate::allocation_trace::*;
use rustycad_tkernel::ncollection::{BaseAllocator, ObjBnd, UBTreeFiller};
fn self_assign(pair: &mut ObjBnd<Payload<'O'>, Payload<'B'>>) {
    pair.assign_self_with(Payload::assign_self, Payload::assign_self);
}
fn pair_case(kind: char) {
    let arena = BaseAllocator::accumulating(65536).unwrap();
    let mut pair = ObjBnd::new(Payload(9), Payload(90));
    start(&arena, ('A', kind, 1));
    let failed = caught(|| self_assign(&mut pair));
    let row = format!(
        "self {kind} {} {} {} {}",
        u8::from(failed),
        event_text(),
        pair.object.0,
        pair.bounds.0
    );
    assert_native(&format!("self {kind} "), &row);
}
#[test]
fn object_self_assignment_failure_skips_bounds_and_preserves_native_prefix() {
    for kind in ['X', 'O', 'B'] {
        pair_case(kind);
    }
}
fn fill_case(count: i32, full: bool, kind: char) {
    let arena = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Tree::with_allocator(arena.clone());
    let mut target = tree.native_insertion();
    let mut filler = UBTreeFiller::new(&mut target, None, full);
    for index in 1..=count {
        filler.add_ref(&Payload(index), &Payload(index * 10));
    }
    let (failed, result, filled) = fill_phase(&mut filler, &arena, kind);
    start(&arena, ('X', 'X', 0));
    filler.reset();
    let reset = event_text();
    drop(filler);
    verify_filled(
        target.tree_view(),
        &arena,
        (count, full, kind),
        (failed, result, filled, reset),
    );
}
fn verify_filled(
    tree: &Tree,
    arena: &BaseAllocator,
    (count, full, kind): (i32, bool, char),
    (failed, result, filled, reset): (bool, i32, String, String),
) {
    let root = tree.root().unwrap();
    let prefix = format!("filled {count} {} {kind} ", u8::from(full));
    let row = format!(
        "{prefix}{} {result} {filled} {reset} {} {}",
        u8::from(failed),
        root.bounds().0,
        allocations(arena)
    );
    assert_native(&prefix, &row);
}
use rustycad_tkernel::ncollection::{BoundingTreeInsertion, NativeBoundingTree};
type NativeFiller<'filler, 'tree> =
    UBTreeFiller<'filler, Payload<'O'>, Payload<'B'>, NativeBoundingTree<'tree, Tree>>;
fn fill_phase(
    filler: &mut NativeFiller<'_, '_>,
    arena: &BaseAllocator,
    kind: char,
) -> (bool, i32, String) {
    start(arena, ('A', kind, 1));
    let mut result = -1;
    let failed = caught(|| result = filler.fill_with_self_assignment(self_assign));
    (failed, result, event_text())
}
#[test]
fn explicit_fill_matches_full_native_callback_stream_in_both_random_modes() {
    for count in 1..=3 {
        for full in [false, true] {
            fill_case(count, full, 'X');
        }
    }
}
#[test]
fn pending_assignment_failures_leave_queue_for_explicit_reset() {
    for (count, full) in [(1, false), (1, true), (2, false), (2, true)] {
        for kind in ['O', 'B'] {
            fill_case(count, full, kind);
        }
    }
}
