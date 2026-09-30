fn all_visited(map: &StringMap) {
    let mut keys = BTreeSet::new();
    let mut values = BTreeSet::new();
    for (key, value) in map.items() {
        keys.insert(*key);
        values.insert(value.as_bytes());
    }
    assert_eq!(keys.len(), 3);
    for key in [1, 2, 3] {
        assert!(keys.contains(&key));
    }
    assert_eq!(values.len(), 3);
    for value in ["One", "Two", "Three"] {
        assert!(values.contains(value.as_bytes()));
    }
}
#[test]
fn NCollection_DataMapTest__IteratorAccess() {
    all_visited(&string_map(&[(1, "One"), (2, "Two"), (3, "Three")]));
}
#[test]
fn NCollection_DataMapTest__ChangeValue() {
    let mut map = string_map(&[(1, "One")]);
    let value = map.values_mut().next().unwrap();
    check(value, "One");
    value.copy("Modified via Iterator");
    check(value, "Modified via Iterator");
    check(map.find(&1).unwrap(), "Modified via Iterator");
}
#[test]
fn NCollection_DataMapTest__ExhaustiveIterator() {
    let mut map = DataMap::<i32, i32>::default();
    for i in 0..1000 {
        map.bind(i, i * 2);
    }
    assert_eq!(map.len(), 1000);
    let mut count = 0;
    for (key, value) in map.items() {
        assert_eq!(*value, key * 2);
        count += 1;
    }
    assert_eq!(count, 1000);
}
#[test]
fn NCollection_DataMapTest__STLAlgorithmCompatibility_MinMax() {
    let mut map = DataMap::<i32, i32>::default();
    for i in (10..=50).step_by(5) {
        map.bind(i, i * 2);
    }
    assert!(!map.is_empty());
    let min = map.values().min();
    let max = map.values().max();
    assert!(min.is_some());
    assert!(max.is_some());
    assert!(min.unwrap() <= max.unwrap());
}
#[test]
fn NCollection_DataMapTest__STLAlgorithmCompatibility_Find() {
    let mut map = DataMap::<i32, i32>::default();
    for (key, value) in [(100, 200), (200, 400), (300, 600)] {
        map.bind(key, value);
    }
    assert_eq!(map.values().find(|&&value| value == 200), Some(&200));
    assert!(map.values().find(|&&value| value == 999).is_none());
}
#[test]
fn NCollection_DataMapTest__ItemsIteration() {
    all_visited(&string_map(&[(1, "One"), (2, "Two"), (3, "Three")]));
}
#[test]
fn NCollection_DataMapTest__ItemsStructuredBindings() {
    let map = string_map(&[(10, "Ten"), (20, "Twenty")]);
    let mut sum = 0;
    for (key, _value) in map.items() {
        sum += key;
    }
    assert_eq!(sum, 30);
}
#[test]
fn NCollection_DataMapTest__ItemsModifyValue() {
    let mut map = string_map(&[(1, "Original")]);
    for (_key, value) in map.items_mut() {
        value.copy("Modified");
    }
    check(map.find(&1).unwrap(), "Modified");
}
#[test]
fn NCollection_DataMapTest__ConstItemsIteration() {
    let map = string_map(&[(1, "One"), (2, "Two")]);
    let constant = &map;
    let mut count = 0;
    for (_key, _value) in constant.items() {
        count += 1;
    }
    assert_eq!(count, 2);
}
#[test]
fn NCollection_DataMapTest__StandardIteratorContract() {
    let mut map = string_map(&[(1, "one"), (2, "two")]);
    let constant = &map;
    assert!(map.items() == constant.items());
    assert!(map.items().nth(2).is_none());
    assert!(constant.items().nth(2).is_none());
    for value in map.values_mut() {
        *value += "!";
    }
    check(map.find(&1).unwrap(), "one!");
    check(map.find(&2).unwrap(), "two!");
}
#[test]
fn NCollection_DataMapTest__ItemsIteratorEquality() {
    let map = string_map(&[(1, "One"), (2, "Two"), (3, "Three")]);
    assert!(map.items() == map.items());
    let mut end = map.items();
    for _ in end.by_ref() {}
    let end2 = end.clone();
    assert!(end == end2);
    assert!(map.items() != end);
    let first = map.items();
    let mut second = map.items();
    second.next();
    assert!(first != second);
    let mut third = map.items();
    third.next();
    assert!(second == third);
}
