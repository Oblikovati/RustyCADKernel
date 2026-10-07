//! Borrowed `IndexedMap::Add` keeps native lookup and key-copy ordering.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, IndexedMap};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug)]
struct Key {
    value: i32,
    events: Rc<RefCell<Vec<char>>>,
}

impl Clone for Key {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push('K');
        Self {
            value: self.value,
            events: Rc::clone(&self.events),
        }
    }
}

struct Policy(Rc<RefCell<Vec<char>>>);

impl HashPolicy<Key> for Policy {
    fn hash(&self, key: &Key) -> usize {
        self.0.borrow_mut().push('H');
        key.value as usize
    }

    fn equal(&self, left: &Key, right: &Key) -> bool {
        self.0.borrow_mut().push('E');
        left.value == right.value
    }
}

fn key(value: i32, events: &Rc<RefCell<Vec<char>>>) -> Key {
    Key {
        value,
        events: Rc::clone(events),
    }
}

#[test]
fn borrowed_add_hashes_before_copying_on_insert() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut map = IndexedMap::with_hasher(Policy(Rc::clone(&events)), 101);
    let incoming = key(7, &events);

    assert_eq!(map.add_ref(&incoming), 1);
    assert_eq!(*events.borrow(), ['H', 'K']);
    assert_eq!(map.find_key(1).unwrap().value, 7);
}

#[test]
fn borrowed_add_hit_does_not_clone_the_key() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut map = IndexedMap::with_hasher(Policy(Rc::clone(&events)), 101);
    let incoming = key(7, &events);
    assert_eq!(map.add_ref(&incoming), 1);
    events.borrow_mut().clear();

    assert_eq!(map.add_ref(&incoming), 1);
    assert_eq!(*events.borrow(), ['H', 'E']);
    assert_eq!(map.len(), 1);
}

// The native allocation trace (`HAK`, `HAKF`; tools/oracle/ncollection_indexed_map_borrowed_add_
// allocation.md) needs an injected C++ allocator subclass: per-node allocation is protected
// state (plan D1 class B6) that slot storage does not model. Its class-A part stays checked:
// hash and search first, one key copy on a miss, none on a hit, an empty map after a copy panic.
#[derive(Debug)]
struct CountedKey {
    value: i32,
    clones: Rc<Cell<usize>>,
    should_panic: Rc<Cell<bool>>,
}

impl Clone for CountedKey {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);
        assert!(!self.should_panic.get(), "injected key clone panic");
        Self {
            value: self.value,
            clones: Rc::clone(&self.clones),
            should_panic: Rc::clone(&self.should_panic),
        }
    }
}

struct CountedPolicy;

impl HashPolicy<CountedKey> for CountedPolicy {
    fn hash(&self, key: &CountedKey) -> usize {
        key.value as usize
    }

    fn equal(&self, left: &CountedKey, right: &CountedKey) -> bool {
        left.value == right.value
    }
}

fn counted_key(should_panic: bool) -> CountedKey {
    CountedKey {
        value: 7,
        clones: Rc::new(Cell::new(0)),
        should_panic: Rc::new(Cell::new(should_panic)),
    }
}

#[test]
fn borrowed_add_copies_once_on_insert_and_never_on_a_hit() {
    let arena = BaseAllocator::incremental(128);
    let key = counted_key(false);
    let mut map = IndexedMap::with_allocator(CountedPolicy, 101, arena);
    assert_eq!(map.add_ref(&key), 1);
    assert_eq!(key.clones.get(), 1);
    assert_eq!(map.add_ref(&key), 1);
    assert_eq!((key.clones.get(), map.len()), (1, 1));
}

#[test]
fn borrowed_add_clone_panic_publishes_nothing() {
    let key = counted_key(true);
    let mut map = IndexedMap::with_hasher(CountedPolicy, 101);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.add_ref(&key)));
    assert!(result.is_err());
    assert!(map.is_empty());
    key.should_panic.set(false);
    assert_eq!(map.add_ref(&key), 1);
}
