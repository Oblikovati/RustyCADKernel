//! Verifies DataMap borrowed insertion copies the value before the key, as native Bind does.
//!
//! The native probe (tools/oracle/ncollection_data_map_bind_ref_allocation.cxx) also traces the
//! node allocation `A` through an injected C++ allocator subclass. Rust allocators are a closed
//! set and map entries take no allocator lease, so that event is not observable here (plan D1
//! class B6); the hash, copy and destructor order is.
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap, HashPolicy};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Tracked {
    number: i32,
    kind: char,
    events: Rc<RefCell<Vec<char>>>,
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
        assert!(!self.panic_on_clone, "injected {} clone panic", self.kind);
        Self {
            number: self.number,
            kind: self.kind,
            events: Rc::clone(&self.events),
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

fn tracked(number: i32, kind: char, events: &EventLog, panic_on_clone: bool) -> Tracked {
    Tracked {
        number,
        kind,
        events: Rc::clone(events),
        panic_on_clone,
    }
}

fn event_trace(events: &EventLog) -> Vec<char> {
    events.borrow().clone()
}

#[test]
fn bind_ref_copies_value_then_key_and_hit_keeps_native_path() {
    let (arena, events, key, value, mut map) = insertion_fixture(false);
    // Native miss: H A V K (A is the allocator event of the C++ probe).
    assert!(map.bind_ref(&key, &value));
    assert_eq!(event_trace(&events), ['H', 'V', 'K']);
    events.borrow_mut().clear();
    // Native hit: H E V M D; the moved-from temporary has no Rust destructor event.
    assert!(!map.bind_ref(&key, &value));
    assert_eq!(event_trace(&events), ['H', 'E', 'V', 'v']);
    assert_eq!((map.len(), map.allocator()), (1, &arena));
    assert_eq!(map.find(&key).map(|stored| stored.number), Ok(9));
}

fn insertion_fixture(panic_on_key: bool) -> InsertionFixture {
    let allocator = BaseAllocator::incremental(256);
    let events = Rc::new(RefCell::new(Vec::new()));
    let key = tracked(4, 'K', &events, panic_on_key);
    let value = tracked(9, 'V', &events, false);
    let map = DataMap::with_allocator(TraceHash(Rc::clone(&events)), 101, allocator.clone());
    (allocator, events, key, value, map)
}

#[test]
fn bind_ref_key_clone_panic_drops_the_value_copy_and_publishes_nothing() {
    let (_arena, events, key, value, mut map) = insertion_fixture(true);
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.bind_ref(&key, &value)));
    assert!(result.is_err());
    // Native: H A V K D F; the value copy is destroyed and the extent stays zero.
    assert_eq!(event_trace(&events), ['H', 'V', 'K', 'v']);
    assert!(map.is_empty());
}
