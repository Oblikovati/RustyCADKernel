#[test]
fn NCollection_DynamicArrayTest__Iterator() {
    let mut array = values(&[10, 20, 30]);
    let mut cursor = array.legacy_cursor::<true>();
    let mut sum = 0;
    while cursor.more() {
        sum += *cursor.value(&array).unwrap();
        cursor.next();
    }
    assert_eq!(sum, 60);
    cursor.initialize(&array);
    while cursor.more() {
        *cursor.change_value(&mut array).unwrap() *= 2;
        cursor.next();
    }
    equal(&array, &[20, 40, 60]);
}
#[test]
fn NCollection_DynamicArrayTest__STLIterators() {
    let mut array = values(&[10, 20, 30]);
    assert_eq!(array.iter().sum::<i32>(), 60);
    let mut sum = 0;
    for value in array.iter_mut() {
        *value *= 2;
        sum += *value;
    }
    assert_eq!(sum, 120);
    equal(&array, &[20, 40, 60]);
}
fn append_with_saved_cursor(factor: i32, check_end: bool) {
    let mut array = DynamicArray::with_increment(2);
    for index in 0..4 {
        array.append((index + 1) * factor);
    }
    let mut cursor = array.begin_cursor();
    let end = array.end_cursor();
    for index in 4..64 {
        array.append((index + 1) * factor);
    }
    if check_end {
        assert_ne!(end, array.end_cursor());
    }
    let mut sum = 0;
    while cursor != end {
        sum += *cursor.value(&array).unwrap();
        cursor = cursor + 1;
    }
    assert_eq!(sum, 10 * factor);
    assert_eq!(array.len(), 64);
}
#[test]
fn NCollection_DynamicArrayTest__STLIteratorSurvivesAppendAcrossBlocks() {
    append_with_saved_cursor(1, true);
}
#[test]
fn NCollection_DynamicArrayTest__ConstSTLIteratorSurvivesAppendAcrossBlocks() {
    append_with_saved_cursor(10, false);
}
#[test]
fn NCollection_DynamicArrayTest__IteratorsAcrossBlocks() {
    let mut array = DynamicArray::with_increment(2);
    for index in 0..6 {
        array.append(index + 1);
    }
    assert_eq!(array.iter().sum::<i32>(), 21);
    let begin = array.begin_cursor();
    assert_eq!(*(begin + 3).value(&array).unwrap(), 4);
    assert_eq!(*(array.end_cursor() - 1).value(&array).unwrap(), 6);
    assert_eq!(array.end_cursor(), begin + 6);
}
fn random_vectors() -> (DynamicArray<i32>, Vec<i32>) {
    let reference: Vec<i32> = include_str!("../fixtures/array1_mt19937_seed1.txt")
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect();
    assert_eq!(reference.len(), 100);
    (values(&reference), reference)
}
#[test]
fn NCollection_DynamicArrayTest__STLAlgorithmCompatibility_MinMax() {
    let (array, reference) = random_vectors();
    assert_eq!(array.iter().min(), reference.iter().min());
    assert_eq!(array.iter().max(), reference.iter().max());
}
#[test]
fn NCollection_DynamicArrayTest__STLAlgorithmCompatibility_Replace() {
    let (mut array, mut reference) = random_vectors();
    let target = *reference.last().unwrap();
    for value in array.iter_mut() {
        if *value == target {
            *value = -1;
        }
    }
    for value in &mut reference {
        if *value == target {
            *value = -1;
        }
    }
    equal(&array, &reference);
}
#[test]
fn NCollection_DynamicArrayTest__STLAlgorithmCompatibility_Reverse() {
    let mut reference: Vec<i32> = (0..100).collect();
    let mut array = values(&reference);
    array.reverse();
    reference.reverse();
    equal(&array, &reference);
}
#[test]
fn NCollection_DynamicArrayTest__STLAlgorithmCompatibility_Sort() {
    let (mut array, mut reference) = random_vectors();
    array.sort();
    reference.sort();
    equal(&array, &reference);
}
