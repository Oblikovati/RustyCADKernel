//! Supplemental OrderedDataMap methods and iterator identity contracts.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, OrderedDataCursor, OrderedDataMap};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct TraceCopy {
    number: i32,
    label: &'static str,
    events: Rc<RefCell<Vec<&'static str>>>,
}
impl Clone for TraceCopy {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push(self.label);
        Self {
            number: self.number,
            label: self.label,
            events: Rc::clone(&self.events),
        }
    }
    fn clone_from(&mut self, _: &Self) {
        panic!("OrderedDataMap must construct the item temporary");
    }
}
#[derive(Clone)]
struct TraceHash;
impl HashPolicy<TraceCopy> for TraceHash {
    fn hash(&self, key: &TraceCopy) -> usize {
        key.events.borrow_mut().push("hash");
        key.number as usize
    }
    fn equal(&self, first: &TraceCopy, second: &TraceCopy) -> bool {
        first.number == second.number
    }
}
fn trace(number: i32, label: &'static str, events: &Rc<RefCell<Vec<&'static str>>>) -> TraceCopy {
    TraceCopy {
        number,
        label,
        events: Rc::clone(events),
    }
}
struct NonClonePolicy(i32);
impl HashPolicy<i32> for NonClonePolicy {
    fn hash(&self, key: &i32) -> usize {
        (*key + self.0) as usize
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}

#[test]
fn ordered_borrowed_bind_hashes_then_copies_item_before_key() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let key = trace(1, "key", &events);
    let value = trace(10, "item", &events);
    let mut map = OrderedDataMap::with_hasher(TraceHash, 1);
    assert!(map.bind_ref(&key, &value));
    assert_eq!(*events.borrow(), ["hash", "item", "key"]);
    events.borrow_mut().clear();
    assert!(!map.bind_ref(&key, &value));
    assert_eq!(*events.borrow(), ["hash", "item"]);
}

#[test]
fn ordered_borrowed_bound_preserves_address_and_try_hit_skips_copy() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let key = trace(1, "key", &events);
    let value = trace(10, "item", &events);
    let mut map = OrderedDataMap::with_hasher(TraceHash, 1);
    let stored = map.bound_ref(&key, &value) as *mut TraceCopy;
    events.borrow_mut().clear();
    assert_eq!(map.bound_ref(&key, &value) as *mut TraceCopy, stored);
    assert_eq!(*events.borrow(), ["hash", "item"]);
    events.borrow_mut().clear();
    assert!(!map.try_bind_ref(&key, &value));
    assert_eq!(map.try_bound_ref(&key, &value) as *mut TraceCopy, stored);
    assert_eq!(*events.borrow(), ["hash", "hash"]);
    assert!(map.try_bind_ref(&trace(2, "key2", &events), &value));
    assert_eq!(
        map.try_bound_ref(&trace(3, "key3", &events), &value).number,
        10
    );
}

#[test]
fn ordered_assignment_and_clone_follow_native_per_binding_copy_order() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut source = OrderedDataMap::with_hasher(TraceHash, 1);
    source.bind(trace(1, "key1", &events), trace(10, "item1", &events));
    source.bind(trace(2, "key2", &events), trace(20, "item2", &events));
    events.borrow_mut().clear();
    let mut target = OrderedDataMap::with_hasher(TraceHash, 1);
    target.assign(&source);
    assert_eq!(
        *events.borrow(),
        ["hash", "item1", "key1", "hash", "item2", "key2"]
    );
    events.borrow_mut().clear();
    let copy = source.clone();
    assert_eq!(
        *events.borrow(),
        ["hash", "item1", "key1", "hash", "item2", "key2"]
    );
    assert_eq!(copy.first_value().unwrap().number, 10);
}

#[test]
fn ordered_returning_assignment_preserves_nonclone_policy_and_allocator() {
    let arena = BaseAllocator::incremental(4096);
    let mut target = OrderedDataMap::with_allocator(NonClonePolicy(90), 1009, arena.clone());
    target.bind(9, 90);
    let mut source = OrderedDataMap::with_hasher(NonClonePolicy(7), 1);
    source.bind(1, 10);
    source.bind(2, 20);
    let destination = &raw const target;
    assert_eq!(target.assign(&source) as *const _, destination);
    assert_eq!(target.hasher().0, 90);
    assert_eq!(target.allocator(), &arena);
    assert_eq!(target.bucket_count(), 1009);
    assert_eq!(
        target.items().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        [(1, 10), (2, 20)]
    );
    assert_eq!(target.assign_self() as *const _, destination);
    assert_eq!(target.move_assign(&mut source) as *const _, destination);
    assert_eq!((target.hasher().0, source.hasher().0), (7, 90));
}

#[test]
fn ordered_cursor_equality_compares_nodes_across_distinct_maps() {
    let mut first = OrderedDataMap::<i32, i32>::default();
    first.bind(1, 10);
    let mut second = OrderedDataMap::<i32, i32>::default();
    second.bind(1, 10);
    let mut left = first.cursor();
    let mut right = second.cursor();
    assert!(left != right);
    assert!(!left.is_equal(&right));
    assert!(left.value(&second).is_err());
    assert!(left.key(&second).is_err());
    assert!(left.change_value(&mut second).is_err());
    left.advance(&first);
    right.advance(&second);
    assert!(left == right);
    assert!(left == OrderedDataCursor::default());
}

#[test]
fn ordered_cursor_prefix_postfix_assignment_and_reset_preserve_native_state() {
    let mut map = OrderedDataMap::<i32, i32>::default();
    map.bind(1, 10);
    map.bind(2, 20);
    let mut cursor = map.cursor();
    let previous = cursor.post_advance(&map);
    assert_eq!(previous.key(&map), Ok(&1));
    assert_eq!(cursor.value(&map), Ok(&20));
    *cursor.change_value(&mut map).unwrap() = 21;
    let destination = &raw const cursor;
    assert_eq!(cursor.advance_returning(&map) as *const _, destination);
    assert!(!cursor.more());
    assert_eq!(cursor.assign(&previous) as *const _, destination);
    assert_eq!(cursor.assign_self() as *const _, destination);
    cursor.reset(&map);
    assert_eq!(cursor.key(&map), Ok(&1));
}

#[test]
fn ordered_cursor_reset_uses_current_head_and_default_remains_detached() {
    let mut map = OrderedDataMap::<i32, i32>::default();
    map.bind(1, 10);
    map.bind(2, 20);
    let mut cursor = map.cursor();
    assert!(map.unbind(&1));
    assert!(cursor.value(&map).is_err());
    cursor.reset(&map);
    assert_eq!(cursor.key(&map), Ok(&2));
    let mut detached = OrderedDataCursor::default();
    detached.reset(&map);
    assert!(!detached.more());
    detached.initialize(&map);
    assert!(detached == cursor);
}

#[derive(Debug)]
struct AllocationProbe {
    arena: BaseAllocator,
    fail: bool,
}
impl Clone for AllocationProbe {
    fn clone(&self) -> Self {
        assert!(
            self.arena.reset(false).is_err(),
            "expected allocation before item copying"
        );
        assert!(!self.fail, "injected item-copy failure");
        Self {
            arena: self.arena.clone(),
            fail: self.fail,
        }
    }
}

#[test]
fn ordered_borrowed_copy_allocates_first_and_unwinds_failed_node() {
    let arena = BaseAllocator::incremental(4096);
    let mut map = OrderedDataMap::<i32, AllocationProbe>::with_allocator(
        Default::default(),
        1,
        arena.clone(),
    );
    let item = AllocationProbe {
        arena: arena.clone(),
        fail: true,
    };
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| map.bind_ref(&1, &item)));
    assert!(failed.is_err());
    assert!(map.is_empty());
    assert!(arena.reset(false).is_ok());
    let item = AllocationProbe {
        arena: arena.clone(),
        fail: false,
    };
    assert!(map.bind_ref(&1, &item));
    assert_eq!(map.first(), Ok(&1));
}
