#[test]
fn NCollection_Array2Test__ValueAccess() {
    let array = filled((1, 3), (1, 4), 100);
    for row in 1..=3 {
        for col in 1..=4 {
            assert_eq!(*array.value(row, col).unwrap(), 100 * row + col);
            assert_eq!((*array.value(row, col).unwrap()), 100 * row + col);
        }
    }
}
#[test]
fn NCollection_Array2Test__ChangeValueAccess() {
    let mut array = Array2::with_bounds(0, 2, 0, 3).unwrap();
    for row in 0..=2 {
        for col in 0..=3 {
            *array.change_value(row, col).unwrap() = row * 100 + col;
        }
    }
    assert_eq!((*array.value(1, 2).unwrap()), 102);
    *array.change_value(1, 2).unwrap() = 999;
    assert_eq!((*array.value(1, 2).unwrap()), 999);
}
#[test]
fn NCollection_Array2Test__Init() {
    let mut array = Array2::with_bounds(1, 5, 1, 5).unwrap();
    array.init(42);
    assert!(array.iter().all(|value| *value == 42));
}
#[test]
fn NCollection_Array2Test__ReIndex_UpdateBounds() {
    let mut array = Array2::<i32>::with_bounds(1, 5, 1, 10).unwrap();
    array.update_lower_row(0);
    array.update_lower_col(0);
    shape(&array, 5, 10, (0, 0));
    array.update_upper_row(10);
    array.update_upper_col(20);
    shape(&array, 5, 10, (6, 11));
}
#[test]
fn NCollection_Array2Test__ReIndex_Interference() {
    let mut array = Array2::<i32>::with_bounds(1, 10, 1, 1).unwrap();
    array.update_lower_row(5);
    assert_eq!(array.upper_row(), 14);
    array.update_upper_row(12);
    assert_eq!(array.lower_row(), 3);
    assert_ne!(array.lower_row(), 5);
    assert_eq!(array.nb_rows(), 10);
}
#[test]
fn NCollection_Array2Test__STLIteration() {
    let mut array = filled((1, 2), (1, 3), 10);
    assert_eq!(
        array.iter().copied().collect::<Vec<_>>(),
        [11, 12, 13, 21, 22, 23]
    );
    assert_eq!(array.iter().count(), 6);
    assert_eq!(array.iter().sum::<i32>(), 102);
    let pointer = array.iter().next().unwrap() as *const i32;
    assert_eq!(array.iter_mut().next().unwrap() as *const i32, pointer);
    for value in &mut array {
        *value += 1;
    }
    assert_eq!((*array.value(1, 1).unwrap()), 12);
    assert_eq!((*array.value(2, 3).unwrap()), 24);
}
#[derive(Clone, Default)]
struct Pair {
    integer: i32,
    real: f64,
}
fn check_pair(pair: &Pair, integer: i32, real: f64) {
    assert_eq!(pair.integer, integer);
    assert!((pair.real - real).abs() <= 1e-10);
}
#[test]
fn NCollection_Array2Test__EmplaceValue() {
    let mut array = Array2::<Pair>::with_bounds(1, 3, 1, 4).unwrap();
    check_pair(
        array
            .emplace_value(1, 1, || Pair {
                integer: 42,
                real: 3.14,
            })
            .unwrap(),
        42,
        3.14,
    );
    check_pair(
        array
            .emplace_value(2, 3, || Pair {
                integer: 100,
                real: 2.71,
            })
            .unwrap(),
        100,
        2.71,
    );
    check_pair(array.value(1, 1).unwrap(), 42, 3.14);
    check_pair(array.value(2, 3).unwrap(), 100, 2.71);
    check_pair(array.value(1, 2).unwrap(), 0, 0.0);
    check_pair(array.value(3, 4).unwrap(), 0, 0.0);
}
#[test]
fn NCollection_Array2Test__EmplaceValue_ReplacesExisting() {
    let mut array = Array2::<Pair>::with_bounds(1, 2, 1, 2).unwrap();
    for (row, col, value) in [(1, 1, 11), (1, 2, 12), (2, 1, 21), (2, 2, 22)] {
        array
            .emplace_value(row, col, || Pair {
                integer: value,
                real: f64::from(value) / 10.0,
            })
            .unwrap();
    }
    check_pair(
        array
            .emplace_value(1, 2, || Pair {
                integer: 120,
                real: 12.0,
            })
            .unwrap(),
        120,
        12.0,
    );
    check_pair(array.value(1, 1).unwrap(), 11, 1.1);
    check_pair(array.value(1, 2).unwrap(), 120, 12.0);
    check_pair(array.value(2, 1).unwrap(), 21, 2.1);
    check_pair(array.value(2, 2).unwrap(), 22, 2.2);
}
#[test]
fn NCollection_Array2Test__SizeConstructor_AtAccess() {
    let mut array = Array2::with_size(3, 5);
    for row in 0..3 {
        for col in 0..5 {
            *array.change_at(row, col).unwrap() = row * 100 + col;
        }
    }
    for row in 0..3 {
        for col in 0..5 {
            assert_eq!(*array.at(row, col).unwrap(), row * 100 + col);
        }
    }
}
#[test]
fn NCollection_Array2Test__SizeConstructor_LegacyOperatorZeroBased() {
    let mut array = Array2::<i32>::with_size(2, 3);
    array.set_value(0, 0, 1).unwrap();
    array.set_value(0, 1, 2).unwrap();
    array.set_value(1, 2, 9).unwrap();
    assert_eq!(*array.at(0, 0).unwrap(), 1);
    assert_eq!(*array.at(0, 1).unwrap(), 2);
    assert_eq!(*array.at(1, 2).unwrap(), 9);
}
