use super::*;
struct BorrowedDrop<'a>(&'a Cell<usize>);
impl Drop for BorrowedDrop<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[test]
fn abandoned_owner_releases_lease_and_handle_without_dropping_borrowed_payload() {
    let arena = BaseAllocator::incremental(1024);
    let drops = Cell::new(0);
    let owner = SequenceNodeOwner::construct(&arena, || BorrowedDrop(&drops));
    let pointer = NonNull::new(owner.link.cast::<SequenceNode<BorrowedDrop<'_>>>()).unwrap();
    owner.abandon();
    assert_eq!((arena.ref_count(), drops.get()), (1, 0));
    // SAFETY: original initialized node provenance is retained; no reset or owner intervened.
    let recovered = unsafe { arena.adopt_abandoned_value(pointer) };
    assert!(arena.reset(false).is_err());
    drop(recovered);
    assert_eq!(drops.get(), 1);
    assert!(arena.reset(true).is_ok());
}
#[test]
fn failed_guard_clears_observer_topology_before_ordinary_cleanup() {
    let arena = BaseAllocator::incremental(1024);
    let drops = Cell::new(0);
    let mut target = Sequence::with_allocator(arena.clone());
    target.emplace_append(|| BorrowedDrop(&drops));
    let observer = target.observer();
    drop(IncompleteSequenceCopy {
        destination: Some(&mut target),
    });
    assert!(target.is_empty());
    assert!(observer.snapshot().first_node.is_none());
    drop(target);
    assert_eq!(drops.get(), 0);
    assert!(arena.reset(true).is_ok());
}
#[test]
fn non_clone_callback_failure_frees_current_node_and_relinquishes_prefix_leases() {
    let arena = BaseAllocator::incremental(1024);
    let drops = Cell::new(0);
    let mut source = Sequence::with_allocator(arena.clone());
    source.emplace_append(|| BorrowedDrop(&drops));
    source.emplace_append(|| BorrowedDrop(&drops));
    let copied = Cell::new(0);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        HSequence::from_sequence_with_constructor(&source, |_| reject_second_copy(&copied, &drops))
    }));
    assert!(failure.is_err());
    assert_eq!(drops.get(), 0);
    drop(source);
    assert_eq!(drops.get(), 2);
    assert!(arena.reset(true).is_ok());
}
fn reject_second_copy<'a>(copied: &Cell<usize>, drops: &'a Cell<usize>) -> BorrowedDrop<'a> {
    copied.set(copied.get() + 1);
    assert_ne!(
        copied.get(),
        2,
        "copy index=2; expected successful first copy only"
    );
    BorrowedDrop(drops)
}
#[test]
fn copy_paths_preserve_send_sync_unpin_and_borrowed_covariance() {
    fn thread_traits<T: Send + Sync + Unpin>() {}
    fn shorten<'a>(source: Sequence<&'static str>, _: &'a str) -> Sequence<&'a str> {
        source
    }
    thread_traits::<Sequence<PhantomPinned>>();
    thread_traits::<HSequence<PhantomPinned>>();
    let shorter = String::from("short");
    let source = shorten(["OCCT"].into_iter().collect(), &shorter);
    let copied = source.clone();
    assert_eq!(copied.first().unwrap(), &"OCCT");
}
