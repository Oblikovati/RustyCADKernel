#[test]
fn NCollection_ListTest__Assignment() {
    let mut original = list(&[10, 20, 30]);
    let mut copy = List::default();
    copy.assign(&original);
    equal(&copy, &original.iter().copied().collect::<Vec<_>>());
    *original.first_mut().unwrap() = 100;
    assert_eq!(original.first(), Some(&100));
    assert_eq!(copy.first(), Some(&10));
}
#[test]
fn NCollection_ListTest__AssignMethod() {
    let original = list(&[10, 20, 30]);
    let mut copy = list(&[40]);
    copy.assign(&original);
    equal(&copy, &[10, 20, 30]);
}
#[test]
fn NCollection_ListTest__AppendList() {
    let mut first = list(&[10, 20]);
    let mut second = list(&[30, 40]);
    first.append_list(&mut second);
    equal(&first, &[10, 20, 30, 40]);
    assert!(second.is_empty());
}
#[test]
fn NCollection_ListTest__PrependList() {
    let mut first = list(&[30, 40]);
    let mut second = list(&[10, 20]);
    first.prepend_list(&mut second);
    equal(&first, &[10, 20, 30, 40]);
    assert!(second.is_empty());
}
#[test]
fn NCollection_ListTest__InsertList() {
    let mut first = list(&[10, 40]);
    let mut second = list(&[20, 30]);
    let mut cursor = first.cursor();
    cursor.next(&first);
    first.insert_list_before(&mut second, &cursor).unwrap();
    equal(&first, &[10, 20, 30, 40]);
    assert!(second.is_empty());
}
#[test]
fn NCollection_ListTest__OCC25348_AssignDoesNotChangeAllocator() {
    let mut first = List::with_allocator(BaseAllocator::incremental(12 * 1024));
    for i in 0..10 {
        let mut second = List::with_allocator(BaseAllocator::incremental(12 * 1024));
        second.append(i);
        let before = first.allocator().clone();
        first.assign(&second);
        assert_eq!(first.allocator(), &before);
        assert_eq!(first.len(), 1);
        assert_eq!(first.first(), Some(&i));
    }
}
#[test]
fn NCollection_ListTest__InitializerListConstructor() {
    let list: List<i32> = [10, 20, 30, 40, 50].into_iter().collect();
    equal(&list, &[10, 20, 30, 40, 50]);
    assert_eq!(list.first(), Some(&10));
    assert_eq!(list.last(), Some(&50));
}
#[test]
fn NCollection_ListTest__InitializerListConstructorEmpty() {
    let list: List<i32> = [].into_iter().collect();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}
#[test]
fn NCollection_ListTest__Exchange() {
    let mut first = list(&[10, 20]);
    let mut second = list(&[30, 40, 50]);
    first.exchange(&mut second);
    equal(&first, &[30, 40, 50]);
    equal(&second, &[10, 20]);
}
#[test]
fn NCollection_ListTest__ExchangeWithEmpty() {
    let mut first = list(&[10, 20]);
    let mut second = List::default();
    first.exchange(&mut second);
    assert!(first.is_empty());
    equal(&second, &[10, 20]);
}
#[test]
fn NCollection_ListTest__MoveConstructor() {
    let mut first = list(&[10, 20, 30]);
    let second = List::move_from(&mut first);
    equal(&second, &[10, 20, 30]);
    assert!(first.is_empty());
    assert_eq!(first.len(), 0);
}
#[test]
fn NCollection_ListTest__MoveAssignment() {
    let mut first = list(&[10, 20]);
    let mut second = list(&[100]);
    second.move_assign(&mut first);
    equal(&second, &[10, 20]);
    assert!(first.is_empty());
}
#[test]
fn NCollection_ListTest__InitializerListWithCustomAllocator() {
    let allocator = BaseAllocator::incremental(12 * 1024);
    let list = List::from_iter_in([10, 20, 30], allocator.clone());
    equal(&list, &[10, 20, 30]);
    assert_eq!(list.allocator(), &allocator);
}
#[test]
fn NCollection_ListTest__ExchangeWithDifferentAllocators() {
    let a = BaseAllocator::incremental(12 * 1024);
    let b = BaseAllocator::incremental(12 * 1024);
    let mut first = List::from_iter_in([10, 20], a.clone());
    let mut second = List::from_iter_in([30, 40, 50], b.clone());
    first.exchange(&mut second);
    equal(&first, &[30, 40, 50]);
    equal(&second, &[10, 20]);
    assert_eq!(first.allocator(), &b);
    assert_eq!(second.allocator(), &a);
}
