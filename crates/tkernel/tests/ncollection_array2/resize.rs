fn check_region(array: &Array2<'_, i32>, rows: i32, cols: i32, lower: (i32, i32), factor: i32) {
    for row in 0..rows {
        for col in 0..cols {
            assert_eq!(
                (*array.value(lower.0 + row, lower.1 + col).unwrap()),
                (lower.0 + row) * factor + lower.1 + col
            );
        }
    }
}
#[test]
fn NCollection_Array2Test__Resize() {
    let mut array = filled((1, 4), (1, 5), 100);
    array.resize_bounds(0, 5, 0, 6, true).unwrap();
    shape(&array, 6, 7, (0, 0));
    for row in 0..4 {
        for col in 0..5 {
            assert_eq!((*array.value(row, col).unwrap()), (row + 1) * 100 + col + 1);
        }
    }
}
#[test]
fn NCollection_Array2Test__ResizeWithTrim_PreservesElementPositions() {
    let source = filled((1, 5), (1, 4), 100);
    for (rows, cols) in [(3, 4), (5, 2), (3, 2), (7, 6)] {
        let mut array = source.clone();
        array.resize_trim_bounds(1, rows, 1, cols, true).unwrap();
        shape(&array, rows as usize, cols as usize, (1, 1));
        check_region(&array, rows.min(5), cols.min(4), (1, 1), 100);
    }
}
#[test]
fn NCollection_Array2Test__ResizeWithTrim_NonUnitLowerBounds() {
    let source = filled((3, 5), (10, 12), 100);
    for (upper_row, upper_col) in [(4, 12), (7, 14), (5, 12)] {
        let mut array = source.clone();
        array
            .resize_trim_bounds(3, upper_row, 10, upper_col, true)
            .unwrap();
        shape(
            &array,
            (upper_row - 2) as usize,
            (upper_col - 9) as usize,
            (3, 10),
        );
        check_region(
            &array,
            (upper_row - 2).min(3),
            (upper_col - 9).min(3),
            (3, 10),
            100,
        );
    }
}
#[test]
fn NCollection_Array2Test__Resize_FromEmpty() {
    let mut array = Array2::<i32>::default();
    array.resize_bounds(1, 3, 1, 2, true).unwrap();
    shape(&array, 3, 2, (1, 1));
    let mut trimmed = Array2::<i32>::default();
    trimmed.resize_trim_bounds(1, 2, 1, 4, true).unwrap();
    shape(&trimmed, 2, 4, (1, 1));
}
fn check_rebased(trim: bool) {
    let mut array = filled((1, 3), (1, 3), 10);
    if trim {
        array.resize_trim_bounds(5, 7, 10, 12, true).unwrap();
    } else {
        array.resize_bounds(5, 7, 10, 12, true).unwrap();
    }
    shape(&array, 3, 3, (5, 10));
    for row in 5..=7 {
        for col in 10..=12 {
            assert_eq!((*array.value(row, col).unwrap()), (row - 4) * 10 + col - 9);
        }
    }
}
#[test]
fn NCollection_Array2Test__Resize_SameSizeNewBounds() {
    check_rebased(true);
}
#[test]
fn NCollection_Array2Test__Resize_SameSizeNewBounds_Linear() {
    check_rebased(false);
}
#[test]
fn NCollection_Array2Test__Resize_ChangeShapeSameSize() {
    let mut array = Array2::<i32>::with_bounds(1, 4, 1, 6).unwrap();
    for (index, value) in array.as_mut_slice().iter_mut().enumerate() {
        *value = index as i32;
    }
    array.resize_bounds(1, 6, 1, 4, true).unwrap();
    shape(&array, 6, 4, (1, 1));
    for index in 0..16 {
        assert_eq!(*array.array1().at(index).unwrap(), index as i32);
    }
}
#[test]
fn NCollection_Array2Test__Resize_NoCopy() {
    let mut array = Array2::with_bounds(1, 3, 1, 4).unwrap();
    array.init(42);
    array.resize_bounds(1, 5, 1, 2, false).unwrap();
    shape(&array, 5, 2, (1, 1));
    array.init(7);
    array.resize_bounds(10, 14, 20, 21, false).unwrap();
    shape(&array, 5, 2, (10, 20));
}
#[test]
fn NCollection_Array2Test__ResizeWithTrim_GrowPreservesOldRegion() {
    let mut array = Array2::with_bounds(1, 2, 1, 2).unwrap();
    array.init(99);
    array.resize_trim_bounds(1, 4, 1, 3, true).unwrap();
    for row in 1..=2 {
        for col in 1..=2 {
            assert_eq!((*array.value(row, col).unwrap()), 99);
        }
    }
}
#[test]
fn NCollection_Array2Test__SizeConstructor_Resize_Grow() {
    let mut array = Array2::with_size(2, 2);
    array.init(7);
    array.resize(3, 3, true).unwrap();
    shape(&array, 3, 3, (0, 0));
    assert_eq!(*array.at(0, 0).unwrap(), 7);
    assert_eq!(*array.at(0, 1).unwrap(), 7);
}
#[test]
fn NCollection_Array2Test__SizeConstructor_Resize_NoData() {
    let mut array = Array2::with_size(3, 4);
    array.init(5);
    array.resize(2, 2, false).unwrap();
    shape(&array, 2, 2, (0, 0));
    assert!(array.is_deletable());
}
#[test]
fn NCollection_Array2Test__SizeConstructor_ResizeWithTrim_GrowPreserves() {
    let mut array = Array2::with_size(2, 2);
    array.init(42);
    array.resize_with_trim(3, 3, true).unwrap();
    shape(&array, 3, 3, (0, 0));
    for row in 0..2 {
        for col in 0..2 {
            assert_eq!(*array.at(row, col).unwrap(), 42);
        }
    }
}
