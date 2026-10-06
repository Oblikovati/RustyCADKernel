use rustycad_tkernel::ncollection::{List, ListCursorObserver, ListNativeCursor, ListObserver};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
#[derive(Default)]
pub(super) struct Trace {
    pub(super) legacy: RefCell<Vec<String>>,
    pub(super) endpoints: RefCell<Vec<String>>,
    identities: RefCell<Vec<usize>>,
    pub(super) cursor: RefCell<Option<ListCursorObserver>>,
    pub(super) recording: Cell<bool>,
}
pub(super) struct ObservedItem {
    pub(super) value: i32,
    pub(super) observer: ListObserver,
    trace: Rc<Trace>,
    fails: bool,
}
impl Drop for ObservedItem {
    fn drop(&mut self) {
        if !self.trace.recording.get() {
            return;
        }
        self.trace
            .record(&format!("drop {}", self.value), &self.observer);
        assert!(
            !self.fails,
            "injected destructor panic for value {}",
            self.value
        );
    }
}
impl Trace {
    pub(super) fn label(&self, value: String) {
        self.legacy.borrow_mut().push(value.clone());
        self.endpoints.borrow_mut().push(value);
    }
    fn identity(&self, pointer: Option<usize>) -> usize {
        pointer.map_or(0, |pointer| {
            self.identities
                .borrow()
                .iter()
                .position(|known| *known == pointer)
                .expect("observer points at fixture node")
                + 1
        })
    }
    fn links(&self, observer: &ListObserver) -> String {
        let cursor = self.cursor.borrow();
        let current = cursor.as_ref().and_then(ListCursorObserver::current_node);
        let previous = cursor.as_ref().and_then(ListCursorObserver::previous_node);
        format!(
            "{} {} {} {}",
            self.identity(observer.first_node()),
            self.identity(observer.last_node()),
            self.identity(current),
            self.identity(previous)
        )
    }
    pub(super) fn record(&self, prefix: &str, observer: &ListObserver) {
        let size = observer.size();
        let empty = u8::from(observer.is_empty());
        let cursor = self.cursor.borrow();
        let more = u8::from(cursor.as_ref().is_some_and(ListCursorObserver::more));
        let legacy = if prefix == "after" && cursor.is_none() {
            format!("{prefix} {size} {empty}")
        } else {
            format!("{prefix} {size} {empty} {more}")
        };
        self.legacy.borrow_mut().push(legacy);
        self.endpoints.borrow_mut().push(format!(
            "{prefix} {size} {empty} {more} {}",
            self.links(observer)
        ));
    }
}
pub(super) fn fixture(count: usize, failure: i32) -> (List<ObservedItem>, Rc<Trace>) {
    let mut list = List::default();
    let observer = list.observer();
    let trace = Rc::new(Trace::default());
    for value in 1..=count as i32 {
        list.append(ObservedItem {
            value,
            observer: observer.clone(),
            trace: trace.clone(),
            fails: value == failure,
        });
    }
    capture_identities(&list, &trace);
    (list, trace)
}
fn capture_identities(list: &List<ObservedItem>, trace: &Trace) {
    let mut cursor = list.cursor();
    while cursor.more() {
        trace
            .identities
            .borrow_mut()
            .push(cursor.current_node().unwrap());
        cursor.next(list);
    }
}
pub(super) fn remove_case(count: usize, position: usize) -> Rc<Trace> {
    let (mut list, trace) = fixture(count, 0);
    let mut cursor = ListNativeCursor::new(&list);
    for _ in 0..position {
        cursor.next(&list).unwrap();
    }
    *trace.cursor.borrow_mut() = Some(cursor.observer());
    trace.label(format!("remove {count} {position}"));
    trace.recording.set(true);
    list.remove_native(&mut cursor).unwrap();
    trace.recording.set(false);
    trace.record("after", &list.observer());
    trace
}
pub(super) fn clear_case() -> Rc<Trace> {
    let (mut list, trace) = fixture(3, 0);
    trace.label("clear".to_owned());
    trace.recording.set(true);
    list.clear();
    trace.recording.set(false);
    trace.record("after", &list.observer());
    trace
}
pub(super) fn rows() -> (Vec<String>, Vec<String>) {
    let mut traces = Vec::new();
    for count in 1..=3 {
        for position in 0..count {
            traces.push(remove_case(count, position));
        }
    }
    traces.push(clear_case());
    let legacy = traces
        .iter()
        .flat_map(|trace| trace.legacy.borrow().clone())
        .collect();
    let endpoints = traces
        .iter()
        .flat_map(|trace| trace.endpoints.borrow().clone())
        .collect();
    (legacy, endpoints)
}
