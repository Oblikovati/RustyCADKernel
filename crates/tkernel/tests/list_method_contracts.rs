//! Method-level contracts observed against NCollection_List / NCollection_BaseList.
use rustycad_tkernel::ncollection::{BaseAllocator, List, ListCursor};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn assignment_returns_destination_and_preserves_allocator_rules() {
    let arena = BaseAllocator::incremental(128);
    let mut destination = List::from_iter_in([9], arena.clone());
    let mut source: List<i32> = [1, 2].into_iter().collect();
    let address = std::ptr::from_mut(&mut destination);
    assert_eq!(std::ptr::from_mut(destination.assign(&source)), address);
    assert_eq!(destination.allocator(), &arena);
    let node = destination.cursor().value_ptr(&destination);
    assert_eq!(std::ptr::from_mut(destination.assign_self()), address);
    assert_eq!(destination.cursor().value_ptr(&destination), node);
    assert_eq!(
        std::ptr::from_mut(destination.move_assign(&mut source)),
        address
    );
    assert_eq!(destination.allocator(), source.allocator());
    assert_eq!(source.size(), 0);
    assert_eq!(destination.size(), 2);
}

#[test]
fn checked_endpoint_access_matches_native_exception_types() {
    let mut list = List::<i32>::default();
    assert_eq!(
        list.first_checked().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        list.last_checked().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        list.first_mut_checked().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        list.last_mut_checked().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    list.append(10);
    list.append(20);
    *list.first_mut_checked().unwrap() += 1;
    *list.last_mut_checked().unwrap() += 2;
    assert_eq!(list.first_checked().unwrap(), &11);
    assert_eq!(list.last_checked().unwrap(), &22);
}

#[test]
fn native_cursor_construction_assignment_and_increment_keep_node_identity() {
    let list: List<i32> = [10, 20].into_iter().collect();
    let mut cursor = ListCursor::from_iterator(&list.iter());
    let previous = cursor.post_advance(&list);
    assert_eq!(previous.value(&list), Some(&10));
    assert_eq!(cursor.value(&list), Some(&20));
    assert!(previous.is_equal(&ListCursor::new(&list)));
    cursor.advance(&list).advance(&list);
    assert!(!cursor.more());
    cursor.initialize(&list);
    assert_eq!(
        cursor.value_ptr(&list),
        Some(list.first().unwrap() as *const i32)
    );
    let address = std::ptr::from_mut(&mut cursor);
    assert_eq!(
        std::ptr::from_mut(cursor.assign(&ListCursor::default())),
        address
    );
    assert_eq!(
        std::ptr::from_mut(cursor.assign_iterator(&list.iter())),
        address
    );
    assert_eq!(cursor.value(&list), Some(&10));
}

#[test]
fn const_iterator_copy_and_default_preserve_cursor_conversion() {
    struct NoClone;
    let list: List<NoClone> = [NoClone].into_iter().collect();
    let first = list.iter();
    let mut copy = first;
    assert!(first == copy);
    assert!(copy.next().is_some());
    assert!(first != copy);
    let end = Default::default();
    assert!(copy == end);
    assert_eq!(ListCursor::from_iterator(&end), ListCursor::default());
}

#[test]
fn clone_from_uses_native_assignment_allocator_policy() {
    let arena = BaseAllocator::incremental(128);
    let mut destination = List::from_iter_in([9], arena.clone());
    let source: List<i32> = [1, 2].into_iter().collect();
    destination.clone_from(&source);
    assert_eq!(destination.allocator(), &arena);
    assert_eq!(destination.iter().copied().collect::<Vec<_>>(), [1, 2]);
    assert_eq!(source.size(), 2);
}

struct Searchable(i32);
impl PartialEq<i32> for Searchable {
    fn eq(&self, value: &i32) -> bool {
        self.0 == *value
    }
}
#[test]
fn heterogeneous_search_and_remove_require_only_requested_equality() {
    let mut list: List<Searchable> = [Searchable(1), Searchable(2), Searchable(2)]
        .into_iter()
        .collect();
    assert!(list.contains(&2));
    assert!(list.remove_value(&2));
    assert_eq!(list.size(), 2);
    assert_eq!(list.last().unwrap().0, 2);
    assert!(!list.contains(&3));
    assert!(!list.remove_value(&3));
}

#[test]
fn cross_allocator_insert_after_tail_raises_without_consuming_source() {
    let mut list: List<i32> = [1].into_iter().collect();
    let mut source = List::from_iter_in([2], BaseAllocator::incremental(128));
    let error = list
        .insert_list_after(&mut source, &list.cursor())
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
    assert_eq!(list.size(), 1);
    assert_eq!(source.size(), 1);
    source.clear();
    list.insert_list_after(&mut source, &list.cursor()).unwrap();
}

#[test]
fn invalid_emplacement_constructs_before_native_position_validation() {
    let calls = Cell::new(0);
    let construct = || {
        calls.set(calls.get() + 1);
        42
    };
    let mut list = List::default();
    assert!(
        list.emplace_before(&ListCursor::default(), construct)
            .is_err()
    );
    assert!(
        list.emplace_after(&ListCursor::default(), construct)
            .is_err()
    );
    assert_eq!(calls.get(), 2);
    assert_eq!(list.size(), 0);
}

struct CopyFailure(i32);
impl Clone for CopyFailure {
    fn clone(&self) -> Self {
        assert_ne!(self.0, 2, "copy 2");
        Self(self.0)
    }
}
#[test]
fn cross_allocator_append_commits_each_copy_before_later_failure() {
    let mut destination = List::default();
    let mut source = List::from_iter_in(
        [CopyFailure(1), CopyFailure(2)],
        BaseAllocator::incremental(128),
    );
    let outcome = catch_unwind(AssertUnwindSafe(|| destination.append_list(&mut source)));
    assert!(outcome.is_err());
    assert_eq!(destination.size(), 1);
    assert_eq!(destination.first().unwrap().0, 1);
    assert_eq!(source.size(), 2);
}

#[test]
fn cross_allocator_insert_commits_each_copy_before_later_failure() {
    let mut destination: List<CopyFailure> = [CopyFailure(9)].into_iter().collect();
    let mut source = List::from_iter_in(
        [CopyFailure(1), CopyFailure(2)],
        BaseAllocator::incremental(128),
    );
    let cursor = destination.cursor();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        destination.insert_list_before(&mut source, &cursor)
    }));
    assert!(outcome.is_err());
    assert_eq!(
        destination.iter().map(|item| item.0).collect::<Vec<_>>(),
        [1, 9]
    );
    assert_eq!(source.size(), 2);
}

#[test]
fn foreign_prepend_to_empty_destination_preserves_native_failure() {
    let mut destination = List::<i32>::default();
    let mut source = List::from_iter_in([1, 2], BaseAllocator::incremental(128));
    let failure = destination.prepend_list_checked(&mut source).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
    assert_eq!(destination.size(), 0);
    assert_eq!(source.size(), 2);
    destination.append(3);
    destination.prepend_list_checked(&mut source).unwrap();
    assert_eq!(destination.iter().copied().collect::<Vec<_>>(), [1, 2, 3]);
    assert!(source.is_empty());
}
