//! Sparse slots preserve gaps, ascending iteration and object destruction.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::SparseArray;
fn populated(increment: usize, pairs: &[(usize, i32)]) -> SparseArray<i32> {
    let mut array = SparseArray::new(increment);
    for &(index, value) in pairs {
        array.set_value(index, value);
    }
    array
}
#[test]
fn NCollection_SparseArrayTest__BasicFunctions() {
    let mut array = SparseArray::new(10);
    assert!(array.is_empty());
    assert_eq!(array.len(), 0);
    assert_eq!(array.extent(), 0);
    array.set_value(5, 55);
    array.set_value(15, 155);
    assert!(!array.is_empty());
    assert_eq!(array.len(), 2);
    assert_eq!(array.value(5), Some(&55));
    assert_eq!(array[15], 155);
    assert!(array.has_value(5));
    assert!(array.is_bound(15));
    assert!(!array.has_value(10));
    assert!(!array.is_bound(20));
    assert!(array.unset_value(5));
    assert!(!array.has_value(5));
    assert_eq!(array.len(), 1);
    *array.change_value(15).unwrap() = 255;
    assert_eq!(array.value(15), Some(&255));
}
#[test]
fn NCollection_SparseArrayTest__LargeIndices() {
    let array = populated(10, &[(1000, 1000), (10000, 10000)]);
    assert_eq!(array.len(), 2);
    assert!(array.has_value(1000));
    assert!(array.has_value(10000));
    assert_eq!(array.value(1000), Some(&1000));
    assert_eq!(array.value(10000), Some(&10000));
}
#[test]
fn NCollection_SparseArrayTest__SparseDistribution() {
    let array = populated(8, &[(3, 3), (10, 10), (17, 17), (100, 100), (1000, 1000)]);
    assert_eq!(array.len(), 5);
    for index in [3, 10, 17, 100, 1000] {
        assert!(array.has_value(index));
    }
    for index in [4, 11, 101] {
        assert!(!array.has_value(index));
    }
}
#[test]
fn NCollection_SparseArrayTest__IteratorFunctions() {
    let expected = [(5, 50), (10, 100), (20, 200), (30, 300)];
    let mut array = populated(8, &expected);
    let mut found = [false; 4];
    let mut count = 0;
    for (index, value) in array.entries() {
        if let Some(position) = expected.iter().position(|entry| *entry == (index, *value)) {
            found[position] = true;
        }
        count += 1;
    }
    assert_eq!(count, 4);
    assert!(found.into_iter().all(|value| value));
    for value in array.iter_mut() {
        *value *= 2;
    }
    for (index, value) in expected {
        assert_eq!(array.value(index), Some(&(value * 2)));
    }
}
#[test]
fn NCollection_SparseArrayTest__STLIterators() {
    let mut array = populated(8, &[(5, 50), (10, 100), (30, 300)]);
    let constant = &array;
    assert_eq!(
        array.iter().next().unwrap() as *const i32,
        constant.iter().next().unwrap() as *const i32
    );
    assert!(array.iter().nth(3).is_none());
    assert!(constant.iter().nth(3).is_none());
    let begin = array.iter().next().unwrap();
    let constant_begin = begin;
    assert!(std::ptr::eq(begin, constant_begin));
    assert_eq!(*constant_begin, 50);
    assert_eq!(constant.iter().copied().collect::<Vec<_>>(), [50, 100, 300]);
    assert_eq!(constant.iter().find(|&&value| value == 100), Some(&100));
    array.iter_mut().for_each(|value| *value += 1);
    for (index, value) in [(5, 51), (10, 101), (30, 301)] {
        assert_eq!(array.value(index), Some(&value));
    }
}
#[derive(Clone)]
struct TestClass {
    id: i32,
    value: f64,
}
#[test]
fn NCollection_SparseArrayTest__ComplexType() {
    let mut array = SparseArray::new(10);
    array.set_value(5, TestClass { id: 5, value: 3.14 });
    array.set_value(
        10,
        TestClass {
            id: 10,
            value: 2.718,
        },
    );
    assert_eq!(array.len(), 2);
    assert_eq!(array[5].id, 5);
    assert_eq!(array[5].value, 3.14);
    assert_eq!(array[10].id, 10);
    assert_eq!(array[10].value, 2.718);
    array.change_value(5).unwrap().value = 6.28;
    assert_eq!(array[5].value, 6.28);
    assert!(array.unset_value(5));
    assert!(!array.has_value(5));
}
#[test]
fn NCollection_SparseArrayTest__Clear() {
    let mut array = populated(8, &[(5, 5), (10, 10), (15, 15)]);
    assert_eq!(array.len(), 3);
    array.clear();
    assert!(array.is_empty());
    assert_eq!(array.len(), 0);
    for index in [5, 10, 15] {
        assert!(!array.has_value(index));
    }
}
#[test]
fn NCollection_SparseArrayTest__DataMapInterface() {
    let mut array = SparseArray::new(10);
    array.bind(5, 50);
    array.bind(15, 150);
    assert!(array.is_bound(5));
    assert!(array.is_bound(15));
    assert_eq!(array.find(5), Some(&50));
    assert_eq!(array.find(15), Some(&150));
    *array.change_find(5).unwrap() = 55;
    assert_eq!(array.find(5), Some(&55));
    assert!(array.unbind(5));
    assert!(!array.is_bound(5));
    assert_eq!(array.len(), 1);
}
fn check_contents(array: &SparseArray<i32>, pairs: &[(usize, i32)], missing: &[usize]) {
    assert_eq!(array.len(), pairs.len());
    for &(index, value) in pairs {
        assert!(array.has_value(index));
        assert_eq!(array.value(index), Some(&value));
    }
    for &index in missing {
        assert!(!array.has_value(index));
    }
}
#[test]
fn NCollection_SparseArrayTest__AssignAndExchange() {
    let first = populated(10, &[(5, 50), (15, 150)]);
    let mut second = populated(20, &[(7, 70)]);
    second.assign(&first);
    check_contents(&second, &[(5, 50), (15, 150)], &[7]);
    let mut third = populated(30, &[(10, 100), (20, 200)]);
    second.exchange(&mut third);
    check_contents(&second, &[(10, 100), (20, 200)], &[5, 15]);
    check_contents(&third, &[(5, 50), (15, 150)], &[10, 20]);
}
#[test]
fn NCollection_SparseArrayTest__AssignClearsUnsetItemsInSharedBlock() {
    let source = populated(4, &[(0, 11)]);
    let mut destination = populated(4, &[(1, 22)]);
    destination.assign(&source);
    check_contents(&destination, &[(0, 11)], &[1]);
}
