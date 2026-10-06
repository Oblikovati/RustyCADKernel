//! Const range adapters share one cursor, as NCollection_ForwardRange.hxx requires.
use rustycad_tkernel::ncollection::forward_range::*;

#[derive(Clone)]
struct CountingHost {
    position: usize,
}
struct PositionAccessor;
impl ReadAccessor<CountingHost> for PositionAccessor {
    type Item = usize;
    type Value = usize;
    fn read(host: &CountingHost) -> usize {
        host.position
    }
    fn snapshot(item: usize) -> usize {
        item
    }
}
impl ForwardHost for CountingHost {
    type Accessors = AccessorSet<Present<PositionAccessor>, Missing, Missing>;
    fn more(&self) -> bool {
        self.position < 3
    }
    fn advance(&mut self) {
        self.position += 1;
    }
}

#[test]
fn const_begin_and_cbegin_share_the_current_cursor() {
    let range = ForwardRange::new(CountingHost { position: 0 });
    let mut first = range.begin();
    let second = range.cbegin();
    assert_eq!(first.next(), Some(0));
    assert_eq!(second.current(), Some(1));
    assert_eq!(*first.post_increment().unwrap(), 1);
    assert_eq!(second.arrow(), Some(2));
    first.pre_increment();
    assert!(first == range.end());
    assert!(range.cend() == second);
    assert!(second.current().is_none());
}

#[test]
fn prefix_increment_returns_adapter_and_delegates_at_exhaustion() {
    let mut host = CountingHost { position: 2 };
    let mut iter = ForwardRangeIter::new(&mut host);
    let original = std::ptr::from_mut(&mut iter);
    assert_eq!(std::ptr::from_mut(iter.pre_increment()), original);
    assert!(iter == ForwardRangeSentinel);
    iter.pre_increment();
    assert_eq!(host.position, 4);
}

#[test]
fn shared_prefix_also_delegates_after_end_but_checked_advance_does_not() {
    let range = ForwardRange::new(CountingHost { position: 2 });
    let mut iter = range.begin();
    iter.pre_increment().pre_increment();
    assert!(range.end() == iter);
    iter.advance();
    assert!(iter.post_increment().is_none());
    assert!(range.cbegin().next().is_none());
}

#[test]
fn owning_copy_and_move_keep_independent_and_preserved_positions() {
    let host = CountingHost { position: 0 };
    let range = ForwardRange::from_copy(&host);
    assert_eq!(range.cbegin().next(), Some(0));
    let mut moved = range;
    assert_eq!(moved.iter().next(), Some(1));
    assert_eq!(moved.begin().next(), Some(2));
    assert!(moved.cbegin() == moved.cend());
    assert_eq!(host.position, 0);
}

struct BorrowedHost<'a> {
    values: &'a [String],
    position: usize,
}
struct BorrowedAccessor;
impl<'a> ReadAccessor<BorrowedHost<'a>> for BorrowedAccessor {
    type Item = &'a String;
    type Value = String;
    fn read(host: &BorrowedHost<'a>) -> Self::Item {
        &host.values[host.position]
    }
    fn snapshot(item: Self::Item) -> String {
        item.clone()
    }
}
impl ForwardHost for BorrowedHost<'_> {
    type Accessors = AccessorSet<Present<BorrowedAccessor>, Missing, Missing>;
    fn more(&self) -> bool {
        self.position < self.values.len()
    }
    fn advance(&mut self) {
        self.position += 1;
    }
}

#[test]
fn shared_reference_result_outlives_cursor_borrow_and_postfix_owns_snapshot() {
    let values = [String::from("first"), String::from("second")];
    let range = ForwardRange::new(BorrowedHost {
        values: &values,
        position: 0,
    });
    let mut first = range.begin();
    let old_reference = first.current().unwrap();
    let snapshot = first.post_increment().unwrap();
    assert_eq!(old_reference, "first");
    assert_eq!(*snapshot, "first");
    assert_eq!(range.cbegin().arrow().unwrap(), "second");
    assert!(!std::ptr::eq(old_reference, &*snapshot));
}
