fn insert_case(original: &[i32], index: i32, after: bool, expected: &[i32]) {
    let mut array = values(original);
    if after {
        array.insert_after(index, 99).unwrap();
    } else {
        array.insert_before(index, 99).unwrap();
    }
    equal(&array, expected);
}
#[test]
fn NCollection_DynamicArrayTest__InsertAfter_Middle() {
    insert_case(&[10, 20, 30, 40], 1, true, &[10, 20, 99, 30, 40]);
}
#[test]
fn NCollection_DynamicArrayTest__InsertAfter_First() {
    insert_case(&[10, 20, 30], 0, true, &[10, 99, 20, 30]);
}
#[test]
fn NCollection_DynamicArrayTest__InsertAfter_Last() {
    insert_case(&[10, 20, 30], 2, true, &[10, 20, 30, 99]);
}
#[test]
fn NCollection_DynamicArrayTest__InsertAfter_SingleElement() {
    insert_case(&[10], 0, true, &[10, 99]);
}
#[test]
fn NCollection_DynamicArrayTest__InsertBefore_Middle() {
    insert_case(&[10, 20, 30, 40], 2, false, &[10, 20, 99, 30, 40]);
}
#[test]
fn NCollection_DynamicArrayTest__InsertBefore_First() {
    insert_case(&[10, 20, 30], 0, false, &[99, 10, 20, 30]);
}
#[test]
fn NCollection_DynamicArrayTest__InsertBefore_Last() {
    insert_case(&[10, 20, 30], 2, false, &[10, 20, 99, 30]);
}
#[test]
fn NCollection_DynamicArrayTest__InsertAfter_MoveSemantics() {
    let mut array = DynamicArray::default();
    for value in [10, 20, 30] {
        array.emplace_append(|| MoveOnly(value));
    }
    array.insert_after(1, MoveOnly(99)).unwrap();
    assert_eq!(array.length(), 4);
    for (index, expected) in [10, 20, 99, 30].into_iter().enumerate() {
        assert_eq!(array[index].0, expected);
    }
}
#[test]
fn NCollection_DynamicArrayTest__InsertAfter_MultipleInserts() {
    let mut array = values(&[10, 40]);
    array.insert_after(0, 20).unwrap();
    array.insert_after(1, 30).unwrap();
    equal(&array, &[10, 20, 30, 40]);
}
