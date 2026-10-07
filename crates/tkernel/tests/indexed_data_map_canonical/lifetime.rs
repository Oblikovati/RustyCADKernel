//! Rust unwind extensions keep every binding after partial user operations.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, IndexedDataMap};
use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};
#[derive(Debug, Default)]
struct Events {
    fail: Option<(char, char)>,
    recorded: Vec<(char, char)>,
}
impl Events {
    fn record(&mut self, operation: char, kind: char) {
        self.recorded.push((operation, kind));
        if self.fail == Some((operation, kind)) {
            self.fail = None;
            panic!("requested {operation} failure for payload {kind}");
        }
    }
}
#[derive(Debug)]
struct Payload {
    value: i32,
    kind: char,
    events: Rc<RefCell<Events>>,
}
impl Clone for Payload {
    fn clone(&self) -> Self {
        self.events.borrow_mut().record('C', self.kind);
        Self {
            value: self.value,
            kind: self.kind,
            events: self.events.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
        self.events.borrow_mut().record('A', self.kind);
    }
}
impl Drop for Payload {
    fn drop(&mut self) {
        self.events.borrow_mut().record('D', self.kind);
    }
}
struct Policy;
impl HashPolicy<Payload> for Policy {
    fn hash(&self, key: &Payload) -> usize {
        key.value as usize
    }
    fn equal(&self, first: &Payload, second: &Payload) -> bool {
        first.value == second.value
    }
}
type Map = IndexedDataMap<Payload, Payload, Policy>;
fn payload(value: i32, kind: char, events: &Rc<RefCell<Events>>) -> Payload {
    Payload {
        value,
        kind,
        events: events.clone(),
    }
}
fn setup() -> (Map, Rc<RefCell<Events>>) {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    (Map::with_allocator(Policy, 1, arena), Rc::default())
}
fn fill(map: &mut Map, events: &Rc<RefCell<Events>>) {
    for value in 1..=3 {
        map.add(
            payload(value, 'K', events),
            payload(value * 10, 'V', events),
        );
    }
    events.borrow_mut().recorded.clear();
}
#[test]
fn borrowed_substitute_assigns_key_then_value_into_the_existing_binding() {
    let (mut map, events) = setup();
    fill(&mut map, &events);
    let key = payload(9, 'K', &events);
    let value = payload(90, 'V', &events);
    map.substitute_ref(2, &key, &value).unwrap();
    assert_eq!(events.borrow().recorded, [('A', 'K'), ('A', 'V')]);
    assert_eq!((map.find_index(&key), map.len()), (2, 3));
    assert_eq!(map.find_from_index(2).unwrap().value, 90);
}
#[test]
fn failed_borrowed_copy_publishes_nothing_and_retains_existing_bindings() {
    for kind in ['K', 'V'] {
        let (mut map, events) = setup();
        fill(&mut map, &events);
        let key = payload(9, 'K', &events);
        let value = payload(90, 'V', &events);
        events.borrow_mut().fail = Some(('C', kind));
        assert!(std::panic::catch_unwind(AssertUnwindSafe(|| map.add_ref(&key, &value))).is_err());
        assert_eq!(map.len(), 3);
        assert_eq!(map.find_from_index(3).unwrap().value, 30);
        assert_copy_failure_events(kind, &events.borrow());
    }
}

fn assert_copy_failure_events(kind: char, events: &Events) {
    let expected = if kind == 'V' {
        vec![('C', 'V')]
    } else {
        vec![('C', 'V'), ('C', 'K'), ('D', 'V')]
    };
    assert_eq!(events.recorded, expected);
}
