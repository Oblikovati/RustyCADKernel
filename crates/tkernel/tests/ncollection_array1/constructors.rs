#[test]
fn NCollection_Array1Test__EmptyRangesAndDefaultIterators() {
    let array = Array1::<i32>::default();
    let range = array.as_slice().as_ptr_range();
    assert_eq!(range.start, range.end);
    assert_eq!(array.iter().as_slice().as_ptr_range().start, range.end);
    assert_eq!(0, array.iter().count());
    assert!(std::slice::Iter::<i32>::default().next().is_none());
    assert!(std::slice::IterMut::<i32>::default().next().is_none());
}

#[test]
fn NCollection_Array1Test__ConstructorWithBounds() {
    let array = Array1::<i32>::with_bounds(1, 5).unwrap();
    assert_eq!(5, array.len());
    assert_eq!(1, array.lower());
    assert_eq!(5, array.upper());
}

#[test]
fn NCollection_Array1Test__ConstructorWithSizeTUpperUsesBounds() {
    let upper: usize = 3;
    let mut array = Array1::<i32>::with_bounds(1, upper as i32).unwrap();
    assert!(array.is_deletable());
    assert_eq!(3, array.len());
    assert_eq!(1, array.lower());
    assert_eq!(3, array.upper());
    *array.change_value(1).unwrap() = 10;
    *array.change_value(2).unwrap() = 20;
    *array.change_value(3).unwrap() = 30;
    assert_eq!(10, *array.value(1).unwrap());
    assert_eq!(20, *array.value(2).unwrap());
    assert_eq!(30, *array.value(3).unwrap());
}

#[test]
fn NCollection_Array1Test__ReferenceSizeConstructorRequiresExplicitBufferPolicy() {
    let mut buffer = [1, 2, 3];
    let original = buffer.as_ptr();
    let mut view = Array1::from_buffer(&mut buffer, true);
    assert!(!view.is_deletable());
    assert_eq!(0, view.lower());
    assert_eq!(2, view.upper());
    assert_eq!(original, view.as_slice().as_ptr());
    *view.change_at(1).unwrap() = 20;
    assert_eq!(20, buffer[1]);
    let owned = Array1::from_buffer(&mut buffer, false);
    assert!(owned.is_deletable());
    assert_eq!(0, owned.lower());
    assert_eq!(2, owned.upper());
    assert_ne!(original, owned.as_slice().as_ptr());
}

#[test]
fn NCollection_Array1Test__ConstructorWithNegativeBounds() {
    let array = Array1::<i32>::with_bounds(-3, 2).unwrap();
    assert_eq!(6, array.len());
    assert_eq!(-3, array.lower());
    assert_eq!(2, array.upper());
}
