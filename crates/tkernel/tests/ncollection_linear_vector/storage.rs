#[test]
fn NCollection_LinearVectorTest__BoundsAccess() {
    let vector = integers(&[10, 20, 30]);
    assert_eq!(vector.len(), 3);
    assert_eq!(vector[0], 10);
    assert_eq!(vector[vector.len() - 1], 30);
}
#[test]
fn NCollection_LinearVectorTest__ReserveZero_NoEffect() {
    let mut vector = integers(&[1]);
    let capacity = vector.capacity();
    vector.reserve(0);
    assert_eq!(vector.capacity(), capacity);
    same_values(&vector, &[1]);
}
#[test]
fn NCollection_LinearVectorTest__ReserveDecreasing_NoShrink() {
    let mut vector = LinearVector::<i32>::default();
    vector.reserve(100);
    assert!(vector.capacity() >= 100);
    let capacity = vector.capacity();
    vector.reserve(10);
    assert_eq!(vector.capacity(), capacity);
}
#[test]
fn NCollection_LinearVectorTest__InsertBeforeAtEnd_EquivalentToAppend() {
    let mut vector = integers(&[1, 2]);
    vector.insert_before(vector.len(), 3).unwrap();
    same_values(&vector, &[1, 2, 3]);
}
#[test]
fn NCollection_LinearVectorTest__SetValueMove() {
    let mut vector = LinearVector::default();
    let mut source = "abc".to_owned();
    vector.set_value_with_policies(0, std::mem::take, String::new, |value| {
        *value = std::mem::take(&mut source);
    });
    assert_eq!(vector.len(), 1);
    assert_eq!(vector[0], "abc");
}
#[test]
fn NCollection_LinearVectorTest__InsertBeforeZero_InsertsAtFront() {
    let mut vector = integers(&[2, 3]);
    vector.insert_before(0, 1).unwrap();
    same_values(&vector, &[1, 2, 3]);
}
#[test]
fn NCollection_LinearVectorTest__OutOfRangeIndicesThrow() {
    let mut vector = integers(&[1]);
    assert!(vector.value(5).is_err());
    assert!(vector.change_value(5).is_err());
    assert!(vector.insert_after(5, 1).is_err());
    assert!(vector.erase(5).is_err());
}
#[test]
fn NCollection_LinearVectorTest__Constructor_SizeAndValue() {
    let vector = LinearVector::with_value(5, 42);
    same_values(&vector, &[42; 5]);
}
#[test]
fn NCollection_LinearVectorTest__Constructor_SizeAndValue_NonTrivial() {
    let vector = LinearVector::with_value(3, AsciiString::new("hello"));
    assert_eq!(vector.len(), 3);
    for value in &vector {
        assert_eq!(value.as_bytes(), b"hello");
    }
}
#[test]
fn NCollection_LinearVectorTest__Constructor_SizeZero_IsEmpty() {
    assert!(LinearVector::with_value(0, 99).is_empty());
}
#[test]
fn NCollection_LinearVectorTest__OcctApiCoverage() {
    let mut vector = LinearVector::default();
    assert!(vector.is_empty());
    assert_eq!(vector.len(), 0);
    vector.append(10);
    *vector.emplace_append(|| 20) = 30;
    assert!(!vector.is_empty());
    assert_eq!(vector.len(), 2);
    assert!(vector.capacity() >= vector.len());
    assert_eq!(*vector.first().unwrap(), 10);
    assert_eq!(*vector.last().unwrap(), 30);
    assert_eq!(&vector[0] as *const i32, vector.as_slice().as_ptr());
    assert_eq!(*vector.value(0).unwrap(), 10);
    assert_eq!(*vector.value(1).unwrap(), 30);
    vector.reserve(8);
    assert!(vector.capacity() >= 8);
    vector.resize_with_value(4, 7);
    assert_eq!(vector.len(), 4);
    assert_eq!(vector[2], 7);
    assert_eq!(vector[3], 7);
    vector.erase_last();
    assert_eq!(vector.len(), 3);
    assert_eq!(*vector.last().unwrap(), 7);
    vector.clear(false);
    assert!(vector.is_empty());
    assert_eq!(vector.len(), 0);
}
#[test]
fn NCollection_LinearVectorTest__ToArray1_SamePointer() {
    let mut vector = LinearVector::default();
    for index in 0..10 {
        vector.append(index * 10);
    }
    let pointer = vector.as_slice().as_ptr();
    let array = vector.to_array1();
    assert_eq!(array.len(), 10);
    assert_eq!(&array[0] as *const i32, pointer);
}
#[test]
fn NCollection_LinearVectorTest__ToArray1_ModifyReflectsInSource() {
    let mut vector = integers(&[1, 2, 3, 4, 5]);
    let mut array = vector.to_array1();
    array[0] = 99;
    array[4] = 77;
    assert_eq!(vector[0], 99);
    assert_eq!(vector[1], 2);
    assert_eq!(vector[4], 77);
}
#[test]
fn NCollection_LinearVectorTest__ToArray1_EmptyVector() {
    let mut vector = LinearVector::<i32>::default();
    let array = vector.to_array1();
    assert_eq!(array.len(), 0);
    assert!(array.is_empty());
}
#[test]
fn NCollection_LinearVectorTest__ToArray1_ReadThrough() {
    let mut vector = LinearVector::default();
    for index in 0..7 {
        vector.append(index * 11);
    }
    let array = vector.to_array1();
    for index in 0..7 {
        assert_eq!(array[index], index * 11);
    }
}
#[test]
fn NCollection_LinearVectorTest__ToArray1_NonTrivial() {
    let mut vector = strings(&["alpha", "beta", "gamma"]);
    let pointer = &vector[0] as *const String;
    let mut array = vector.to_array1();
    assert_eq!(array.len(), 3);
    assert_eq!(&array[0] as *const String, pointer);
    assert_eq!(array[0], "alpha");
    assert_eq!(array[1], "beta");
    assert_eq!(array[2], "gamma");
    array[1] = "modified".into();
    assert_eq!(vector[1], "modified");
}
