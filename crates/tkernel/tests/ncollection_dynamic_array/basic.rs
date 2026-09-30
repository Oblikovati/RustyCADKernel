#[test]
fn NCollection_DynamicArrayTest__DefaultConstructor() {
    let array = DynamicArray::<i32>::default();
    assert_eq!(array.length(), 0);
    assert!(array.is_empty());
}
#[test]
fn NCollection_DynamicArrayTest__ResizeConstructor() {
    let mut array = DynamicArray::with_increment(10);
    for index in 0..10 {
        array.set_value(index, 42).unwrap();
    }
    assert!(!array.is_empty());
    equal(&array, &[42; 10]);
}
#[test]
fn NCollection_DynamicArrayTest__Append() {
    equal(&values(&[10, 20, 30]), &[10, 20, 30]);
}
#[test]
fn NCollection_DynamicArrayTest__SetValue() {
    let mut array = DynamicArray::with_allocator(5, None);
    array.set_value(2, 42).unwrap();
    assert_eq!(array[2], 42);
    assert_eq!(array[0], 0);
    assert_eq!(array[1], 0);
    // The legacy unchecked operator writes an allocated slot outside logical Size().
    array[3] = 99;
    assert_eq!(array[3], 99);
}
#[test]
fn NCollection_DynamicArrayTest__Value() {
    let array = values(&[10, 20]);
    assert_eq!(*array.value(0).unwrap(), 10);
    assert_eq!(*array.value(1).unwrap(), 20);
    assert_eq!(*array.value(0).unwrap(), array[0]);
    assert_eq!(*array.value(1).unwrap(), array[1]);
}
#[test]
fn NCollection_DynamicArrayTest__ChangeValue() {
    let mut array = values(&[10, 20]);
    *array.change_value(1).unwrap() = 25;
    assert_eq!(array[1], 25);
    array[0] = 15;
    assert_eq!(array[0], 15);
}
#[test]
fn NCollection_DynamicArrayTest__FirstLast() {
    let mut array = values(&[10, 20, 30]);
    assert_eq!(*array.first().unwrap(), 10);
    assert_eq!(*array.last().unwrap(), 30);
    *array.change_first().unwrap() = 15;
    *array.change_last().unwrap() = 35;
    assert_eq!(*array.first().unwrap(), 15);
    assert_eq!(*array.last().unwrap(), 35);
}
#[test]
fn NCollection_DynamicArrayTest__CopyConstructor() {
    let mut source = values(&[10, 20, 30]);
    let copy = source.clone();
    equal(&copy, &[10, 20, 30]);
    source[1] = 25;
    assert_eq!(copy[1], 20);
}
#[test]
fn NCollection_DynamicArrayTest__AssignmentOperator() {
    let mut source = values(&[10, 20, 30]);
    let mut copy = DynamicArray::default();
    copy.assign(&source, false);
    equal(&copy, &[10, 20, 30]);
    source[1] = 25;
    assert_eq!(copy[1], 20);
}
#[test]
fn NCollection_DynamicArrayTest__Clear() {
    let mut array = values(&[10, 20]);
    array.clear(false);
    assert_eq!(array.length(), 0);
    assert!(array.is_empty());
}
fn grow_with_increment(increment: usize) {
    let mut array = DynamicArray::default();
    array.set_increment(increment);
    for index in 0..1000 {
        array.append(index);
    }
    assert_eq!(array.length(), 1000);
    for index in 0..1000 {
        assert_eq!(array[index], index);
    }
}
#[test]
fn NCollection_DynamicArrayTest__Grow() {
    grow_with_increment(256);
}
#[test]
fn NCollection_DynamicArrayTest__SetIncrement() {
    grow_with_increment(512);
}
#[test]
fn NCollection_DynamicArrayTest__Move() {
    let mut first = values(&[10, 20]);
    let mut second = DynamicArray::default();
    second.move_from(&mut first);
    assert_eq!(first.length(), 0);
    equal(&second, &[10, 20]);
    let mut third = values(&[30]);
    third.move_from(&mut second);
    assert_eq!(second.length(), 0);
    equal(&third, &[10, 20]);
}
#[test]
fn NCollection_DynamicArrayTest__EraseLast() {
    let mut array = values(&[10, 20, 30]);
    assert_eq!(array.length(), 3);
    array.erase_last();
    equal(&array, &[10, 20]);
    array.erase_last();
    equal(&array, &[10]);
    array.erase_last();
    assert_eq!(array.length(), 0);
    assert!(array.is_empty());
    array.erase_last();
    assert_eq!(array.length(), 0);
}
#[test]
fn NCollection_DynamicArrayTest__Appended() {
    let mut array = DynamicArray::default();
    let first = array.appended();
    *first = 10;
    let pointer = first as *const i32;
    *array.appended() = 20;
    equal(&array, &[10, 20]);
    // Exclusive borrows cannot span Append; verify the same slot survives, then reborrow it.
    assert_eq!(&array[0] as *const i32, pointer);
    array[0] = 15;
    assert_eq!(array[0], 15);
}
#[test]
fn NCollection_DynamicArrayTest__CustomAllocator() {
    let mut array = DynamicArray::with_allocator(256, Some(BaseAllocator::common()));
    for value in [10, 20, 30] {
        array.append(value);
    }
    equal(&array, &[10, 20, 30]);
    array.clear(false);
    assert_eq!(array.length(), 0);
}
