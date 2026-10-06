//! Native DoubleMap ownership, signed overloads and cursor contracts.
use rustycad_tkernel::ncollection::{
    BaseAllocator, DefaultHashPolicy, DoubleMap, DoubleMapIterator, HashPolicy,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Shift(usize);
impl HashPolicy<i32> for Shift {
    fn hash(&self, key: &i32) -> usize {
        *key as usize + self.0
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}

#[test]
fn signed_constructor_and_resize_preserve_negative_exception_domain() {
    let failure = DoubleMap::<i32, i32>::new_int(-1).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    let mut map = DoubleMap::<i32, i32>::new_int(0).unwrap();
    assert_eq!(map.bucket_count(), 0);
    assert_eq!(
        map.resize_int(-1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(map.bucket_count(), 0);
    map.resize_int(0).unwrap();
    assert_eq!(map.bucket_count(), 101);
    let arena = BaseAllocator::incremental(128);
    let custom = DoubleMap::<i32, i32>::with_int_buckets(
        DefaultHashPolicy,
        DefaultHashPolicy,
        5,
        arena.clone(),
    )
    .unwrap();
    assert_eq!(custom.bucket_count(), 5);
    assert_eq!(custom.allocator(), &arena);
}

#[test]
fn move_constructor_keeps_node_addresses_and_source_configuration() {
    let arena = BaseAllocator::incremental(128);
    let mut source = DoubleMap::with_allocator(Shift(10), Shift(20), 500, arena.clone());
    source.bind(1, 10).unwrap();
    let pointer = source.seek1_ptr(&1);
    let moved = DoubleMap::move_from(&mut source);
    assert_eq!(moved.seek1_ptr(&1), pointer);
    assert_eq!(moved.allocator(), &arena);
    assert_eq!(source.allocator(), &arena);
    assert_eq!(source.bucket_count(), 500);
    assert!(source.is_empty());
    source.bind(2, 20).unwrap();
    assert_eq!(source.find2(&20), Ok(&2));
}

#[test]
fn move_assignment_exchanges_policies_allocators_and_retained_bucket_counts() {
    let old_arena = BaseAllocator::incremental(128);
    let new_arena = BaseAllocator::incremental(128);
    let mut destination = DoubleMap::with_allocator(Shift(1), Shift(2), 700, old_arena.clone());
    let mut source = DoubleMap::with_allocator(Shift(3), Shift(4), 500, new_arena.clone());
    destination.bind(9, 90).unwrap();
    source.bind(1, 10).unwrap();
    let pointer = source.seek1_ptr(&1);
    let address = std::ptr::from_mut(&mut destination);
    assert_eq!(
        std::ptr::from_mut(destination.move_assign(&mut source)),
        address
    );
    assert_eq!(destination.seek1_ptr(&1), pointer);
    assert_eq!(destination.allocator(), &new_arena);
    assert_eq!(source.allocator(), &old_arena);
    assert_eq!(source.bucket_count(), 700);
    assert!(source.is_empty());
    source.bind(2, 20).unwrap();
    assert_eq!(source.find1(&2), Ok(&20));
}

#[test]
fn custom_policy_move_is_injected_without_clone_or_default_requirements() {
    let mut source = DoubleMap::with_allocator(Shift(1), Shift(2), 10, BaseAllocator::common());
    source.bind(3, 30).unwrap();
    let take = |policy: &mut Shift| std::mem::replace(policy, Shift(19));
    let moved = DoubleMap::move_from_with(&mut source, take, take);
    assert_eq!(moved.find1(&3), Ok(&30));
    source.bind(4, 40).unwrap();
    assert_eq!(source.find2(&40), Ok(&4));
}

#[test]
fn assignments_return_self_preserve_allocator_and_accept_nonclone_policies() {
    let arena = BaseAllocator::incremental(128);
    let mut destination = DoubleMap::with_allocator(Shift(1), Shift(2), 700, arena.clone());
    let mut source = DoubleMap::with_allocator(Shift(3), Shift(4), 500, BaseAllocator::common());
    source.bind(1, 10).unwrap();
    let address = std::ptr::from_mut(&mut destination);
    assert_eq!(std::ptr::from_mut(destination.assign(&source)), address);
    assert_eq!(destination.find1(&1), Ok(&10));
    assert_eq!(destination.allocator(), &arena);
    let pointer = destination.seek1_ptr(&1);
    assert_eq!(std::ptr::from_mut(destination.assign_self()), address);
    assert_eq!(destination.seek1_ptr(&1), pointer);
    source.clear(false);
    destination.assign(&source);
    assert!(destination.is_empty());
    assert_eq!(destination.bucket_count(), 700);
}

#[derive(Debug)]
struct ObservedKey {
    value: i32,
    trace: Rc<RefCell<Vec<i32>>>,
}
impl Clone for ObservedKey {
    fn clone(&self) -> Self {
        self.trace.borrow_mut().push(self.value);
        Self {
            value: self.value,
            trace: self.trace.clone(),
        }
    }
}
#[derive(Clone)]
struct ObservedPolicy;
impl HashPolicy<ObservedKey> for ObservedPolicy {
    fn hash(&self, key: &ObservedKey) -> usize {
        key.value as usize
    }
    fn equal(&self, left: &ObservedKey, right: &ObservedKey) -> bool {
        left.value == right.value
    }
}
#[test]
fn assignment_preserves_native_local_then_node_copy_order() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut source =
        DoubleMap::with_allocator(ObservedPolicy, ObservedPolicy, 1, BaseAllocator::common());
    source
        .bind(
            ObservedKey {
                value: 1,
                trace: trace.clone(),
            },
            ObservedKey {
                value: 2,
                trace: trace.clone(),
            },
        )
        .unwrap();
    let mut destination =
        DoubleMap::with_allocator(ObservedPolicy, ObservedPolicy, 1, BaseAllocator::common());
    destination.assign(&source);
    assert_eq!(*trace.borrow(), [1, 2, 2, 1]);
    trace.borrow_mut().clear();
    destination.clone_from(&source);
    assert_eq!(*trace.borrow(), [1, 2, 2, 1]);
}

#[test]
fn native_seek_pointer_identity_survives_resize_and_reports_null_absence() {
    let mut map = DoubleMap::<i32, i32>::default();
    assert!(map.seek1_ptr(&1).is_null());
    assert!(map.seek2_ptr(&2).is_null());
    map.bind(1, 2).unwrap();
    let first = map.seek2_ptr(&2);
    let second = map.seek1_ptr(&1);
    map.resize(1000);
    assert_eq!(map.seek2_ptr(&2), first);
    assert_eq!(map.seek1_ptr(&1), second);
}

#[test]
fn cursor_copy_prefix_post_and_assignment_preserve_reset_source() {
    let mut map = DoubleMap::<i32, i32>::default();
    map.bind(1, 10).unwrap();
    map.bind(2, 20).unwrap();
    let mut cursor = DoubleMapIterator::new(&map);
    let previous = cursor.post_advance();
    assert_eq!(previous.value(), Ok(&10));
    assert_eq!(cursor.value_ptr(), map.seek1_ptr(&2));
    let mut copied = previous.clone();
    assert!(copied == previous && copied != cursor);
    let address = std::ptr::from_mut(&mut copied);
    assert_eq!(std::ptr::from_mut(copied.assign(&cursor)), address);
    assert_eq!(std::ptr::from_mut(copied.advance()), address);
    copied.advance();
    assert!(copied.value_ptr().is_null());
    copied.reset();
    assert!(copied == previous);
}

#[test]
fn borrowed_binding_copies_key2_then_key1_only_for_new_pairs() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let first = ObservedKey {
        value: 1,
        trace: trace.clone(),
    };
    let second = ObservedKey {
        value: 2,
        trace: trace.clone(),
    };
    let mut map =
        DoubleMap::with_allocator(ObservedPolicy, ObservedPolicy, 1, BaseAllocator::common());
    map.bind_ref(&first, &second).unwrap();
    assert_eq!(*trace.borrow(), [2, 1]);
    trace.borrow_mut().clear();
    assert!(!map.try_bind_ref(&first, &second));
    assert_eq!(
        map.bind_ref(&first, &second).unwrap_err().exception_type(),
        "Standard_MultiplyDefined"
    );
    assert!(trace.borrow().is_empty());
}
