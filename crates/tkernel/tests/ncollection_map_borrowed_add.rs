//! Supplemental borrowed Add/Added and shared owned lookup regressions.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, Map};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type KeyEvents = Rc<RefCell<Vec<char>>>;
type BorrowedMap = Map<BorrowedKey, KeyHash>;
type BorrowedFixture = (BaseAllocator, KeyEvents, BorrowedKey, BorrowedMap);

#[derive(Debug)]
struct BorrowedKey {
    number: usize,
    allocator: BaseAllocator,
    events: KeyEvents,
    allocation_seen: Rc<Cell<bool>>,
    fail_copy: bool,
}

impl Clone for BorrowedKey {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push('K');
        self.allocation_seen
            .set(self.allocator.reset(false).is_err());
        assert!(!self.fail_copy, "injected key {} clone panic", self.number);
        Self {
            number: self.number,
            allocator: self.allocator.clone(),
            events: Rc::clone(&self.events),
            allocation_seen: Rc::clone(&self.allocation_seen),
            fail_copy: false,
        }
    }
}

struct KeyHash(KeyEvents);
impl HashPolicy<BorrowedKey> for KeyHash {
    fn hash(&self, key: &BorrowedKey) -> usize {
        self.0.borrow_mut().push('H');
        key.number
    }
    fn equal(&self, first: &BorrowedKey, second: &BorrowedKey) -> bool {
        self.0.borrow_mut().push('E');
        first.number == second.number
    }
}

fn borrowed_fixture(fail_copy: bool) -> BorrowedFixture {
    let arena = BaseAllocator::incremental(256);
    let events: KeyEvents = Rc::new(RefCell::new(Vec::new()));
    let key = BorrowedKey {
        number: 4,
        allocator: arena.clone(),
        events: Rc::clone(&events),
        allocation_seen: Rc::new(Cell::new(false)),
        fail_copy,
    };
    let keys = Map::with_allocator(KeyHash(Rc::clone(&events)), 101, arena.clone());
    (arena, events, key, keys)
}

#[test]
fn borrowed_add_hashes_once_and_allocates_before_copy_without_copying_on_hit() {
    let (arena, events, mut key, mut keys) = borrowed_fixture(false);
    assert!(keys.add_ref(&key));
    assert_eq!(*events.borrow(), ['H', 'K']);
    assert!(key.allocation_seen.get());
    let live = arena.reset(false).unwrap_err().message().to_owned();
    key.fail_copy = true;
    events.borrow_mut().clear();
    assert!(!keys.add_ref(&key));
    assert_eq!(*events.borrow(), ['H', 'E']);
    assert_eq!(arena.reset(false).unwrap_err().message(), live);
    assert_eq!(key.number, 4);
    assert_eq!(keys.len(), 1);
}

#[test]
fn borrowed_added_returns_the_same_stored_key_without_an_extra_lookup() {
    let (_, events, mut key, mut keys) = borrowed_fixture(false);
    let stored = std::ptr::from_ref(keys.added_ref(&key));
    assert_ne!(stored, std::ptr::from_ref(&key));
    assert_eq!(*events.borrow(), ['H', 'K']);
    key.fail_copy = true;
    events.borrow_mut().clear();
    let returned = std::ptr::from_ref(keys.added_ref(&key));
    assert_eq!(returned, stored);
    assert_eq!(*events.borrow(), ['H', 'E']);
}

#[test]
fn borrowed_key_copy_failure_releases_node_without_publishing_a_key() {
    let (arena, events, key, mut keys) = borrowed_fixture(true);
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| keys.add_ref(&key)));
    assert!(failure.is_err());
    assert_eq!(*events.borrow(), ['H', 'K']);
    assert!(key.allocation_seen.get());
    assert!(keys.is_empty());
    arena.reset(false).unwrap();
}

#[test]
fn owned_add_uses_one_hash_on_miss_and_no_key_copy() {
    let (_, events, key, mut keys) = borrowed_fixture(true);
    assert!(keys.add(key));
    assert_eq!(*events.borrow(), ['H']);
    assert_eq!(keys.len(), 1);
}
