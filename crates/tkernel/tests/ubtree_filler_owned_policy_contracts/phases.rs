use super::{allocation_trace::*, caught, native_mode, self_assign};
use rustycad_tkernel::ncollection::{
    BaseAllocator, BoundingTreeInsertion, EBTree, HashKey, UBTreeFiller,
};
impl HashKey for Payload<'O'> {
    fn occt_hash(&self) -> usize {
        self.0.occt_hash()
    }
}
fn retry_case(kind: char) {
    let arena = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Tree::with_allocator(arena.clone());
    let mut target = tree.native_insertion();
    let mut filler =
        UBTreeFiller::new(&mut target, None, true).with_self_assignment_policy(self_assign);
    filler.add(Payload(1), Payload(10));
    start(&arena, ('A', kind, 1));
    assert!(caught(|| {
        filler.fill();
    }));
    let failed = format!("caught {}", finish());
    start(&arena, ('X', 'X', 0));
    let retried = format!("retry {} {}", filler.fill(), finish());
    assert_eq!(vec![failed, retried], native_mode(&format!("{kind}e")));
}
#[test]
fn explicit_failure_preserves_native_queue_and_partial_tree_for_retry() {
    retry_case('O');
    retry_case('B');
}
fn queue_and_fill<Target: BoundingTreeInsertion<Payload<'O'>, Payload<'B'>> + ?Sized>(
    target: &mut Target,
    arena: &BaseAllocator,
    count: i32,
    full: bool,
    duplicate: bool,
    implicit: bool,
) -> (String, String, i32, usize) {
    let pending = BaseAllocator::accumulating(65536).unwrap();
    let mut filler = UBTreeFiller::new(target, Some(pending.clone()), full)
        .with_self_assignment_policy(self_assign);
    start(arena, ('X', 'X', 0));
    queue_payloads(&mut filler, count, duplicate);
    let appended = finish();
    start(arena, ('X', 'X', 0));
    let result = if implicit { -1 } else { filler.fill() };
    drop(filler);
    (appended, finish(), result, allocations(&pending))
}
fn queue_payloads<Target: BoundingTreeInsertion<Payload<'O'>, Payload<'B'>> + ?Sized>(
    filler: &mut UBTreeFiller<'_, Payload<'O'>, Payload<'B'>, Target>,
    count: i32,
    duplicate: bool,
) {
    for index in 1..=count {
        let value = if duplicate { 1 } else { index };
        // Pinned GCC constructs native call arguments right-to-left, then destroys object first.
        let bounds = Payload(value * 10);
        let object = Payload(value);
        filler.add_ref_with_constructor(&object, &bounds, |pair| pair.clone());
    }
}
fn phase_case(count: i32, full: bool, editable: bool, duplicate: bool, implicit: bool) {
    let arena = BaseAllocator::accumulating(65536).unwrap();
    let mut base = Tree::with_allocator(arena.clone());
    let mut derived = EBTree::with_allocator(arena.clone());
    let mut base_target = base.native_insertion();
    let mut derived_target = derived.native_insertion();
    let target: &mut dyn BoundingTreeInsertion<Payload<'O'>, Payload<'B'>> = if editable {
        &mut derived_target
    } else {
        &mut base_target
    };
    let observations = queue_and_fill(target, &arena, count, full, duplicate, implicit);
    let prefix = phase_prefix(count, [full, editable, duplicate, implicit]);
    assert_phase(&prefix, observations, allocations(&arena));
}
fn phase_prefix(count: i32, flags: [bool; 4]) -> String {
    let [full, editable, duplicate, implicit] = flags.map(u8::from);
    format!("{count} {full} {editable} {duplicate} {implicit} ")
}
fn assert_phase(
    prefix: &str,
    (appended, filled, result, pending): (String, String, i32, usize),
    blocks: usize,
) {
    let row = format!("{prefix}{appended} {filled} {result} {blocks} {pending}");
    let native =
        include_str!("../../../../tools/oracle/tkernel_ubtree_filler_phase_observations.expected")
            .lines()
            .find(|line| line.starts_with(prefix))
            .unwrap();
    assert_eq!(row, native);
}
#[test]
fn complete_native_success_stream_has_no_second_fill_or_cleanup_callbacks() {
    for count in 1..=3 {
        for full in [false, true] {
            phase_case(count, full, false, false, false);
        }
    }
    for full in [false, true] {
        phase_case(3, full, true, true, false);
    }
    phase_case(2, true, false, false, true);
    phase_case(3, true, true, true, true);
}
#[test]
fn reset_cancels_automatic_insertion_and_deref_preserves_check_tree() {
    let mut tree = Tree::default();
    {
        let mut target = tree.native_insertion();
        let mut filler =
            UBTreeFiller::new(&mut target, None, true).with_self_assignment_policy(self_assign);
        filler.add_ref(&Payload(1), &Payload(10));
        assert!(filler.check_tree(&mut String::new()).is_err());
        filler.reset();
        assert_eq!(filler.fill(), 0);
    }
    assert!(tree.is_empty());
}
