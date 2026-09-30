#[test]
fn NCollection_DoubleMapTest__DefaultConstructor() {
    let map = NumericMap::default();
    assert!(map.is_empty());
    assert_eq!(map.extent(), 0);
    assert_eq!(map.len(), 0);
}

#[test]
fn NCollection_DoubleMapTest__CustomBuckets() {
    let map = NumericMap::new(100);
    assert!(map.is_empty());
    assert_eq!(map.extent(), 0);
    assert_eq!(map.len(), 0);
}

#[test]
fn NCollection_DoubleMapTest__BasicBindFind() {
    let map = numeric([10, 20, 30]);
    check_pairs(&map, &[10, 20, 30]);
    assert!(!map.is_bound1(&40));
    assert!(!map.is_bound2(&4.0));
}

#[test]
fn NCollection_DoubleMapTest__BindWithDuplicateKeys() {
    let mut map = numeric([10]);
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find1(&10), Ok(&1.0));
    assert!(map.bind(10, 2.0).is_err());
    assert_eq!(map.extent(), 1);
    assert_eq!(map.find1(&10), Ok(&1.0));
    assert!(!map.is_bound2(&2.0));
    assert!(map.bind(20, 1.0).is_err());
    assert_eq!(map.extent(), 1);
    assert!(!map.is_bound1(&20));
}

#[test]
fn NCollection_DoubleMapTest__UnbindTest() {
    let mut map = numeric([10, 20, 30]);
    assert_eq!(map.extent(), 3);
    map.unbind1(&20);
    assert_eq!(map.extent(), 2);
    assert!(!map.is_bound1(&20));
    assert!(!map.is_bound2(&2.0));
    map.unbind2(&3.0);
    assert_eq!(map.extent(), 1);
    assert!(!map.is_bound1(&30));
    assert!(!map.is_bound2(&3.0));
    map.unbind1(&40);
    assert_eq!(map.extent(), 1);
    map.unbind2(&4.0);
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_DoubleMapTest__Clear() {
    let mut map = numeric([10, 20, 30]);
    assert_eq!(map.extent(), 3);
    map.clear(true);
    assert!(map.is_empty());
    assert_eq!(map.extent(), 0);
    map.bind(40, 4.0).unwrap();
    assert_eq!(map.extent(), 1);
}

#[test]
fn NCollection_DoubleMapTest__SeekTests() {
    let map = numeric([10, 20]);
    assert_eq!(map.seek1(&10), Some(&1.0));
    assert_eq!(map.seek2(&2.0), Some(&20));
    assert_eq!(map.seek1(&30), None);
    assert_eq!(map.seek2(&3.0), None);
}

#[test]
fn NCollection_DoubleMapTest__CopyConstructor() {
    let mut first = numeric([10, 20, 30]);
    let second = first.clone();
    check_pairs(&second, &[10, 20, 30]);
    first.bind(40, 4.0).unwrap();
    assert_eq!(first.extent(), 4);
    assert_eq!(second.extent(), 3);
    assert!(!second.is_bound1(&40));
    assert!(!second.is_bound2(&4.0));
}

#[test]
fn NCollection_DoubleMapTest__AssignmentOperator() {
    let first = numeric([10, 20]);
    let mut second = numeric([30, 40, 50]);
    second.assign(&first);
    check_pairs(&second, &[10, 20]);
    for key in [30, 40, 50] {
        assert!(!second.is_bound1(&key));
        assert!(!second.is_bound2(&(key as f64 / 10.0)));
    }
}

#[test]
fn NCollection_DoubleMapTest__ReSize() {
    let mut map = NumericMap::new(3);
    for key in 1..=100 {
        map.bind(key, key as f64 / 10.0).unwrap();
    }
    check_pairs(&map, &(1..=100).collect::<Vec<_>>());
}

#[test]
fn NCollection_DoubleMapTest__Exchange() {
    let mut first = numeric([10, 20]);
    let mut second = numeric([30, 40, 50]);
    first.exchange(&mut second);
    check_pairs(&first, &[30, 40, 50]);
    check_pairs(&second, &[10, 20]);
}

#[test]
fn NCollection_DoubleMapTest__Iterator() {
    let map = numeric([10, 20, 30]);
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
}

#[test]
fn NCollection_DoubleMapTest__STLIterators() {
    let map = numeric([10, 20, 30]);
    let shared = &map;
    assert!(map.iter() == shared.iter());
    assert!(map.end() == shared.end());
    assert_eq!(shared.iter().filter(|&&value| value == 2.0).count(), 1);
    let found = shared.iter().find(|&&value| value == 3.0);
    assert!(found.is_some());
    assert_eq!(found, Some(&3.0));
    let mut values = Vec::new();
    for value in shared {
        values.push(*value);
    }
    assert_eq!(values.len(), 3);
}

#[test]
fn NCollection_DoubleMapTest__StringKeys() {
    let mut map = DoubleMap::<AsciiString, AsciiString>::default();
    for i in 1..=3 {
        map.bind(
            AsciiString::new(&format!("Key{i}")),
            AsciiString::new(&format!("Value{i}")),
        )
        .unwrap();
    }
    assert_eq!(map.extent(), 3);
    for i in 1..=3 {
        let key = AsciiString::new(&format!("Key{i}"));
        let value = AsciiString::new(&format!("Value{i}"));
        assert!(map.is_bound1(&key));
        assert!(map.is_bound2(&value));
        assert_eq!(map.find1(&key), Ok(&value));
        assert_eq!(map.find2(&value), Ok(&key));
    }
}

#[test]
fn NCollection_DoubleMapTest__ComplexKeys() {
    let mut map = DoubleMap::<TestKey1, TestKey2>::default();
    let first = [
        TestKey1::new(1, "One"),
        TestKey1::new(2, "Two"),
        TestKey1::new(3, "Three"),
    ];
    let second = [
        TestKey2::new(1.1, "A"),
        TestKey2::new(2.2, "B"),
        TestKey2::new(3.3, "C"),
    ];
    for (key, value) in first.iter().zip(&second) {
        map.bind(key.clone(), value.clone()).unwrap();
    }
    assert_eq!(map.extent(), 3);
    for (key, value) in first.iter().zip(&second) {
        assert!(map.is_bound1(key));
        assert!(map.is_bound2(value));
    }
    let found2 = map.find1(&first[1]).unwrap();
    assert_eq!(found2.value, 2.2);
    assert_eq!(found2.code, "B");
    let found1 = map.find2(&second[2]).unwrap();
    assert_eq!(found1.id, 3);
    assert_eq!(found1.name, "Three");
}
