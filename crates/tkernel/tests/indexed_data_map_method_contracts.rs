//! Supplemental contracts for NCollection_IndexedDataMap's concrete overloads.
use rustycad_tkernel::ncollection::{BaseAllocator, DefaultHashPolicy, HashPolicy, IndexedDataMap};
use std::cell::RefCell;
use std::rc::Rc;

struct SaltPolicy(i32);
impl HashPolicy<i32> for SaltPolicy {
    fn hash(&self, key: &i32) -> usize {
        (*key + self.0) as usize
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}
#[derive(Debug)]
struct CopyTrace {
    number: i32,
    label: &'static str,
    events: Rc<RefCell<Vec<&'static str>>>,
}
impl Clone for CopyTrace {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push(self.label);
        Self {
            number: self.number,
            label: self.label,
            events: Rc::clone(&self.events),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.events.borrow_mut().push("assign");
        self.number = source.number;
    }
}
struct TracePolicy;
impl HashPolicy<CopyTrace> for TracePolicy {
    fn hash(&self, key: &CopyTrace) -> usize {
        key.number as usize
    }
    fn equal(&self, first: &CopyTrace, second: &CopyTrace) -> bool {
        first.number == second.number
    }
}
fn trace(number: i32, label: &'static str, events: &Rc<RefCell<Vec<&'static str>>>) -> CopyTrace {
    CopyTrace {
        number,
        label,
        events: Rc::clone(events),
    }
}

#[test]
fn indexed_data_assignment_retains_policy_allocator_buckets_and_returns_destination() {
    let arena = BaseAllocator::incremental(4096);
    let mut target = IndexedDataMap::with_allocator(SaltPolicy(90), 1009, arena.clone());
    target.add(99, 990);
    let mut source = IndexedDataMap::with_hasher(SaltPolicy(7), 1);
    source.add(3, 30);
    source.add(4, 40);
    let destination = &raw const target;
    assert_eq!(target.assign(&source) as *const _, destination);
    assert_eq!(target.hasher().0, 90);
    assert_eq!(target.allocator(), &arena);
    assert_eq!(target.bucket_count(), 1009);
    assert_eq!(
        target.items().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        [(3, 30), (4, 40)]
    );
    let empty = IndexedDataMap::with_hasher(SaltPolicy(0), 1);
    assert!(target.assign(&empty).is_empty());
    assert_eq!(target.bucket_count(), 1009);
}

#[test]
fn indexed_data_assignment_copies_item_before_key_for_each_index() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut source = IndexedDataMap::with_hasher(TracePolicy, 1);
    source.add(trace(1, "key1", &events), trace(10, "item1", &events));
    source.add(trace(2, "key2", &events), trace(20, "item2", &events));
    let mut target = IndexedDataMap::with_hasher(TracePolicy, 1);
    target.assign(&source);
    assert_eq!(*events.borrow(), ["item1", "key1", "item2", "key2"]);
    assert_eq!(target.find_from_index(2).unwrap().number, 20);
}

#[test]
fn indexed_data_borrowed_bindings_assign_only_item_and_preserve_identity() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let key = trace(1, "key", &events);
    let value = trace(10, "item", &events);
    let mut map = IndexedDataMap::with_hasher(TracePolicy, 1);
    assert!(map.bind_ref(&key, &value));
    assert_eq!(*events.borrow(), ["item", "key"]);
    let stored = map.find_from_index(1).unwrap() as *const CopyTrace;
    let replacement = trace(20, "replacement", &events);
    events.borrow_mut().clear();
    assert!(!map.bind_ref(&key, &replacement));
    assert_eq!(*events.borrow(), ["assign"]);
    events.borrow_mut().clear();
    assert_eq!(
        map.bound_ref(&key, &value) as *mut CopyTrace as *const CopyTrace,
        stored
    );
    assert_eq!(*events.borrow(), ["assign"]);
    assert_eq!(map.find_from_index(1).unwrap().number, 10);
}

#[test]
fn indexed_data_borrowed_try_bindings_skip_copies_on_hit() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let key = trace(1, "key", &events);
    let value = trace(10, "item", &events);
    let mut map = IndexedDataMap::with_hasher(TracePolicy, 1);
    assert!(map.try_bind_ref(&key, &value));
    events.borrow_mut().clear();
    assert!(!map.try_bind_ref(&key, &value));
    assert_eq!(map.try_bound_ref(&key, &value).number, 10);
    assert!(events.borrow().is_empty());
    let absent = trace(2, "key2", &events);
    assert_eq!(map.try_bound_ref(&absent, &value).number, 10);
    assert_eq!(*events.borrow(), ["item", "key2"]);
    assert_eq!(map.bound_ref(&trace(3, "key3", &events), &value).number, 10);
    assert_eq!(map.len(), 3);
}

#[test]
fn indexed_data_move_and_self_assignment_preserve_addresses() {
    let arena = BaseAllocator::incremental(4096);
    let mut source = IndexedDataMap::with_allocator(SaltPolicy(7), 1, arena.clone());
    source.add(3, 30);
    let mut target = IndexedDataMap::with_hasher(SaltPolicy(90), 1009);
    target.add(99, 990);
    let original = source.find_from_index(1).unwrap() as *const i32;
    let replaced = target.find_from_index(1).unwrap() as *const i32;
    let destination = &raw const target;
    assert_eq!(target.move_from(&mut source) as *const _, destination);
    assert_eq!(target.find_from_index(1).unwrap() as *const i32, original);
    assert_eq!(source.find_from_index(1).unwrap() as *const i32, replaced);
    assert_eq!((target.hasher().0, source.hasher().0), (7, 90));
    assert_eq!(target.allocator(), &arena);
    assert_eq!(target.assign_self() as *const _, destination);
    assert_eq!(target.find_from_index(1).unwrap() as *const i32, original);
}

#[test]
fn indexed_data_signed_counts_reject_negative_constructor_and_resize() {
    let invalid = IndexedDataMap::<i32, i32>::with_int_buckets(
        DefaultHashPolicy,
        -7,
        BaseAllocator::common(),
    );
    assert_eq!(
        invalid.err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    let mut map =
        IndexedDataMap::<i32, i32>::with_int_buckets(DefaultHashPolicy, 0, BaseAllocator::common())
            .unwrap();
    assert_eq!(
        map.resize_int(-7).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(map.bucket_count(), 0);
    map.resize_int(0).unwrap();
    assert_eq!(map.bucket_count(), 101);
}

#[test]
fn indexed_data_signed_index_overloads_validate_before_mutation() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    map.add(10, 100);
    for index in [-7, 0, 2] {
        assert!(map.find_key_int(index).is_err());
        assert!(map.find_from_index_int(index).is_err());
        assert!(map.change_from_index_int(index).is_err());
        assert!(map.substitute_int(index, 20, 200).is_err());
        assert!(map.swap_int(index, 1).is_err());
        assert!(map.swap_int(1, index).is_err());
        assert!(map.remove_from_index_int(index).is_err());
    }
    assert_eq!(map.find_key_int(1), Ok(&10));
    assert_eq!(map.find_from_index_int(1), Ok(&100));
    *map.change_from_index_int(1).unwrap() = 101;
    assert_eq!(map.find_from_index_int(1), Ok(&101));
}

#[test]
fn indexed_data_signed_mutations_keep_keys_paired_with_values() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    map.add(10, 100);
    map.add(20, 200);
    map.add(30, 300);
    map.substitute_int(1, 11, 110).unwrap();
    assert_eq!(
        map.substitute_int(1, 20, 201).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    map.swap_int(1, 3).unwrap();
    map.remove_from_index_int(1).unwrap();
    assert_eq!(
        map.items().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        [(11, 110), (20, 200)]
    );
}

struct FailingCopy(i32);
impl Clone for FailingCopy {
    fn clone(&self) -> Self {
        assert_ne!(self.0, 20, "injected item copy failure");
        Self(self.0)
    }
}

#[test]
fn indexed_data_assignment_copy_panic_keeps_completed_prefix_consistent() {
    let mut source = IndexedDataMap::<i32, FailingCopy>::default();
    source.add(1, FailingCopy(10));
    source.add(2, FailingCopy(20));
    let mut target = IndexedDataMap::default();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        target.assign(&source);
    }));
    assert!(failure.is_err());
    assert_eq!(target.len(), 1);
    assert_eq!(target.find_key(1), Ok(&1));
    assert_eq!(target.find_from_index(1).unwrap().0, 10);
    assert_eq!(target.find_index(&2), 0);
    assert_eq!(target.add(3, FailingCopy(30)), 2);
}
