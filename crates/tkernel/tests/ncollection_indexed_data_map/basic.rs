#[test]
fn NCollection_IndexedDataMapTest__DefaultConstructor() {
    let map = NumericMap::default();
    assert!(map.is_empty());
    assert_eq!(map.extent(), 0);
    assert_eq!(map.len(), 0);
}

#[test]
fn NCollection_IndexedDataMapTest__BasicAddFind() {
    let mut map = NumericMap::default();
    for (index, key) in [10, 20, 30].into_iter().enumerate() {
        assert_eq!(map.add(key, key as f64 / 10.0), index + 1);
    }
    check_numeric(&map, &[10, 20, 30]);
    assert!(!map.contains(&40));
}

#[test]
fn NCollection_IndexedDataMapTest__DuplicateKey() {
    let mut map = NumericMap::default();
    assert_eq!(map.add(10, 1.0), 1);
    assert_eq!(map.add(20, 2.0), 2);
    assert_eq!(map.add(10, 3.0), 1);
    assert_eq!(map.extent(), 2);
    assert_eq!(map.find_from_key(&10), Ok(&1.0));
}

#[test]
fn NCollection_IndexedDataMapTest__ChangeValues() {
    let mut map = numeric([10, 20]);
    *map.change_from_index(1).unwrap() = 1.5;
    *map.change_from_index(2).unwrap() = 2.5;
    check_values(&map, &[1.5, 2.5]);
    *map.change_from_key(&10).unwrap() = 1.75;
    *map.change_from_key(&20).unwrap() = 2.75;
    check_values(&map, &[1.75, 2.75]);
    map[1] = 1.9;
    map[2] = 2.9;
    check_values(&map, &[1.9, 2.9]);
}

#[test]
fn NCollection_IndexedDataMapTest__SeekTests() {
    let mut map = numeric([10, 20]);
    assert_eq!(map.seek(&10), Some(&1.0));
    assert_eq!(map.seek(&20), Some(&2.0));
    assert_eq!(map.seek(&30), None);
    assert!(map.change_seek(&10).is_some());
    assert!(map.change_seek(&20).is_some());
    assert!(map.change_seek(&30).is_none());
    *map.change_seek(&10).unwrap() = 1.5;
    *map.change_seek(&20).unwrap() = 2.5;
    check_values(&map, &[1.5, 2.5]);
    let mut value = 0.0;
    assert!(map.find_copy(&10, &mut value));
    assert_eq!(value, 1.5);
    assert!(!map.find_copy(&30, &mut value));
}

#[test]
fn NCollection_IndexedDataMapTest__RemoveLast() {
    let mut map = numeric([10, 20, 30]);
    map.remove_last().unwrap();
    check_numeric(&map, &[10, 20]);
    assert!(!map.contains(&30));
}

#[test]
fn NCollection_IndexedDataMapTest__RemoveFromIndex() {
    let mut map = numeric([10, 20, 30, 40]);
    map.remove_from_index(2).unwrap();
    check_numeric(&map, &[10, 40, 30]);
    assert!(!map.contains(&20));
}

#[test]
fn NCollection_IndexedDataMapTest__RemoveKey() {
    let mut map = numeric([10, 20, 30]);
    map.remove_key(&20);
    assert_eq!(map.extent(), 2);
    assert!(map.contains(&10));
    assert!(!map.contains(&20));
    assert!(map.contains(&30));
    map.remove_key(&50);
    assert_eq!(map.extent(), 2);
}

#[test]
fn NCollection_IndexedDataMapTest__Substitute() {
    let mut map = numeric([10, 20, 30]);
    map.substitute(2, 25, 2.5).unwrap();
    check_numeric(&map, &[10, 25, 30]);
    assert!(!map.contains(&20));
}

#[test]
fn NCollection_IndexedDataMapTest__Swap() {
    let mut map = numeric([10, 20, 30]);
    map.swap(1, 3).unwrap();
    check_numeric(&map, &[30, 20, 10]);
}

#[test]
fn NCollection_IndexedDataMapTest__Clear() {
    let mut map = numeric([10, 20, 30]);
    assert_eq!(map.extent(), 3);
    map.clear(true);
    assert!(map.is_empty());
    assert_eq!(map.extent(), 0);
    assert_eq!(map.add(40, 4.0), 1);
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_IndexedDataMapTest__CopyConstructor() {
    let mut first = numeric([10, 20, 30]);
    let second = first.clone();
    check_numeric(&second, &[10, 20, 30]);
    first.add(40, 4.0);
    assert_eq!(first.extent(), 4);
    assert_eq!(second.extent(), 3);
    assert!(!second.contains(&40));
}

#[test]
fn NCollection_IndexedDataMapTest__AssignmentOperator() {
    let first = numeric([10, 20]);
    let mut second = numeric([30, 40, 50]);
    second.assign(&first);
    check_numeric(&second, &[10, 20]);
    for key in [30, 40, 50] {
        assert!(!second.contains(&key));
    }
}

#[test]
fn NCollection_IndexedDataMapTest__Iterator() {
    let mut map = numeric([10, 20, 30]);
    let mut found = [false; 3];
    let mut count = 0;
    for (key, value) in map.items() {
        if let Some(i) = [(10, 1.0), (20, 2.0), (30, 3.0)]
            .iter()
            .position(|pair| pair == &(*key, *value))
        {
            found[i] = true;
        }
        count += 1;
    }
    assert_eq!(count, 3);
    assert_eq!(found, [true; 3]);
    for (_, value) in map.items_mut() {
        *value *= 2.0;
    }
    check_values(&map, &[2.0, 4.0, 6.0]);
}

#[test]
fn NCollection_IndexedDataMapTest__StlIterator() {
    let map = numeric([10, 20, 30]);
    for _ in 0..2 {
        let mut found = [false; 3];
        let mut count = 0;
        for value in map.iter() {
            if let Some(i) = [1.0, 2.0, 3.0].iter().position(|item| item == value) {
                found[i] = true;
            }
            count += 1;
        }
        assert_eq!(count, 3);
        assert_eq!(found, [true; 3]);
    }
}

#[test]
fn NCollection_IndexedDataMapTest__StringKeys() {
    let mut map = IndexedDataMap::<AsciiString, f64>::default();
    for (i, name) in ["First", "Second", "Third"].iter().enumerate() {
        assert_eq!(map.add(AsciiString::new(name), (i + 1) as f64), i + 1);
    }
    for (i, name) in ["First", "Second", "Third"].iter().enumerate() {
        let key = AsciiString::new(name);
        assert_eq!(map.find_index(&key), i + 1);
        assert_eq!(map.find_key(i + 1), Ok(&key));
        assert_eq!(map.find_from_key(&key), Ok(&((i + 1) as f64)));
    }
}

#[test]
fn NCollection_IndexedDataMapTest__ComplexKeyAndValue() {
    let mut map = IndexedDataMap::<TestKey, TestItem>::default();
    let keys = [
        TestKey::new(1, "One"),
        TestKey::new(2, "Two"),
        TestKey::new(3, "Three"),
    ];
    let values = [
        TestItem::new(1.1, "Item One"),
        TestItem::new(2.2, "Item Two"),
        TestItem::new(3.3, "Item Three"),
    ];
    for (i, (key, value)) in keys.iter().zip(&values).enumerate() {
        assert_eq!(map.add(key.clone(), value.clone()), i + 1);
    }
    for (i, (key, value)) in keys.iter().zip(&values).enumerate() {
        assert_eq!(map.find_index(key), i + 1);
        assert_eq!(map.find_key(i + 1).unwrap().id, i as i32 + 1);
        assert_eq!(map.find_from_key(key).unwrap().value, value.value);
    }
    assert!(map.contains(&TestKey::new(1, "One")));
    assert!(!map.contains(&TestKey::new(4, "Four")));
    map.change_from_key(&keys[0]).unwrap().value = 1.5;
    assert_eq!(map.find_from_key(&keys[0]).unwrap().value, 1.5);
}

#[test]
fn NCollection_IndexedDataMapTest__Exchange() {
    let mut first = numeric([10, 20]);
    let mut second = numeric([30, 40, 50]);
    first.exchange(&mut second);
    check_numeric(&first, &[30, 40, 50]);
    check_numeric(&second, &[10, 20]);
}

#[test]
fn NCollection_IndexedDataMapTest__ReSize() {
    let mut map = NumericMap::new(3);
    for key in 1..=100 {
        map.add(key, key as f64 / 10.0);
    }
    let keys: Vec<i32> = (1..=100).collect();
    check_numeric(&map, &keys);
    map.resize(200);
    check_numeric(&map, &keys);
}

#[test]
fn NCollection_IndexedDataMapTest__STLAlgorithmCompatibility_MinMax() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    for key in (10..=50).step_by(5) {
        map.add(key, key * 2);
    }
    assert!(!map.is_empty());
    let min = map.iter().min();
    let max = map.iter().max();
    assert!(min.is_some());
    assert!(max.is_some());
    assert!(min.unwrap() <= max.unwrap());
}

#[test]
fn NCollection_IndexedDataMapTest__STLAlgorithmCompatibility_Find() {
    let mut map = IndexedDataMap::<i32, i32>::default();
    for key in [100, 200, 300] {
        map.add(key, key * 2);
    }
    let found = map.iter().find(|&&value| value == 200);
    assert!(found.is_some());
    assert_eq!(found, Some(&200));
    assert_eq!(map.iter().find(|&&value| value == 999), None);
}
