#[test]
fn NCollection_PackedMapSignedTest__NegativeValues() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    aMap.add(-1 as TypeParam);
    aMap.add(-100 as TypeParam);
    aMap.add(-1000 as TypeParam);

    assert_eq!(aMap.extent(), 3);
    assert!(aMap.contains(-1 as TypeParam));
    assert!(aMap.contains(-100 as TypeParam));
    assert!(aMap.contains(-1000 as TypeParam));
    assert!(!aMap.contains(1 as TypeParam));

    assert!(aMap.remove(-100 as TypeParam));
    assert_eq!(aMap.extent(), 2);
    assert!(!aMap.contains(-100 as TypeParam));
}

#[test]
fn NCollection_PackedMapSignedTest__MixedPositiveNegative() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    aMap.add(-50 as TypeParam);
    aMap.add(0 as TypeParam);
    aMap.add(50 as TypeParam);

    assert_eq!(aMap.extent(), 3);
    assert!(aMap.contains(-50 as TypeParam));
    assert!(aMap.contains(0 as TypeParam));
    assert!(aMap.contains(50 as TypeParam));
}

#[test]
fn NCollection_PackedMapSignedTest__MinMaxWithNegatives() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    aMap.add(-100 as TypeParam);
    aMap.add(-50 as TypeParam);
    aMap.add(0 as TypeParam);
    aMap.add(50 as TypeParam);
    aMap.add(100 as TypeParam);

    assert_eq!(aMap.minimal(), (-100 as TypeParam));
    assert_eq!(aMap.maximal(), (100 as TypeParam));
}
