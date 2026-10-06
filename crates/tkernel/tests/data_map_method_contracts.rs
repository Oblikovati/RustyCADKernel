//! Missing DataMap forwarding overloads and native allocation/lookup order.
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap, HashPolicy};
use std::cell::RefCell;
use std::rc::Rc;

type Events = Rc<RefCell<String>>;
#[derive(Debug)]
struct CopyTrace(i32, char, Events);
impl Clone for CopyTrace {
    fn clone(&self) -> Self {
        self.2.borrow_mut().push(self.1);
        Self(self.0, self.1, Rc::clone(&self.2))
    }
}
struct TracePolicy(Events);
impl HashPolicy<CopyTrace> for TracePolicy {
    fn hash(&self, key: &CopyTrace) -> usize {
        self.0.borrow_mut().push('H');
        key.0 as usize
    }
    fn equal(&self, first: &CopyTrace, second: &CopyTrace) -> bool {
        self.0.borrow_mut().push('E');
        first.0 == second.0
    }
}

#[test]
fn borrowed_bound_and_try_operations_copy_only_on_native_paths() {
    let events = Rc::new(RefCell::new(String::new()));
    let key = CopyTrace(1, 'K', Rc::clone(&events));
    let value = CopyTrace(7, 'V', Rc::clone(&events));
    let mut map = DataMap::with_hasher(TracePolicy(Rc::clone(&events)), 1);
    let address = map.bound_ref(&key, &value) as *mut CopyTrace;
    assert_eq!(&*events.borrow(), "HVK");
    events.borrow_mut().clear();
    assert!(!map.try_bind_ref(&key, &value));
    assert_eq!(map.try_bound_ref(&key, &value) as *mut CopyTrace, address);
    assert_eq!(&*events.borrow(), "HEHE");
    events.borrow_mut().clear();
    assert_eq!(map.bound_ref(&key, &value) as *mut CopyTrace, address);
    assert_eq!(&*events.borrow(), "HEV");
}

#[test]
fn all_borrowed_emplacement_variants_preserve_lazy_construction() {
    let mut map = DataMap::<i32, String>::default();
    assert!(map.emplace_ref(&1, || "first".into()));
    assert!(!map.emplace_ref(&1, || "second".into()));
    assert_eq!(map.emplaced_ref(&1, || "third".into()), "third");
    assert!(!map.try_emplace_ref(&1, || panic!("existing key constructed")));
    assert_eq!(
        map.try_emplaced_ref(&1, || panic!("existing key constructed")),
        "third"
    );
    assert!(map.try_emplace_ref(&2, || "new".into()));
    assert_eq!(map.try_emplaced_ref(&3, || "another".into()), "another");
    assert_eq!(map.emplaced_ref(&4, || "fourth".into()), "fourth");
}

#[test]
fn borrowed_try_binding_misses_copy_value_before_key() {
    let events = Rc::new(RefCell::new(String::new()));
    let value = CopyTrace(7, 'V', Rc::clone(&events));
    let mut map = DataMap::with_hasher(TracePolicy(Rc::clone(&events)), 101);
    assert!(map.try_bind_ref(&CopyTrace(1, 'K', Rc::clone(&events)), &value));
    assert_eq!(&*events.borrow(), "HVK");
    events.borrow_mut().clear();
    assert_eq!(
        map.try_bound_ref(&CopyTrace(2, 'K', Rc::clone(&events)), &value)
            .0,
        7
    );
    assert_eq!(&*events.borrow(), "HVK");
}

#[test]
fn owned_emplacement_hashes_once_and_allocates_before_constructor() {
    let events = Rc::new(RefCell::new(String::new()));
    let arena = BaseAllocator::incremental(256);
    let mut map = DataMap::with_allocator(TracePolicy(Rc::clone(&events)), 101, arena.clone());
    assert!(map.emplace(CopyTrace(1, 'K', Rc::clone(&events)), || {
        assert!(arena.reset(false).is_err());
        events.borrow_mut().push('C');
        42
    }));
    assert_eq!(&*events.borrow(), "HC");
    drop(map);
    arena.reset(false).unwrap();
}

#[test]
fn emplacement_constructor_panic_releases_preallocated_node() {
    let arena = BaseAllocator::incremental(256);
    let mut map = DataMap::<i32, i32>::with_allocator(Default::default(), 101, arena.clone());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        map.emplace(1, || {
            assert!(arena.reset(false).is_err());
            panic!("injected value constructor failure");
        });
    }));
    assert!(panic.is_err());
    assert!(map.is_empty());
    arena.reset(false).unwrap();
}

#[test]
fn signed_bucket_overloads_validate_before_mutation() {
    type IntMap = DataMap<i32, i32>;
    assert!(IntMap::with_hasher_signed(Default::default(), -1).is_err());
    let arena = BaseAllocator::incremental(256);
    let mut map = IntMap::with_allocator_signed(Default::default(), 0, arena.clone()).unwrap();
    assert_eq!(map.bucket_count(), 0);
    assert_eq!(map.allocator(), &arena);
    map.bind(1, 42);
    assert!(map.resize_signed(-1).is_err());
    assert_eq!(map.find(&1), Ok(&42));
    assert_eq!(map.bucket_count(), 101);
    map.resize_signed(101).unwrap();
    assert_eq!(map.bucket_count(), 1009);
}

#[test]
fn readonly_cursor_copy_and_prefix_postfix_preserve_positions() {
    let mut map = DataMap::<i32, i32>::default();
    map.bind(1, 42);
    let mut cursor = map.cursor();
    let copy = cursor.clone();
    assert!(cursor.is_equal(&copy));
    assert!(cursor.advance_post().is_equal(&copy));
    assert!(!cursor.more());
    cursor.reset();
    let address = &mut cursor as *mut _;
    assert_eq!(cursor.advance() as *mut _, address);
    assert!(!cursor.more());
}

#[test]
fn mutable_cursor_default_comparison_and_postfix_snapshot() {
    use rustycad_tkernel::ncollection::DataMapIteratorMut;
    let mut empty = DataMapIteratorMut::<i32, i32>::default();
    let other = DataMapIteratorMut::default();
    assert!(empty.is_equal(&other));
    empty.reset();
    assert!(!empty.more());
    let mut map = DataMap::<i32, i32>::default();
    map.bind(1, 42);
    let mut cursor = map.cursor_mut();
    assert!(!cursor.is_equal(&empty));
    assert_eq!(cursor.advance_post().value(), Ok(&42));
    assert!(cursor.is_equal(&empty));
    cursor.reset();
    let address = &mut cursor as *mut _;
    assert_eq!(cursor.advance() as *mut _, address);
    assert!(!cursor.more());
}

#[test]
fn empty_retained_table_queries_do_not_invoke_user_hasher() {
    let events = Rc::new(RefCell::new(String::new()));
    let mut map = DataMap::with_hasher(TracePolicy(Rc::clone(&events)), 101);
    let key = CopyTrace(1, 'K', Rc::clone(&events));
    map.bind_ref(&key, &42);
    map.clear(false);
    events.borrow_mut().clear();
    assert!(!map.is_bound(&key));
    assert!(map.seek(&key).is_none());
    assert!(!map.unbind(&key));
    assert!(map.find(&key).is_err());
    assert!(events.borrow().is_empty());
}
