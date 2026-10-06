//! OrderedMap borrowed insertion, assignment and inherited iterator contracts.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, OrderedMap, OrderedMapIter};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct TraceKey {
    number: i32,
    events: Rc<RefCell<Vec<&'static str>>>,
}
impl Clone for TraceKey {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push("copy");
        Self {
            number: self.number,
            events: Rc::clone(&self.events),
        }
    }
}
#[derive(Clone)]
struct TraceHash(i32);
impl HashPolicy<TraceKey> for TraceHash {
    fn hash(&self, key: &TraceKey) -> usize {
        key.events.borrow_mut().push("hash");
        (key.number % self.0) as usize
    }
    fn equal(&self, a: &TraceKey, b: &TraceKey) -> bool {
        a.number % self.0 == b.number % self.0
    }
}
fn key(number: i32, events: &Rc<RefCell<Vec<&'static str>>>) -> TraceKey {
    TraceKey {
        number,
        events: Rc::clone(events),
    }
}
struct NonCloneHash(i32);
impl HashPolicy<i32> for NonCloneHash {
    fn hash(&self, key: &i32) -> usize {
        (*key + self.0) as usize
    }
    fn equal(&self, a: &i32, b: &i32) -> bool {
        a == b
    }
}

#[test]
fn ordered_add_ref_hashes_before_copy_and_duplicates_never_copy() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let input = key(1, &events);
    let mut map = OrderedMap::with_hasher(TraceHash(100), 1);
    assert!(map.add_ref(&input));
    assert_eq!(*events.borrow(), ["hash", "copy"]);
    let stored = map.first().unwrap() as *const TraceKey;
    events.borrow_mut().clear();
    assert!(!map.add_ref(&input));
    assert_eq!(map.added_ref(&input) as *const TraceKey, stored);
    assert_eq!(*events.borrow(), ["hash", "hash"]);
    events.borrow_mut().clear();
    assert_eq!(map.added_ref(&key(2, &events)).number, 2);
    assert_eq!(*events.borrow(), ["hash", "copy"]);
}

#[test]
fn ordered_owned_insertion_hashes_once_on_nonempty_miss() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut map = OrderedMap::with_hasher(TraceHash(100), 1);
    assert!(map.add(key(1, &events)));
    events.borrow_mut().clear();
    assert!(map.add(key(2, &events)));
    assert_eq!(*events.borrow(), ["hash"]);
    events.borrow_mut().clear();
    assert!(map.emplace(key(3, &events)));
    assert_eq!(*events.borrow(), ["hash"]);
}

#[test]
fn ordered_assignment_uses_destination_equivalence_before_cloning() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut source = OrderedMap::with_hasher(TraceHash(100), 1);
    source.add(key(1, &events));
    source.add(key(11, &events));
    let mut target = OrderedMap::with_hasher(TraceHash(10), 1);
    events.borrow_mut().clear();
    target.assign(&source);
    assert_eq!(target.len(), 1);
    assert_eq!(target.first().unwrap().number, 1);
    assert_eq!(*events.borrow(), ["hash", "copy", "hash"]);
    events.borrow_mut().clear();
    let copied = source.clone();
    assert_eq!(copied.len(), 2);
    assert_eq!(*events.borrow(), ["hash", "copy", "hash", "copy"]);
}

#[test]
fn ordered_returning_assignments_preserve_or_swap_native_state() {
    let arena = BaseAllocator::incremental(4096);
    let mut source = OrderedMap::with_hasher(NonCloneHash(7), 1);
    source.add(3);
    let mut target = OrderedMap::with_allocator(NonCloneHash(90), 1009, arena.clone());
    target.add(9);
    let destination = &raw const target;
    assert_eq!(target.assign(&source) as *const _, destination);
    assert_eq!((target.hasher().0, target.bucket_count()), (90, 1009));
    assert_eq!(target.allocator(), &arena);
    let stored = target.first().unwrap() as *const i32;
    assert_eq!(target.assign_self() as *const _, destination);
    assert_eq!(target.first().unwrap() as *const i32, stored);
    assert_eq!(target.move_assign(&mut source) as *const _, destination);
    assert_eq!((target.hasher().0, source.hasher().0), (7, 90));
    assert_eq!(source.first().unwrap() as *const i32, stored);
}

#[test]
fn ordered_empty_assignment_retains_bucket_selection() {
    let mut map = OrderedMap::<i32>::new(1009);
    map.add(3);
    let empty = OrderedMap::default();
    assert!(map.assign(&empty).is_empty());
    assert_eq!(map.bucket_count(), 1009);
    assert!(map.add_ref(&7));
}

#[test]
fn ordered_iterator_prefix_postfix_and_returning_assignment() {
    let mut map = OrderedMap::<i32>::default();
    map.add(3);
    map.add(7);
    let mut cursor = map.iter();
    let previous = cursor.post_advance();
    assert_eq!(previous.key(), Ok(&3));
    assert_eq!(cursor.value(), Ok(&7));
    let destination = &raw const cursor;
    assert_eq!(cursor.advance_returning() as *const _, destination);
    assert!(cursor.is_equal(&map.end()));
    assert_eq!(cursor.assign(&previous) as *const _, destination);
    assert_eq!(cursor.assign_self() as *const _, destination);
    assert!(cursor.is_equal(&map.iter()));
    cursor.advance();
    cursor.reset();
    assert_eq!(cursor.key(), Ok(&3));
}

#[test]
fn ordered_iterator_cross_map_identity_and_default_initialization() {
    let mut first = OrderedMap::<i32>::default();
    first.add(1);
    let mut second = OrderedMap::<i32>::default();
    second.add(1);
    assert!(!first.iter().is_equal(&second.iter()));
    assert!(first.end().is_equal(&second.end()));
    let mut detached = OrderedMapIter::default();
    detached.reset();
    assert!(!detached.more());
    detached.initialize(&first);
    assert!(detached.is_equal(&first.iter()));
}

#[derive(Debug)]
struct AllocationKey {
    arena: BaseAllocator,
    fail: bool,
}
impl Clone for AllocationKey {
    fn clone(&self) -> Self {
        assert!(
            self.arena.reset(false).is_err(),
            "expected node allocation before copying"
        );
        assert!(!self.fail, "injected key-copy failure");
        Self {
            arena: self.arena.clone(),
            fail: self.fail,
        }
    }
}
struct AllocationHash;
impl HashPolicy<AllocationKey> for AllocationHash {
    fn hash(&self, _: &AllocationKey) -> usize {
        1
    }
    fn equal(&self, _: &AllocationKey, _: &AllocationKey) -> bool {
        true
    }
}

#[test]
fn ordered_borrowed_key_copy_failure_does_not_publish_or_leak_node() {
    let arena = BaseAllocator::incremental(4096);
    let mut map = OrderedMap::with_allocator(AllocationHash, 1, arena.clone());
    let input = AllocationKey {
        arena: arena.clone(),
        fail: true,
    };
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.add_ref(&input)));
    assert!(failed.is_err());
    assert!(map.is_empty());
    assert!(arena.reset(false).is_ok());
    let input = AllocationKey {
        arena: arena.clone(),
        fail: false,
    };
    assert!(map.add_ref(&input));
    assert!(!map.add_ref(&input));
}

#[test]
fn ordered_backend_constructs_lazy_item_after_allocation_and_unwinds_failure() {
    use rustycad_tkernel::ncollection::OrderedDataMap;
    let arena = BaseAllocator::incremental(4096);
    let mut map = OrderedDataMap::<i32, i32>::with_allocator(Default::default(), 1, arena.clone());
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        map.emplace(1, || {
            assert!(arena.reset(false).is_err());
            panic!("injected item construction failure");
        });
    }));
    assert!(failed.is_err());
    assert!(map.is_empty());
    assert!(arena.reset(false).is_ok());
    assert!(map.emplace(1, || 10));
    assert_eq!(map.find(&1), Ok(&10));
}
