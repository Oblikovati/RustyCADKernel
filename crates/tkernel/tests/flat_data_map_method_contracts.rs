//! Borrowed forwarding and iterator behavior for the pinned FlatDataMap class.
use rustycad_tkernel::ncollection::{FlatDataItems, FlatDataMap, FlatDataValues, HashPolicy};
use std::cell::RefCell;
use std::rc::Rc;

type Events = Rc<RefCell<String>>;
#[derive(Debug)]
struct Key {
    number: i32,
    events: Events,
    fail_copy: bool,
}
impl Clone for Key {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push('K');
        assert!(!self.fail_copy, "injected key copy failure");
        Self {
            number: self.number,
            events: Rc::clone(&self.events),
            fail_copy: false,
        }
    }
}
struct Policy(Events);
impl HashPolicy<Key> for Policy {
    fn hash(&self, key: &Key) -> usize {
        self.0.borrow_mut().push('H');
        key.number as usize
    }
    fn equal(&self, left: &Key, right: &Key) -> bool {
        self.0.borrow_mut().push('E');
        left.number == right.number
    }
}
fn key(number: i32, events: &Events) -> Key {
    Key {
        number,
        events: Rc::clone(events),
        fail_copy: false,
    }
}
fn value(events: &Events, number: i32) -> i32 {
    events.borrow_mut().push('C');
    number
}

#[test]
fn borrowed_emplace_hashes_then_copies_key_before_lookup_and_value_construction() {
    let events = Rc::new(RefCell::new(String::new()));
    let mut map = FlatDataMap::with_hasher(Policy(Rc::clone(&events)), 0);
    let lookup = key(1, &events);
    assert!(map.emplace_ref(&lookup, || value(&events, 42)));
    assert_eq!(&*events.borrow(), "HKC");
    events.borrow_mut().clear();
    assert!(!map.emplace_ref(&lookup, || value(&events, 43)));
    assert_eq!(&*events.borrow(), "HKEC");
    events.borrow_mut().clear();
    assert!(!map.try_emplace_ref(&lookup, || panic!("constructed on hit")));
    assert_eq!(&*events.borrow(), "HKE");
    events.borrow_mut().clear();
    assert_eq!(
        map.try_emplaced_ref(&lookup, || panic!("constructed on hit")),
        &43
    );
    assert_eq!(&*events.borrow(), "HKE");
}

#[test]
fn borrowed_reference_variants_return_original_key_value_after_displacement() {
    struct IntPolicy;
    impl HashPolicy<i32> for IntPolicy {
        fn hash(&self, key: &i32) -> usize {
            *key as usize
        }
        fn equal(&self, a: &i32, b: &i32) -> bool {
            a == b
        }
    }
    let mut map = FlatDataMap::with_hasher(IntPolicy, 0);
    map.bind(0, 10);
    map.bind(1, 20);
    assert_eq!(map.emplaced_ref(&8, || 80), &80);
    assert_eq!(map.try_emplaced_ref(&16, || 160), &160);
    assert!(map.try_emplace_ref(&24, || 240));
    assert_eq!(map.emplaced_ref(&8, || 81), &81);
    assert_eq!(map.find(&1), Ok(&20));
}

#[test]
fn borrowed_key_copy_panic_occurs_after_reserve_without_constructing_value() {
    let events = Rc::new(RefCell::new(String::new()));
    let mut map = FlatDataMap::with_hasher(Policy(Rc::clone(&events)), 0);
    let mut lookup = key(1, &events);
    lookup.fail_copy = true;
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        map.emplace_ref(&lookup, || value(&events, 42));
    }));
    assert!(failed.is_err());
    assert!(map.is_empty());
    assert_eq!(map.capacity(), 8);
    assert_eq!(&*events.borrow(), "HK");
}

#[test]
fn raw_queries_preserve_null_and_stored_value_identity() {
    let mut map = FlatDataMap::<i32, String>::default();
    assert!(map.seek_ptr(&1).is_null());
    assert!(map.change_seek_ptr(&1).is_null());
    map.bind(1, "first".into());
    let pointer = map.seek_ptr(&1);
    assert_eq!(pointer, map.change_seek_ptr(&1).cast_const());
    *map.change_find(&1).unwrap() = "replacement".into();
    assert_eq!(pointer, map.seek_ptr(&1));
    map.clear(false);
    assert!(map.seek_ptr(&1).is_null());
}

macro_rules! readonly_contract {
    ($name:ident, $type:ident, $method:ident) => {
        #[test]
        fn $name() {
            let mut map = FlatDataMap::<i32, i32>::default();
            map.bind(1, 42);
            let source = map.$method();
            let mut cursor = $type::default();
            let address = &mut cursor as *mut _;
            assert_eq!(cursor.assign_from(&source) as *mut _, address);
            assert_eq!(cursor.assign_self() as *mut _, address);
            assert!(cursor.is_equal(&source));
            assert_eq!(cursor.key(), Ok(&1));
            assert_eq!(cursor.value_ptr(), Ok(map.seek_ptr(&1)));
            assert!(cursor.advance_post().is_equal(&source));
            assert!(!cursor.more());
            cursor.assign_from(&source);
            assert_eq!(cursor.advance_returning() as *mut _, address);
            assert!(cursor.is_equal(&$type::default()));
        }
    };
}
readonly_contract!(
    legacy_pair_cursor_preserves_native_assignment_and_increment,
    FlatDataItems,
    items
);
readonly_contract!(
    const_value_iterator_preserves_native_assignment_and_increment,
    FlatDataValues,
    iter
);

#[test]
fn mutable_cursor_pointer_prefix_postfix_and_self_assignment() {
    let mut map = FlatDataMap::<i32, i32>::default();
    map.bind(1, 42);
    map.bind(2, 43);
    let mut cursor = map.cursor_mut();
    let first = *cursor.value().unwrap();
    assert_eq!(
        cursor.value_ptr().unwrap(),
        cursor.change_value_ptr().unwrap().cast_const()
    );
    let address = &mut cursor as *mut _;
    assert_eq!(cursor.assign_self() as *mut _, address);
    assert_eq!(cursor.advance_post().value(), Ok(&first));
    assert!(cursor.more());
    assert_eq!(cursor.advance_returning() as *mut _, address);
    assert!(!cursor.more());
    assert!(cursor.value_ptr().is_err());
    assert!(cursor.change_value_ptr().is_err());
}

#[test]
fn self_assignment_needs_no_key_value_or_policy_copy() {
    #[derive(Debug)]
    struct NonClone(i32);
    let events = Rc::new(RefCell::new(String::new()));
    let mut map = FlatDataMap::with_hasher(Policy(Rc::clone(&events)), 0);
    let lookup = key(1, &events);
    map.bind(key(1, &events), NonClone(42));
    let pointer = map.seek_ptr(&lookup);
    let address = &mut map as *mut _;
    events.borrow_mut().clear();
    assert_eq!(map.assign_self() as *mut _, address);
    assert!(events.borrow().is_empty());
    assert_eq!(map.seek_ptr(&lookup), pointer);
    assert_eq!(map.find(&lookup).unwrap().0, 42);
}
