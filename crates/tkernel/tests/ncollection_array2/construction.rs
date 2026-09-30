#[test]
fn NCollection_Array2Test__DefaultConstructor() {
    let array = Array2::<i32>::default();
    shape(&array, 0, 0, (1, 1));
    assert!(array.is_empty());
}
#[test]
fn NCollection_Array2Test__ConstructorWithBounds() {
    shape(&Array2::with_bounds(1, 5, 1, 10).unwrap(), 5, 10, (1, 1));
}
#[test]
fn NCollection_Array2Test__ConstructorWithNegativeBounds() {
    shape(&Array2::with_bounds(-2, 2, -5, 5).unwrap(), 5, 11, (-2, -5));
}
#[test]
fn NCollection_Array2Test__SizeConstructor_AllocatesCorrectly() {
    let array = Array2::<i32>::with_size(3, 4);
    shape(&array, 3, 4, (0, 0));
    assert!(array.is_deletable());
}
#[test]
fn NCollection_Array2Test__SizeConstructor_BufferReuse_NotOwner() {
    let mut buffer = [1, 2, 3, 4, 5, 6];
    let array = Array2::from_buffer(&mut buffer, 2, 3).unwrap();
    shape(&array, 2, 3, (0, 0));
    assert!(!array.is_deletable());
}
#[test]
fn NCollection_Array2Test__SizeConstructor_BufferReuse_DataPreserved() {
    let mut buffer = [10, 20, 30, 40, 50, 60];
    let array = Array2::from_buffer(&mut buffer, 2, 3).unwrap();
    for row in 0..2 {
        for col in 0..3 {
            assert_eq!(
                *array.at(row, col).unwrap(),
                (row * 3 + col + 1) as i32 * 10
            );
        }
    }
}
