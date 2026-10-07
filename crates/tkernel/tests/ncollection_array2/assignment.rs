#[test]
fn NCollection_Array2Test__CopyConstructor() {
    let mut original = Array2::with_bounds(1, 3, 1, 4).unwrap();
    original.init(123);
    let copy = original.clone();
    shape(&copy, 3, 4, (1, 1));
    assert_eq!((*copy.value(2, 3).unwrap()), 123);
    original.set_value(2, 3, 999).unwrap();
    assert_ne!((*copy.value(2, 3).unwrap()), (*original.value(2, 3).unwrap()));
    assert_eq!((*copy.value(2, 3).unwrap()), 123);
}
#[test]
fn NCollection_Array2Test__AssignmentOperator() {
    let mut source = filled((5, 7), (10, 13), 100);
    let mut dest = Array2::with_bounds(1, 1, 1, 1).unwrap();
    dest.init(0);
    dest.assign(&source);
    shape(&dest, 3, 4, (5, 10));
    assert_eq!((*dest.value(6, 12).unwrap()), 612);
    source.set_value(6, 12, 999).unwrap();
    assert_eq!((*dest.value(6, 12).unwrap()), 612);
}
#[test]
fn NCollection_Array2Test__AssignmentOperatorReusesSameSizeOwnedStorage() {
    let source = filled((10, 12), (20, 23), 100);
    let mut dest = Array2::<i32>::with_bounds(1, 3, 1, 4).unwrap();
    let pointer = dest.as_slice().as_ptr();
    dest.assign(&source);
    assert_eq!(dest.as_slice().as_ptr(), pointer);
    assert!(dest.is_deletable());
    shape(&dest, 3, 4, (10, 20));
    assert_eq!((*dest.value(11, 22).unwrap()), 1122);
}
#[test]
fn NCollection_Array2Test__AssignmentOperatorReusesSameSizeExternalBuffer() {
    let mut buffer = [0; 4];
    let pointer = buffer.as_ptr();
    let mut dest = Array2::from_buffer(&mut buffer, 2, 2).unwrap();
    dest.assign(&filled((5, 6), (7, 8), 100));
    assert_eq!(dest.as_slice().as_ptr(), pointer);
    assert!(!dest.is_deletable());
    shape(&dest, 2, 2, (5, 7));
    assert_eq!(buffer, [507, 508, 607, 608]);
}
#[test]
fn NCollection_Array2Test__AssignmentOperatorDetachesDifferentSizeExternalBuffer() {
    let mut buffer = [1, 2, 3, 4];
    let pointer = buffer.as_ptr();
    let mut dest = Array2::from_buffer(&mut buffer, 2, 2).unwrap();
    dest.assign(&filled((5, 6), (7, 9), 100));
    assert_ne!(dest.as_slice().as_ptr(), pointer);
    assert!(dest.is_deletable());
    shape(&dest, 2, 3, (5, 7));
    assert_eq!((*dest.value(6, 8).unwrap()), 608);
    assert_eq!(buffer, [1, 2, 3, 4]);
}
#[test]
fn NCollection_Array2Test__CopyValuesPreservesBounds() {
    let source = filled((5, 6), (7, 9), 100);
    let mut dest = Array2::<i32>::with_bounds(1, 2, 1, 3).unwrap();
    dest.copy_values(&source).unwrap();
    shape(&dest, 2, 3, (1, 1));
    assert_eq!((*dest.value(1, 1).unwrap()), 507);
    assert_eq!((*dest.value(2, 3).unwrap()), 609);
}
#[test]
fn NCollection_Array2Test__CopyValuesDifferentDimensions() {
    let source = Array2::<i32>::with_bounds(1, 2, 1, 3).unwrap();
    let mut dest = Array2::<i32>::with_bounds(1, 3, 1, 2).unwrap();
    assert!(dest.copy_values(&source).is_err());
}
fn check_move(mut dest: Array2<'static, i32>) {
    let mut source = Array2::with_bounds(1, 5, 1, 10).unwrap();
    source.set_value(3, 7, 123).unwrap();
    dest.move_from(&mut source);
    shape(&dest, 5, 10, (1, 1));
    assert_eq!((*dest.value(3, 7).unwrap()), 123);
    assert_eq!(source.len(), 0);
    assert_eq!(source.nb_rows(), 0);
}
#[test]
fn NCollection_Array2Test__MoveConstructor() {
    check_move(Array2::default());
}
#[test]
fn NCollection_Array2Test__MoveAssignment() {
    check_move(Array2::default());
}
