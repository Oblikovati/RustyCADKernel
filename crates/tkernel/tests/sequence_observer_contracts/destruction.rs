use rustycad_tkernel::ncollection::{Sequence, SequenceNativeCursor, SequenceObserver};
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
struct ThreadCheckedDrop {
    observer: SequenceObserver,
    expected_size: usize,
}
impl Drop for ThreadCheckedDrop {
    fn drop(&mut self) {
        let observer = self.observer.clone();
        let snapshot = std::thread::spawn(move || observer.snapshot())
            .join()
            .unwrap();
        assert_eq!(snapshot.size, self.expected_size);
        assert_eq!(snapshot.first_node.is_some(), snapshot.size != 0);
        assert_eq!(snapshot.cached_index, snapshot.size);
    }
}
#[test]
fn every_removal_phase_releases_cache_lock_before_cross_thread_drop_observation() {
    for mode in 0..4 {
        let mut sequence = Sequence::default();
        sequence.append(ThreadCheckedDrop {
            observer: sequence.observer(),
            expected_size: usize::from(mode == 3),
        });
        let mut cursor = SequenceNativeCursor::new(&sequence, true);
        match mode {
            0 => sequence.remove(1).unwrap(),
            1 => sequence.remove_range(1, 1).unwrap(),
            2 => sequence.remove_native(&mut cursor).unwrap(),
            _ => sequence.clear(None),
        }
        assert_eq!(sequence.observer().snapshot().size, 0);
    }
}
type SequenceDropRows = Rc<RefCell<Vec<(usize, usize)>>>;
struct PanicObservedDrop {
    number: usize,
    observer: SequenceObserver,
    once: Rc<Cell<bool>>,
    rows: SequenceDropRows,
}
impl Drop for PanicObservedDrop {
    fn drop(&mut self) {
        self.rows
            .borrow_mut()
            .push((self.number, self.observer.snapshot().size));
        assert!(
            !(self.number == 1 && self.once.replace(false)),
            "injected first-item Drop panic"
        );
    }
}
fn panic_fixture() -> (Sequence<PanicObservedDrop>, SequenceDropRows) {
    let mut sequence = Sequence::default();
    let once = Rc::new(Cell::new(true));
    let rows = Rc::new(RefCell::new(Vec::new()));
    for number in 1..=3 {
        sequence.append(PanicObservedDrop {
            number,
            observer: sequence.observer(),
            once: once.clone(),
            rows: rows.clone(),
        });
    }
    (sequence, rows)
}
#[test]
fn clear_panic_resets_canonical_metadata_before_remaining_owner_unwind() {
    let (mut sequence, rows) = panic_fixture();
    assert!(catch_unwind(AssertUnwindSafe(|| sequence.clear(None))).is_err());
    assert_eq!(sequence.observer().snapshot().size, 0);
    let rows = rows.borrow();
    assert_eq!(rows[0], (1, 3));
    assert_eq!(rows.len(), 3);
    assert!(rows[1..].iter().all(|(_, size)| *size == 0));
}
#[test]
fn range_panic_releases_remaining_detached_owners_with_survivor_metadata() {
    let (mut sequence, rows) = panic_fixture();
    assert!(catch_unwind(AssertUnwindSafe(|| sequence.remove_range(1, 2))).is_err());
    assert_eq!(*rows.borrow(), [(1, 1), (2, 1)]);
    assert_eq!(sequence.len(), 1);
    assert_eq!(sequence.first().unwrap().number, 3);
    assert_eq!(sequence.observer().snapshot().size, 1);
}
#[test]
fn cursor_panic_preserves_already_advanced_canonical_cursor() {
    let (mut sequence, rows) = panic_fixture();
    let mut cursor = SequenceNativeCursor::new(&sequence, true);
    let view = cursor.observer();
    assert!(catch_unwind(AssertUnwindSafe(|| sequence.remove_native(&mut cursor))).is_err());
    assert_eq!(*rows.borrow(), [(1, 2)]);
    assert_eq!(view.current_node(), sequence.cursor().current_node());
    assert_eq!(view.previous_node(), None);
    assert_eq!(cursor.position().value(&sequence).unwrap().number, 2);
}
