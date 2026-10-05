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

#[derive(Debug)]
struct AllocationKey {
    value: i32,
    allocator: BaseAllocator,
    clones: Rc<Cell<usize>>,
    saw_live_allocation: Rc<Cell<bool>>,
    should_panic: Rc<Cell<bool>>,
}

impl Clone for AllocationKey {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);
        self.saw_live_allocation
            .set(self.allocator.reset(false).is_err());
        assert!(!self.should_panic.get(), "injected key clone panic");
        Self {
            value: self.value,
            allocator: self.allocator.clone(),
            clones: Rc::clone(&self.clones),
            saw_live_allocation: Rc::clone(&self.saw_live_allocation),
            should_panic: Rc::clone(&self.should_panic),
        }
    }
}

struct AllocationPolicy;

impl HashPolicy<AllocationKey> for AllocationPolicy {
    fn hash(&self, key: &AllocationKey) -> usize {
        key.value as usize
    }

    fn equal(&self, left: &AllocationKey, right: &AllocationKey) -> bool {
        left.value == right.value
    }
}

fn allocation_key(allocator: &BaseAllocator, should_panic: bool) -> AllocationKey {
    AllocationKey {
        value: 7,
        allocator: allocator.clone(),
        clones: Rc::new(Cell::new(0)),
        saw_live_allocation: Rc::new(Cell::new(false)),
        should_panic: Rc::new(Cell::new(should_panic)),
    }
}

#[test]
fn borrowed_add_allocates_before_clone_and_hit_adds_no_live_node() {
    let arena = BaseAllocator::incremental(128);
    let key = allocation_key(&arena, false);
    let mut map = IndexedMap::with_allocator(AllocationPolicy, 101, arena.clone());

    assert_eq!(map.add_ref(&key), 1);
    assert!(key.saw_live_allocation.get());
    assert_eq!(key.clones.get(), 1);
    let live_nodes = arena.reset(false).unwrap_err().message().to_owned();
    assert_eq!(map.add_ref(&key), 1);
    assert_eq!(key.clones.get(), 1);
    assert_eq!(arena.reset(false).unwrap_err().message(), live_nodes);
}

#[test]
fn borrowed_add_clone_panic_releases_uninitialized_node() {
    let arena = BaseAllocator::incremental(128);
    let key = allocation_key(&arena, true);
    let mut map = IndexedMap::with_allocator(AllocationPolicy, 101, arena.clone());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.add_ref(&key)));

    assert!(result.is_err());
    assert!(key.saw_live_allocation.get());
    assert!(map.is_empty());
    arena.reset(false).unwrap();
}
