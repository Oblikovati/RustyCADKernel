//! Supplemental borrowed Add/Added and shared owned lookup regressions.
//!
//! The native probe (tools/oracle/ncollection_map_borrowed_add.cxx) also traces the node
//! allocation `A` through an injected C++ allocator subclass. Rust allocators are a closed set
//! and map entries take no allocator lease, so that event is not observable here (plan D1
//! class B6); the hash, equality and key-copy order is.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, Map};
use std::cell::RefCell;
use std::rc::Rc;

type KeyEvents = Rc<RefCell<Vec<char>>>;
type BorrowedMap = Map<BorrowedKey, KeyHash>;
type BorrowedFixture = (BaseAllocator, KeyEvents, BorrowedKey, BorrowedMap);

#[derive(Debug)]
struct BorrowedKey {
    number: usize,
    events: KeyEvents,
    fail_copy: bool,
}

impl Clone for BorrowedKey {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push('K');
        assert!(!self.fail_copy, "injected key {} clone panic", self.number);
        Self {
            number: self.number,
            events: Rc::clone(&self.events),
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
        events: Rc::clone(&events),
        fail_copy,
    };
    let keys = Map::with_allocator(KeyHash(Rc::clone(&events)), 101, arena.clone());
    (arena, events, key, keys)
}

#[test]
fn borrowed_add_hashes_once_and_copies_without_copying_on_hit() {
    let (arena, events, mut key, mut keys) = borrowed_fixture(false);
    // Native miss: H A K; native hit: H E.
    assert!(keys.add_ref(&key));
    assert_eq!(*events.borrow(), ['H', 'K']);
    key.fail_copy = true;
    events.borrow_mut().clear();
    assert!(!keys.add_ref(&key));
    assert_eq!(*events.borrow(), ['H', 'E']);
    assert_eq!(key.number, 4);
    assert_eq!((keys.len(), keys.allocator()), (1, &arena));
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
fn borrowed_key_copy_failure_publishes_no_key() {
    let (_, events, key, mut keys) = borrowed_fixture(true);
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| keys.add_ref(&key)));
    assert!(failure.is_err());
    // Native: H A K F with extent 0.
    assert_eq!(*events.borrow(), ['H', 'K']);
    assert!(keys.is_empty());
}

#[test]
fn owned_add_uses_one_hash_on_miss_and_no_key_copy() {
    let (_, events, key, mut keys) = borrowed_fixture(true);
    assert!(keys.add(key));
    assert_eq!(*events.borrow(), ['H']);
    assert_eq!(keys.len(), 1);
}
