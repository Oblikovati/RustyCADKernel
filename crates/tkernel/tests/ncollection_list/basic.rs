#[test]
fn NCollection_ListTest__DefaultConstructor() {
    let list = List::<i32>::default();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert_eq!(list.extent(), 0);
}
#[test]
fn NCollection_ListTest__Append() {
    let mut list = List::default();
    assert_eq!(*list.append(10), 10);
    assert_eq!(*list.append(20), 20);
    assert_eq!(*list.append(30), 30);
    assert!(!list.is_empty());
    assert_eq!(list.len(), 3);
    assert_eq!(list.first(), Some(&10));
    assert_eq!(list.last(), Some(&30));
}
#[test]
fn NCollection_ListTest__Prepend() {
    let mut list = List::default();
    assert_eq!(*list.prepend(30), 30);
    assert_eq!(*list.prepend(20), 20);
    assert_eq!(*list.prepend(10), 10);
    assert!(!list.is_empty());
    assert_eq!(list.len(), 3);
    assert_eq!(list.first(), Some(&10));
    assert_eq!(list.last(), Some(&30));
}
#[test]
fn NCollection_ListTest__IteratorAccess() {
    let list = list(&[10, 20, 30]);
    let mut cursor = list.cursor();
    let mut count = 0;
    while cursor.more() {
        assert_eq!(cursor.value(&list), Some(&[10, 20, 30][count]));
        cursor.next(&list);
        count += 1;
    }
    assert_eq!(count, 3);
}
#[test]
fn NCollection_ListTest__STLIterators() {
    let list = list(&[10, 20, 30]);
    for (i, value) in list.iter().enumerate() {
        assert_eq!(*value, [10, 20, 30][i]);
    }
    assert_eq!(list.iter().count(), 3);
    for (i, value) in (&list).into_iter().enumerate() {
        assert_eq!(*value, [10, 20, 30][i]);
    }
    let constant = &list;
    assert!(list.iter() == constant.iter());
    let (mut mutable_end, mut const_end) = (list.iter(), constant.iter());
    mutable_end.by_ref().for_each(drop);
    const_end.by_ref().for_each(drop);
    assert!(mutable_end == const_end);
    assert!(list.iter().nth(3).is_none());
    assert!(constant.iter().nth(3).is_none());
    assert_eq!(constant.iter().filter(|&&value| value == 20).count(), 1);
    assert!(constant.iter().any(|&value| value == 30));
}
#[test]
fn NCollection_ListTest__RemoveFirst() {
    let mut list = list(&[10, 20, 30]);
    list.remove_first().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.first(), Some(&20));
    list.remove_first().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list.first(), Some(&30));
    list.remove_first().unwrap();
    assert!(list.is_empty());
}
#[test]
fn NCollection_ListTest__Remove() {
    let mut list = list(&[10, 20, 30]);
    let mut cursor = list.cursor();
    cursor.next(&list);
    list.remove(&mut cursor).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.first(), Some(&10));
    assert_eq!(list.last(), Some(&30));
    assert_eq!(cursor.value(&list), Some(&30));
}
#[test]
fn NCollection_ListTest__RemoveByValue() {
    let mut list = list(&[10, 20, 10, 30]);
    assert!(list.remove_value(&10));
    assert_eq!(list.len(), 3);
    assert_eq!(list.first(), Some(&20));
    assert!(!list.remove_value(&50));
    assert_eq!(list.len(), 3);
    assert!(list.remove_value(&10));
    equal(&list, &[20, 30]);
}
#[test]
fn NCollection_ListTest__Clear() {
    let mut list = list(&[10, 20, 30]);
    list.clear();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}
#[test]
fn NCollection_ListTest__InsertBefore() {
    let mut list = list(&[10, 30]);
    let mut cursor = list.cursor();
    cursor.next(&list);
    assert_eq!(*list.insert_before(20, &cursor).unwrap(), 20);
    equal(&list, &[10, 20, 30]);
}
#[test]
fn NCollection_ListTest__InsertAfter() {
    let mut list = list(&[10, 30]);
    let cursor = list.cursor();
    assert_eq!(*list.insert_after(20, &cursor).unwrap(), 20);
    equal(&list, &[10, 20, 30]);
}
#[test]
fn NCollection_ListTest__Reverse() {
    let mut list = list(&[10, 20, 30]);
    list.reverse();
    assert_eq!(list.first(), Some(&30));
    assert_eq!(list.last(), Some(&10));
    equal(&list, &[30, 20, 10]);
}
#[test]
fn NCollection_ListTest__STLAlgorithmCompatibility_MinMax() {
    let values = random_values();
    let list = list(&values);
    assert_eq!(list.iter().min(), values.iter().min());
    assert_eq!(list.iter().max(), values.iter().max());
}
#[test]
fn NCollection_ListTest__STLAlgorithmCompatibility_Replace() {
    let mut values = random_values();
    let mut list = list(&values);
    let target = *values.last().unwrap();
    for value in list.iter_mut() {
        if *value == target {
            *value = -1;
        }
    }
    for value in &mut values {
        if *value == target {
            *value = -1;
        }
    }
    equal(&list, &values);
}
#[test]
fn NCollection_ListTest__ConstIteratorFromConstList() {
    let list = list(&[10, 20, 30]);
    let constant = &list;
    let mut count = 0;
    for value in constant.iter() {
        assert_eq!(*value, [10, 20, 30][count]);
        count += 1;
    }
    assert_eq!(count, 3);
    count = 0;
    for value in constant {
        assert_eq!(*value, [10, 20, 30][count]);
        count += 1;
    }
    assert_eq!(count, 3);
}
