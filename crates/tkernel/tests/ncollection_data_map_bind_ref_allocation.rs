//! Verifies DataMap borrowed insertion allocates before native-ordered subobject copies.
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap, HashPolicy};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug)]
struct Tracked {
    number: i32,
    kind: char,
    allocator: BaseAllocator,
    events: Rc<RefCell<Vec<char>>>,
    allocation_live_during_clone: Rc<Cell<bool>>,
    panic_on_clone: bool,
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.events
            .borrow_mut()
            .push(self.kind.to_ascii_lowercase());
    }
}

impl Clone for Tracked {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push(self.kind);
        self.allocation_live_during_clone
            .set(self.allocator.reset(false).is_err());
        assert!(!self.panic_on_clone, "injected {} clone panic", self.kind);
        Self {
            number: self.number,
            kind: self.kind,
            allocator: self.allocator.clone(),
            events: Rc::clone(&self.events),
            allocation_live_during_clone: Rc::clone(&self.allocation_live_during_clone),
            panic_on_clone: false,
        }
    }
}

type EventLog = Rc<RefCell<Vec<char>>>;
type TestMap = DataMap<Tracked, Tracked, TraceHash>;
type InsertionFixture = (BaseAllocator, EventLog, Tracked, Tracked, TestMap);

struct TraceHash(EventLog);

impl HashPolicy<Tracked> for TraceHash {
    fn hash(&self, key: &Tracked) -> usize {
        self.0.borrow_mut().push('H');
        key.number as usize
    }

    fn equal(&self, first: &Tracked, second: &Tracked) -> bool {
        self.0.borrow_mut().push('E');
        first.number == second.number
    }
}

fn tracked(
    number: i32,
    kind: char,
    allocator: &BaseAllocator,
    events: &EventLog,
    panic_on_clone: bool,
) -> Tracked {
    Tracked {
        number,
        kind,
        allocator: allocator.clone(),
        events: Rc::clone(events),
        allocation_live_during_clone: Rc::new(Cell::new(false)),
        panic_on_clone,
    }
}

fn event_trace(events: &EventLog) -> Vec<char> {
    events.borrow().clone()
}

#[test]
fn bind_ref_allocates_before_value_then_key_copy_and_hit_keeps_native_path() {
    let (arena, events, key, value, mut map) = insertion_fixture(false);
    assert!(map.bind_ref(&key, &value));
    assert_eq!(event_trace(&events), ['H', 'V', 'K']);
    assert!(value.allocation_live_during_clone.get());
    assert!(key.allocation_live_during_clone.get());
    let live_allocations = arena.reset(false).unwrap_err().message().to_owned();
    events.borrow_mut().clear();
    assert!(!map.bind_ref(&key, &value));
    assert_eq!(event_trace(&events), ['H', 'E', 'V', 'v']);
    assert_eq!(arena.reset(false).unwrap_err().message(), live_allocations);
    assert_eq!(map.len(), 1);
    drop(map);
    arena.reset(false).unwrap();
}

fn insertion_fixture(panic_on_key: bool) -> InsertionFixture {
    let allocator = BaseAllocator::incremental(256);
    let events = Rc::new(RefCell::new(Vec::new()));
    let key = tracked(4, 'K', &allocator, &events, panic_on_key);
    let value = tracked(9, 'V', &allocator, &events, false);
    let map = DataMap::with_allocator(TraceHash(Rc::clone(&events)), 101, allocator.clone());
    (allocator, events, key, value, map)
}

#[test]
fn bind_ref_key_clone_panic_releases_the_preallocated_node() {
    let (arena, events, key, value, mut map) = insertion_fixture(true);
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.bind_ref(&key, &value)));
    assert!(result.is_err());
    assert_eq!(event_trace(&events), ['H', 'V', 'K', 'v']);
    assert!(map.is_empty());
    arena.reset(false).unwrap();
}
