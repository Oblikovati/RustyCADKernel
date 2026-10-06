use rustycad_tkernel::ncollection::{
    Sequence, SequenceCursorObserver, SequenceNativeCursor, SequenceObserver,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Default)]
pub(super) struct SequenceTrace {
    pub(super) rows: RefCell<Vec<String>>,
    identities: RefCell<Vec<usize>>,
    cursor: RefCell<Option<SequenceCursorObserver>>,
    pub(super) recording: Cell<bool>,
}
pub(super) struct SequenceObservedItem {
    number: usize,
    observer: SequenceObserver,
    trace: Rc<SequenceTrace>,
}
impl Drop for SequenceObservedItem {
    fn drop(&mut self) {
        if self.trace.recording.get() {
            self.trace
                .record(&format!("drop {}", self.number), &self.observer);
        }
    }
}
impl SequenceTrace {
    fn identity(&self, address: Option<usize>) -> usize {
        address.map_or(0, |address| {
            self.identities
                .borrow()
                .iter()
                .position(|known| *known == address)
                .expect("native identity remains in fixture")
                + 1
        })
    }
    fn cursor_fields(&self) -> String {
        let cursor = self.cursor.borrow();
        let more = usize::from(cursor.as_ref().is_some_and(SequenceCursorObserver::more));
        let current = cursor
            .as_ref()
            .and_then(SequenceCursorObserver::current_node);
        let previous = cursor
            .as_ref()
            .and_then(SequenceCursorObserver::previous_node);
        format!(
            "{more} {} {}",
            self.identity(current),
            self.identity(previous)
        )
    }
    fn record(&self, prefix: &str, observer: &SequenceObserver) {
        let observed = observer.snapshot();
        self.rows.borrow_mut().push(format!(
            "{prefix} {} {} {} {} {} {} {}",
            observed.size,
            usize::from(observed.size == 0),
            self.identity(observed.first_node),
            self.identity(observed.last_node),
            self.identity(observed.cached_node),
            observed.cached_index,
            self.cursor_fields()
        ));
    }
}
fn fixture(count: usize, cache: usize) -> (Sequence<SequenceObservedItem>, Rc<SequenceTrace>) {
    let mut sequence = Sequence::default();
    let observer = sequence.observer();
    let trace = Rc::new(SequenceTrace::default());
    for number in 1..=count {
        sequence.append(SequenceObservedItem {
            number,
            observer: observer.clone(),
            trace: trace.clone(),
        });
    }
    capture_identities(&sequence, &trace);
    if cache != 0 {
        sequence.value(cache).unwrap();
    }
    (sequence, trace)
}
fn execute(
    kind: usize,
    first: usize,
    last: usize,
    sequence: &mut Sequence<SequenceObservedItem>,
    cursor: &mut SequenceNativeCursor,
) {
    match kind {
        0 => sequence.remove(first).unwrap(),
        1 => sequence.remove_native(cursor).unwrap(),
        2 => sequence.remove_range(first, last).unwrap(),
        _ => sequence.clear(None),
    }
}
fn case(kind: usize, count: usize, cache: usize, first: usize, last: usize) -> Vec<String> {
    let (mut sequence, trace) = fixture(count, cache);
    let mut cursor = SequenceNativeCursor::new(&sequence, true);
    if kind == 1 {
        for _ in 0..first {
            cursor.next(&sequence).unwrap();
        }
        *trace.cursor.borrow_mut() = Some(cursor.observer());
    }
    trace
        .rows
        .borrow_mut()
        .push(format!("case {kind} {count} {cache} {first} {last}"));
    trace.recording.set(true);
    execute(kind, first, last, &mut sequence, &mut cursor);
    trace.recording.set(false);
    trace.record("after", &sequence.observer());
    trace.rows.borrow().clone()
}
fn cache_cases(count: usize, cache: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for index in 1..=count {
        rows.extend(case(0, count, cache, index, 0));
    }
    for position in 0..=count {
        rows.extend(case(1, count, cache, position, 0));
    }
    for first in 1..=count {
        for last in first..=count {
            rows.extend(case(2, count, cache, first, last));
        }
    }
    rows.extend(case(3, count, cache, 0, 0));
    rows
}
pub(super) fn all_rows() -> Vec<String> {
    let mut rows = Vec::new();
    for count in 0..=3 {
        for cache in 0..=count {
            rows.extend(cache_cases(count, cache));
        }
    }
    rows
}

fn capture_identities(sequence: &Sequence<SequenceObservedItem>, trace: &SequenceTrace) {
    let mut cursor = sequence.cursor();
    while cursor.more() {
        trace
            .identities
            .borrow_mut()
            .push(cursor.current_node().unwrap());
        cursor.next(sequence);
    }
}
