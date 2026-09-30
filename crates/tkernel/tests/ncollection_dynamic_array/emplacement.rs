#[test]
fn NCollection_DynamicArrayTest__EmplaceAppend() {
    let mut array = DynamicArray::default();
    check_pair(
        array.emplace_append(|| MultiArg {
            integer: 42,
            real: 3.14,
        }),
        42,
        3.14,
    );
    assert_eq!(array.length(), 1);
    check_pair(
        array.emplace_append(|| MultiArg {
            integer: 100,
            real: 2.71,
        }),
        100,
        2.71,
    );
    assert_eq!(array.length(), 2);
    assert_eq!(array[0].integer, 42);
    assert_eq!(array[1].integer, 100);
}
#[test]
fn NCollection_DynamicArrayTest__EmplaceValue() {
    let mut array = DynamicArray::default();
    check_pair(
        array
            .emplace_value(0, || MultiArg {
                integer: 10,
                real: 1.0,
            })
            .unwrap(),
        10,
        1.0,
    );
    assert_eq!(array.length(), 1);
    check_pair(
        array
            .emplace_value(3, || MultiArg {
                integer: 40,
                real: 4.0,
            })
            .unwrap(),
        40,
        4.0,
    );
    assert_eq!(array.length(), 4);
    check_pair(&array[1], 0, 0.0);
    check_pair(&array[2], 0, 0.0);
}
#[test]
fn NCollection_DynamicArrayTest__EmplaceValue_ReplacesExisting() {
    let mut array = DynamicArray::default();
    for index in 0..3 {
        array
            .emplace_value(index, || MultiArg {
                integer: (index + 1) * 10,
                real: f64::from(index + 1),
            })
            .unwrap();
    }
    assert_eq!(array.length(), 3);
    check_pair(
        array
            .emplace_value(1, || MultiArg {
                integer: 200,
                real: 20.0,
            })
            .unwrap(),
        200,
        20.0,
    );
    assert_eq!(array.length(), 3);
    assert_eq!(array[0].integer, 10);
    assert_eq!(array[1].integer, 200);
    assert_eq!(array[2].integer, 30);
}
#[test]
fn NCollection_DynamicArrayTest__EmplaceWithMoveOnlyType() {
    let mut array = DynamicArray::default();
    assert_eq!(array.emplace_append(|| MoveOnly(42)).0, 42);
    assert_eq!(array.emplace_append(|| MoveOnly(100)).0, 100);
    assert_eq!(array.length(), 2);
    assert_eq!(array[0].0, 42);
    assert_eq!(array[1].0, 100);
}
#[test]
fn NCollection_DynamicArrayTest__ReplaceConstructOnlyType() {
    let mut array = DynamicArray::default();
    array.emplace_value(0, || ConstructOnly(10)).unwrap();
    array.emplace_value(0, || ConstructOnly(20)).unwrap();
    assert_eq!(array[0].0, 20);
    let replacement = array[0].clone();
    array.emplace_value(0, || replacement).unwrap();
    assert_eq!(array[0].0, 20);
    array.set_value(0, ConstructOnly(30)).unwrap();
    assert_eq!(array[0].0, 30);
}
#[test]
fn NCollection_DynamicArrayTest__EmplaceAppendMany() {
    let mut array = DynamicArray::default();
    for index in 0..1000 {
        check_pair(
            array.emplace_append(|| MultiArg {
                integer: index,
                real: f64::from(index) * 0.1,
            }),
            index,
            f64::from(index) * 0.1,
        );
    }
    assert_eq!(array.length(), 1000);
    for index in 0..1000 {
        check_pair(&array[index], index as i32, index as f64 * 0.1);
    }
}
#[test]
fn NCollection_DynamicArrayTest__SetValue_ReplacesExisting() {
    let mut array = DynamicArray::default();
    for index in 0..3 {
        array
            .set_value(
                index,
                MultiArg {
                    integer: (index + 1) * 10,
                    real: f64::from(index + 1),
                },
            )
            .unwrap();
    }
    assert_eq!(array.length(), 3);
    check_pair(
        array
            .set_value(
                1,
                MultiArg {
                    integer: 200,
                    real: 20.0,
                },
            )
            .unwrap(),
        200,
        20.0,
    );
    assert_eq!(array.length(), 3);
    assert_eq!(array[0].integer, 10);
    assert_eq!(array[1].integer, 200);
    assert_eq!(array[2].integer, 30);
}
