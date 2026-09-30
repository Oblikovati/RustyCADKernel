#[test]
fn NCollection_PackedMapTypedTest__IsEqual() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(1 as TypeParam);
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);

    assert!(packed_map_algo::is_equal(&aMap1, &aMap2));

    aMap2.add(4 as TypeParam);
    assert!(!packed_map_algo::is_equal(&aMap1, &aMap2));
}

#[test]
fn NCollection_PackedMapTypedTest__IsEqualEmpty() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();

    assert!(packed_map_algo::is_equal(&aMap1, &aMap2));
}

#[test]
fn NCollection_PackedMapTypedTest__IsSubset() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(1 as TypeParam);
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);
    aMap2.add(4 as TypeParam);

    assert!(packed_map_algo::is_subset(&aMap1, &aMap2));
    assert!(!packed_map_algo::is_subset(&aMap2, &aMap1));
}

#[test]
fn NCollection_PackedMapTypedTest__IsSubsetSame() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(1 as TypeParam);
    aMap2.add(2 as TypeParam);

    assert!(packed_map_algo::is_subset(&aMap1, &aMap2));
    assert!(packed_map_algo::is_subset(&aMap2, &aMap1));
}

#[test]
fn NCollection_PackedMapTypedTest__IsSubsetEmpty() {
    let mut aEmptyMap: PackedMap<TypeParam> = PackedMap::default();
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);

    assert!(packed_map_algo::is_subset(&aEmptyMap, &aMap));
    assert!(packed_map_algo::is_subset(&aEmptyMap, &aEmptyMap));
}

#[test]
fn NCollection_PackedMapTypedTest__HasIntersection() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(3 as TypeParam);
    aMap2.add(4 as TypeParam);
    aMap2.add(5 as TypeParam);

    assert!(packed_map_algo::has_intersection(&aMap1, &aMap2));

    let mut aMap3: PackedMap<TypeParam> = PackedMap::default();
    aMap3.add(10 as TypeParam);
    aMap3.add(20 as TypeParam);

    assert!(!packed_map_algo::has_intersection(&aMap1, &aMap3));
}

#[test]
fn NCollection_PackedMapTypedTest__IteratorBasic() {
    let mut aMap = PackedMap::<TypeParam>::default();
    for value in [10, 20, 30] {
        aMap.add(value as TypeParam);
    }
    let mut aValues: Vec<TypeParam> = Vec::new();
    let mut anIt = aMap.iter();
    while anIt.more() {
        aValues.push(*anIt.key().unwrap());
        anIt.advance();
    }
    assert_eq!(aValues.len(), 3);
    for value in [10, 20, 30] {
        assert!(aValues.contains(&(value as TypeParam)));
    }
}

#[test]
fn NCollection_PackedMapTypedTest__STLIterators() {
    let mut aMap = PackedMap::<TypeParam>::default();
    for value in [10, 20, 30] {
        aMap.add(value as TypeParam);
    }
    let aValues: Vec<TypeParam> = aMap.iter().collect();
    assert_eq!(aValues.len(), 3);
    for value in [10, 20, 30] {
        assert!(aValues.contains(&(value as TypeParam)));
    }
    let aConstMap: &PackedMap<TypeParam> = &aMap;
    let anFound: Option<TypeParam> = aConstMap.iter().find(|value| *value == 20 as TypeParam);
    assert!(anFound.is_some());
    assert_eq!(20 as TypeParam, anFound.unwrap());
    assert_eq!(
        1,
        aConstMap
            .iter()
            .filter(|value| *value == 10 as TypeParam)
            .count()
    );
    let mut aCount: i32 = 0;
    for _ in aConstMap.iter() {
        aCount += 1;
    }
    assert_eq!(aCount, 3);
}

#[test]
fn NCollection_PackedMapTypedTest__IteratorEmpty() {
    let aMap = PackedMap::<TypeParam>::default();
    let mut aCount: i32 = 0;
    let mut anIt = aMap.iter();
    while anIt.more() {
        aCount += 1;
        anIt.advance();
    }
    assert_eq!(aCount, 0);
}

#[test]
fn NCollection_PackedMapTypedTest__IteratorReinitialize() {
    let mut aMap1 = PackedMap::<TypeParam>::default();
    for value in [1, 2] {
        aMap1.add(value as TypeParam);
    }
    let mut aMap2 = PackedMap::<TypeParam>::default();
    for value in [100, 200, 300] {
        aMap2.add(value as TypeParam);
    }
    let mut anIt = aMap1.iter();
    let mut aCount1: i32 = 0;
    while anIt.more() {
        aCount1 += 1;
        anIt.advance();
    }
    assert_eq!(aCount1, 2);
    anIt.initialize(&aMap2);
    let mut aCount2: i32 = 0;
    while anIt.more() {
        aCount2 += 1;
        anIt.advance();
    }
    assert_eq!(aCount2, 3);
}
