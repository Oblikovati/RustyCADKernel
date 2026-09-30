#[test]
fn NCollection_Array1Test__ValueAccess() {
    let mut array = Array1::with_bounds(1, 5).unwrap();
    for i in 1..=5 { array.set_value(i, i * 10).unwrap(); }
    for i in 1..=5 {
        assert_eq!(i * 10, *array.value(i).unwrap());
        assert_eq!(i * 10, array[i]);
    }
}

#[test]
fn NCollection_Array1Test__ChangeValueAccess() {
    let mut array = Array1::with_bounds(1, 5).unwrap();
    for i in 1..=5 { *array.change_value(i).unwrap() = i * 10; }
    for i in 1..=5 { assert_eq!(i * 10, array[i]); }
    for i in 1..=5 { *array.change_value(i).unwrap() += 5; }
    for i in 1..=5 { assert_eq!(i * 10 + 5, array[i]); }
}

#[test]
fn NCollection_Array1Test__Init() {
    let mut array = Array1::with_bounds(1, 5).unwrap();
    array.init(42);
    for i in array.lower()..=array.upper() { assert_eq!(42, array[i]); }
}

#[test]
fn NCollection_Array1Test__SetValue() {
    let mut array = Array1::with_bounds(1, 5).unwrap();
    array.set_value(3, 123).unwrap();
    assert_eq!(123, array[3]);
}

#[test]
fn NCollection_Array1Test__FirstLast() {
    let mut array = Array1::with_bounds(5, 10).unwrap();
    array[5] = 55;
    array[10] = 1010;
    assert_eq!(55, *array.first().unwrap());
    assert_eq!(1010, *array.last().unwrap());
    *array.change_first().unwrap() = 555;
    *array.change_last().unwrap() = 10101;
    assert_eq!(555, *array.first().unwrap());
    assert_eq!(10101, *array.last().unwrap());
}

#[test]
fn NCollection_Array1Test__STLIteration() {
    let array = filled_array(1, 5);
    for (index, value) in array.iter().enumerate() { assert_eq!((index as i32 + 1) * 10, *value); }
    let const_array = &array;
    assert_eq!(const_array.iter().as_slice().as_ptr_range(), array.as_slice().as_ptr_range());
    let found = const_array.iter().find(|&&v| v == 30);
    assert!(found.is_some());
    assert_eq!(30, *found.unwrap());
}

#[test]
fn NCollection_Array1Test__Resize() {
    let mut array = filled_array(1, 5);
    array.resize_bounds(1, 10, true).unwrap();
    assert_eq!(10, array.len());
    assert_eq!(1, array.lower());
    assert_eq!(10, array.upper());
    for i in 1..=5 { assert_eq!(i * 10, array[i]); }
    let mut array2 = filled_array(1, 10);
    array2.resize_bounds(1, 5, true).unwrap();
    assert_eq!(5, array2.len());
    assert_eq!(1, array2.lower());
    assert_eq!(5, array2.upper());
    for i in 1..=5 { assert_eq!(i * 10, array2[i]); }
}

#[test]
fn NCollection_Array1Test__ChangeValue() {
    let mut array = Array1::with_bounds(1, 5).unwrap();
    array.init(42);
    *array.change_value(3).unwrap() = 123;
    assert_eq!(123, array[3]);
    for i in [1, 2, 4, 5] { assert_eq!(42, array[i]); }
}

#[test]
fn NCollection_Array1Test__IteratorAccess() {
    let array = filled_array(1, 5);
    for (index, value) in array.iter().enumerate() { assert_eq!((index as i32 + 1) * 10, *value); }
    for (index, value) in (&array).into_iter().enumerate() { assert_eq!((index as i32 + 1) * 10, *value); }
}
