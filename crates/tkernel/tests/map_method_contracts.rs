//! Class-level Map methods and native forwarding/assignment behavior.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, Map};
use std::cell::RefCell;
use std::rc::Rc;

fn integer_keys(values: &[i32]) -> Map<i32> {
    let mut result = Map::default();
    for &value in values {
        result.add(value);
    }
    result
}
fn sorted_keys(map: &Map<i32>) -> Vec<i32> {
    let mut keys: Vec<i32> = map.iter().copied().collect();
    keys.sort_unstable();
    keys
}

#[test]
fn legacy_queries_compare_keys_and_containment() {
    let a = integer_keys(&[1, 2]);
    let equal = integer_keys(&[2, 1]);
    let subset = integer_keys(&[1]);
    assert!(a.is_equal(&equal));
    assert!(!a.is_equal(&subset));
    assert!(a.contains_map(&subset));
    assert!(!subset.contains_map(&a));
    assert!(a.has_intersection(&subset));
    assert!(!a.has_intersection(&integer_keys(&[3])));
}

#[test]
fn legacy_binary_construction_operations_match_native_sets() {
    let a = integer_keys(&[1, 2]);
    let b = integer_keys(&[2, 3]);
    let mut result = integer_keys(&[99]);
    result.union(&a, &b);
    assert_eq!(sorted_keys(&result), [1, 2, 3]);
    result.intersection(&a, &b);
    assert_eq!(sorted_keys(&result), [2]);
    result.subtraction(&a, &b);
    assert_eq!(sorted_keys(&result), [1]);
    result.difference(&a, &b);
    assert_eq!(sorted_keys(&result), [1, 3]);
    result.difference(&a, &a);
    assert!(result.is_empty());
}

#[test]
fn legacy_mutating_operations_return_native_size_change_flags() {
    let b = integer_keys(&[2, 3]);
    let mut a = integer_keys(&[1, 2]);
    assert!(a.unite(&b));
    assert!(!a.unite(&b));
    assert!(a.intersect(&b));
    assert!(!a.intersect(&b));
    assert_eq!(sorted_keys(&a), [2, 3]);
    assert!(a.subtract(&b));
    assert!(!a.subtract(&b));
    assert!(a.differ(&b));
    assert!(a.differ(&b));
    assert!(a.is_empty());
}

#[test]
fn differ_false_can_still_change_all_keys() {
    let mut a = integer_keys(&[1, 2]);
    assert!(!a.differ(&integer_keys(&[2, 3])));
    assert_eq!(sorted_keys(&a), [1, 3]);
}

struct FixedPolicy(i32);
impl HashPolicy<i32> for FixedPolicy {
    fn hash(&self, key: &i32) -> usize {
        (*key + self.0) as usize
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}

#[test]
fn assignment_returns_destination_and_retains_nonclone_policy_and_allocator() {
    let arena = BaseAllocator::incremental(256);
    let mut source = Map::with_hasher(FixedPolicy(11), 101);
    let mut destination = Map::with_allocator(FixedPolicy(17), 1009, arena.clone());
    source.add(1);
    destination.add(8);
    let address = &mut destination as *mut _;
    assert_eq!(destination.assign_returning(&source) as *mut _, address);
    assert_eq!(destination.hasher().0, 17);
    assert_eq!(destination.allocator(), &arena);
    assert_eq!(destination.bucket_count(), 1009);
    assert!(destination.contains(&1) && !destination.contains(&8));
    destination.assign(&source);
    assert_eq!(destination.assign_self() as *mut _, address);
}

#[test]
fn move_assignment_swaps_nodes_policies_and_allocators() {
    let arena = BaseAllocator::incremental(256);
    let mut source = Map::with_allocator(FixedPolicy(11), 101, arena.clone());
    let mut destination = Map::with_hasher(FixedPolicy(17), 1009);
    let source_node = source.added(1) as *const i32;
    let destination_node = destination.added(8) as *const i32;
    let address = &mut destination as *mut _;
    assert_eq!(
        destination.move_from_returning(&mut source) as *mut _,
        address
    );
    assert_eq!(
        destination.contained(&1).unwrap() as *const i32,
        source_node
    );
    assert_eq!(
        source.contained(&8).unwrap() as *const i32,
        destination_node
    );
    assert_eq!((destination.hasher().0, source.hasher().0), (11, 17));
    assert_eq!(destination.allocator(), &arena);
}

#[test]
fn signed_overloads_reject_negatives_without_mutation() {
    assert!(Map::<i32>::with_hasher_signed(Default::default(), -1).is_err());
    let arena = BaseAllocator::incremental(256);
    let mut map = Map::<i32>::with_allocator_signed(Default::default(), 0, arena.clone()).unwrap();
    assert_eq!(map.bucket_count(), 0);
    assert_eq!(map.allocator(), &arena);
    map.add(42);
    assert!(map.resize_signed(-1).is_err());
    assert!(map.contains(&42));
    map.resize_signed(101).unwrap();
    assert_eq!(map.bucket_count(), 1009);
}

#[test]
fn emplacement_closures_run_on_hits_and_keep_or_replace_native_key() {
    let mut map = Map::<i32>::default();
    assert!(map.emplace_with(|| 42));
    let pointer = map.contained(&42).unwrap() as *const i32;
    assert!(!map.emplace_with(|| 42));
    assert_eq!(map.emplaced_with(|| 42) as *const i32, pointer);
    assert!(!map.try_emplace_with(|| 42));
    assert_eq!(map.try_emplaced_with(|| 42) as *const i32, pointer);
    assert!(map.try_emplace_with(|| 43));
    assert_eq!(map.try_emplaced_with(|| 44), &44);
    assert_eq!(map.emplaced_with(|| 45), &45);
}

#[test]
fn throwing_emplacement_constructor_keeps_native_prior_resize() {
    let mut map = Map::<i32>::new(0);
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        map.emplace_with(|| panic!("injected key constructor failure"));
    }));
    assert!(failed.is_err());
    assert!(map.is_empty());
    assert_eq!(map.bucket_count(), 101);
}

#[test]
fn cursor_clone_and_native_increment_preserve_identity() {
    let map = integer_keys(&[1]);
    let mut cursor = map.cursor();
    let copy = cursor.clone();
    assert!(cursor.advance_post().is_equal(&copy));
    assert!(!cursor.more());
    cursor.reset();
    let address = &mut cursor as *mut _;
    assert_eq!(cursor.advance() as *mut _, address);
    assert!(!cursor.more());
}

#[derive(Debug)]
struct TraceKey(i32, Rc<RefCell<String>>);
impl Clone for TraceKey {
    fn clone(&self) -> Self {
        self.1.borrow_mut().push('K');
        Self(self.0, Rc::clone(&self.1))
    }
}
#[derive(Clone)]
struct TracePolicy(Rc<RefCell<String>>);
impl HashPolicy<TraceKey> for TracePolicy {
    fn hash(&self, key: &TraceKey) -> usize {
        self.0.borrow_mut().push('H');
        key.0 as usize
    }
    fn equal(&self, left: &TraceKey, right: &TraceKey) -> bool {
        self.0.borrow_mut().push('E');
        left.0 == right.0
    }
}

#[test]
fn emplacement_constructs_lookup_key_even_on_try_hit() {
    let events = Rc::new(RefCell::new(String::new()));
    let key = TraceKey(1, Rc::clone(&events));
    let mut map = Map::with_hasher(TracePolicy(Rc::clone(&events)), 101);
    map.add_ref(&key);
    events.borrow_mut().clear();
    assert!(!map.try_emplace_with(|| key.clone()));
    assert_eq!(&*events.borrow(), "KHE");
}

#[test]
fn union_and_unite_copy_only_missing_keys_after_lookup() {
    let events = Rc::new(RefCell::new(String::new()));
    let policy = TracePolicy(Rc::clone(&events));
    let mut left = Map::with_hasher(policy.clone(), 101);
    let mut right = Map::with_hasher(policy.clone(), 101);
    left.add(TraceKey(1, Rc::clone(&events)));
    right.add(TraceKey(1, Rc::clone(&events)));
    let mut result = Map::with_hasher(policy, 101);
    events.borrow_mut().clear();
    result.union(&left, &right);
    assert_eq!(&*events.borrow(), "HKHE");
    events.borrow_mut().clear();
    assert!(!result.unite(&right));
    assert_eq!(&*events.borrow(), "HE");
}

#[test]
fn set_construction_does_not_require_cloning_hasher() {
    let mut left = Map::with_hasher(FixedPolicy(1), 101);
    let mut right = Map::with_hasher(FixedPolicy(2), 101);
    let mut result = Map::with_hasher(FixedPolicy(3), 101);
    left.add(1);
    right.add(2);
    result.union(&left, &right);
    assert_eq!(result.len(), 2);
    result.intersection(&left, &right);
    assert!(result.is_empty());
    result.subtraction(&left, &right);
    assert!(result.contains(&1));
}
