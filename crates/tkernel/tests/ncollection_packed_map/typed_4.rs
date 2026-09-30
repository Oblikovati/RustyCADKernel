#[test]
fn NCollection_PackedMapTypedTest__IteratorReset() {
    let mut aMap = PackedMap::<TypeParam>::default();
    for value in [1, 2] {
        aMap.add(value as TypeParam);
    }
    let mut anIt = aMap.iter();
    let mut aCount1: i32 = 0;
    while anIt.more() {
        aCount1 += 1;
        anIt.advance();
    }
    assert_eq!(aCount1, 2);
    anIt.reset();
    let mut aCount2: i32 = 0;
    while anIt.more() {
        aCount2 += 1;
        anIt.advance();
    }
    assert_eq!(aCount2, 2);
}

#[test]
fn NCollection_PackedMapTypedTest__ReSize() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::new(1);

    for i in 0..1000 {
        aMap.add(i as TypeParam);
    }

    assert_eq!(aMap.extent(), 1000);

    for i in 0..1000 {
        assert!(aMap.contains(i as TypeParam));
    }
}

#[test]
fn NCollection_PackedMapTypedTest__LargeValues() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    aMap.add(1000000 as TypeParam);
    aMap.add(2000000 as TypeParam);
    aMap.add(3000000 as TypeParam);

    assert_eq!(aMap.extent(), 3);
    assert!(aMap.contains(1000000 as TypeParam));
    assert!(aMap.contains(2000000 as TypeParam));
    assert!(aMap.contains(3000000 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__UnionWithSelf() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);
    aMap.add(2 as TypeParam);
    aMap.add(3 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::union(&mut aResult, &aMap, &aMap);

    assert_eq!(aResult.extent(), 3);
    assert!(packed_map_algo::is_equal(&aResult, &aMap));
}

#[test]
fn NCollection_PackedMapTypedTest__IntersectionWithSelf() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);
    aMap.add(2 as TypeParam);
    aMap.add(3 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::intersection(&mut aResult, &aMap, &aMap);

    assert_eq!(aResult.extent(), 3);
    assert!(packed_map_algo::is_equal(&aResult, &aMap));
}

#[test]
fn NCollection_PackedMapTypedTest__SubtractionFromSelf() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);
    aMap.add(2 as TypeParam);
    aMap.add(3 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::subtraction(&mut aResult, &aMap, &aMap);

    assert!(aResult.is_empty());
}

#[test]
fn NCollection_PackedMapTypedTest__DifferenceWithSelf() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);
    aMap.add(2 as TypeParam);
    aMap.add(3 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::difference(&mut aResult, &aMap, &aMap);

    assert!(aResult.is_empty());
}

#[test]
fn NCollection_PackedMapTypedTest__UnionWithEmpty() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);
    aMap.add(2 as TypeParam);

    let mut aEmpty: PackedMap<TypeParam> = PackedMap::default();

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::union(&mut aResult, &aMap, &aEmpty);

    assert!(packed_map_algo::is_equal(&aResult, &aMap));
}

#[test]
fn NCollection_PackedMapTypedTest__IntersectionWithEmpty() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);
    aMap.add(2 as TypeParam);

    let mut aEmpty: PackedMap<TypeParam> = PackedMap::default();

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::intersection(&mut aResult, &aMap, &aEmpty);

    assert!(aResult.is_empty());
}

#[test]
fn NCollection_PackedMapTypedTest__SubtractionEmpty() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);
    aMap.add(2 as TypeParam);

    let mut aEmpty: PackedMap<TypeParam> = PackedMap::default();

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::subtraction(&mut aResult, &aMap, &aEmpty);

    assert!(packed_map_algo::is_equal(&aResult, &aMap));

    let mut aResult2: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::subtraction(&mut aResult2, &aEmpty, &aMap);

    assert!(aResult2.is_empty());
}
