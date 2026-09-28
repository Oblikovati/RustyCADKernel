//! Ownership boundaries not fully exercised by upstream's common-allocator tests.
use rustycad_tkernel::ncollection::{BaseAllocator, List, OccAllocator};
#[test]
fn splices_preserve_allocator_identity_and_release_original_nodes() {
    let a = BaseAllocator::incremental(128);
    let b = BaseAllocator::incremental(128);
    let mut first = List::from_iter_in([1, 4], a.clone());
    let mut second = List::from_iter_in([2, 3], b.clone());
    let mut cursor = first.cursor();
    cursor.next(&first);
    first.insert_list_before(&mut second, &cursor).unwrap();
    assert_eq!(first.iter().copied().collect::<Vec<_>>(), [1, 2, 3, 4]);
    assert_eq!(cursor.value(&first), Some(&4));
    assert_eq!(second.allocator(), &b);
    assert!(second.is_empty());
    b.reset(true).unwrap();
    assert!(a.reset(true).is_err());
    first.clear();
    a.reset(true).unwrap();
}
#[test]
fn linked_nodes_retain_addresses_across_reverse_and_splice() {
    let mut first: List<i32> = [1, 2].into_iter().collect();
    let cursor = first.cursor();
    let original = cursor.value(&first).unwrap() as *const i32;
    first.reverse();
    let mut destination: List<i32> = [3].into_iter().collect();
    destination.prepend_list(&mut first);
    assert_eq!(cursor.value(&destination).unwrap() as *const i32, original);
    assert_eq!(destination.iter().copied().collect::<Vec<_>>(), [2, 1, 3]);
    assert!(first.is_empty());
}
#[test]
fn list_clone_and_allocator_vector_mutation_use_real_storage() {
    let allocator = OccAllocator::<i32>::default();
    let mut vector = allocator.clone().vector();
    vector.push(7);
    vector[0] = 8;
    assert_eq!(vector[0], 8);
    let mut list = allocator.list();
    assert!(list.is_empty());
    list.push_back(7);
    let mut clone = list.clone();
    *clone.first_mut().unwrap() = 8;
    assert_eq!(list.front(), Some(&7));
    assert_eq!(clone.front(), Some(&8));
}
