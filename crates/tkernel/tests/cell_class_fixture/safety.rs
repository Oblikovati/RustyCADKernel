use super::{BaseAllocator, CellFilterCell};
use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[derive(Default)]
struct DropProbe {
    seen: RefCell<Vec<i32>>,
    fail_drop: Cell<Option<i32>>,
    fail_clone: Cell<bool>,
}
struct Observed<'a> {
    id: i32,
    probe: &'a DropProbe,
    stored: bool,
}
impl Clone for Observed<'_> {
    fn clone(&self) -> Self {
        assert!(!self.probe.fail_clone.get(), "injected target copy failure");
        Self {
            id: self.id,
            probe: self.probe,
            stored: true,
        }
    }
}
impl Drop for Observed<'_> {
    fn drop(&mut self) {
        if !self.stored {
            return;
        }
        self.probe.seen.borrow_mut().push(self.id);
        if self.probe.fail_drop.get() == Some(self.id) {
            self.probe.fail_drop.set(None);
            panic!("injected target destructor failure for {}", self.id);
        }
    }
}
fn observed_cell<'a>(arena: &BaseAllocator, probe: &'a DropProbe) -> CellFilterCell<Observed<'a>> {
    let mut cell = CellFilterCell::from_indices(&[4], arena.clone());
    for id in 1..=3 {
        cell.prepend_target(&Observed {
            id,
            probe,
            stored: false,
        });
    }
    cell
}
#[test]
fn standalone_cell_copy_failure_keeps_its_targets() {
    let (arena, probe) = (BaseAllocator::incremental(1024), DropProbe::default());
    let mut cell = observed_cell(&arena, &probe);
    probe.fail_clone.set(true);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        cell.prepend_target(&Observed {
            id: 4,
            probe: &probe,
            stored: false,
        })
    }));
    assert!(failed.is_err());
    assert_eq!(
        cell.targets().map(|target| target.id).collect::<Vec<_>>(),
        [3, 2, 1]
    );
    drop(cell);
    assert!(arena.reset(false).is_ok());
}
fn shorten_cell<'a>(cell: CellFilterCell<&'static str>, _: &'a ()) -> CellFilterCell<&'a str> {
    cell
}
fn require_thread_traits<T: Send + Sync + Unpin>() {}
#[test]
fn standalone_cell_preserves_payload_covariance_and_owned_node_thread_traits() {
    require_thread_traits::<CellFilterCell<i32>>();
    require_thread_traits::<CellFilterCell<std::marker::PhantomPinned>>();
    let cell = CellFilterCell::<&str>::from_indices(&[1], BaseAllocator::incremental(1024));
    let token = ();
    assert_eq!(shorten_cell(cell, &token).index(), [1]);
}
#[test]
fn move_between_arenas_discards_old_targets_and_adopts_the_source_allocator() {
    let (first, second, probe) = (
        BaseAllocator::incremental(1024),
        BaseAllocator::incremental(1024),
        DropProbe::default(),
    );
    let mut source = observed_cell(&first, &probe);
    let mut destination = observed_cell(&second, &probe);
    destination.move_from_discard(&mut source);
    assert!(second.reset(false).is_ok());
    assert!(probe.seen.borrow().is_empty());
    drop(destination);
    drop(source);
    assert_eq!(*probe.seen.borrow(), [3, 2, 1]);
    assert!(first.reset(false).is_ok());
}
