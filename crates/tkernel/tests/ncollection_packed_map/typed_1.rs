#[test]
fn NCollection_PackedMapTypedTest__STLIterator() {
    let mut aMap = PackedMap::<TypeParam>::default();
    for value in [1, 32, 100] {
        aMap.add(value as TypeParam);
    }
    let mut aValues: Vec<TypeParam> = Vec::new();
    for aValue in &aMap {
        aValues.push(aValue);
    }
    assert_eq!(
        vec![1 as TypeParam, 32 as TypeParam, 100 as TypeParam],
        aValues
    );
}

#[test]
fn NCollection_PackedMapTypedTest__ZeroValue() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    assert!(aMap.add(0 as TypeParam));
    assert!(aMap.contains(0 as TypeParam));
    assert_eq!(aMap.extent(), 1);

    assert!(aMap.remove(0 as TypeParam));
    assert!(!aMap.contains(0 as TypeParam));
    assert!(aMap.is_empty());
}

#[test]
fn NCollection_PackedMapTypedTest__ConsecutiveValues() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    let aNbValues: i32 = 64;

    for i in 0..aNbValues {
        aMap.add(i as TypeParam);
    }

    assert_eq!(aMap.extent(), aNbValues);

    for i in 0..aNbValues {
        assert!(aMap.contains(i as TypeParam));
    }
}

#[test]
fn NCollection_PackedMapTypedTest__SparseValues() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    aMap.add(0 as TypeParam);
    aMap.add(100 as TypeParam);
    aMap.add(1000 as TypeParam);
    aMap.add(10000 as TypeParam);
    aMap.add(100000 as TypeParam);

    assert_eq!(aMap.extent(), 5);

    assert!(aMap.contains(0 as TypeParam));
    assert!(aMap.contains(100 as TypeParam));
    assert!(aMap.contains(1000 as TypeParam));
    assert!(aMap.contains(10000 as TypeParam));
    assert!(aMap.contains(100000 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__BlockBoundary() {
    // Test values around block boundaries (31, 32, 33 for 32-bit, 63, 64, 65 for 64-bit)
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    let aBitsPerBlock: i32 = PackedMap::<TypeParam>::BITS_PER_BLOCK;

    aMap.add((aBitsPerBlock - 1) as TypeParam);
    aMap.add(aBitsPerBlock as TypeParam);
    aMap.add((aBitsPerBlock + 1) as TypeParam);

    assert_eq!(aMap.extent(), 3);
    assert!(aMap.contains((aBitsPerBlock - 1) as TypeParam));
    assert!(aMap.contains(aBitsPerBlock as TypeParam));
    assert!(aMap.contains((aBitsPerBlock + 1) as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__GetMinimalMapped() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    aMap.add(50 as TypeParam);
    aMap.add(10 as TypeParam);
    aMap.add(100 as TypeParam);
    aMap.add(5 as TypeParam);
    aMap.add(75 as TypeParam);

    assert_eq!(aMap.minimal(), (5 as TypeParam));

    aMap.remove(5 as TypeParam);
    assert_eq!(aMap.minimal(), (10 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__GetMaximalMapped() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    aMap.add(50 as TypeParam);
    aMap.add(10 as TypeParam);
    aMap.add(100 as TypeParam);
    aMap.add(5 as TypeParam);
    aMap.add(75 as TypeParam);

    assert_eq!(aMap.maximal(), (100 as TypeParam));

    aMap.remove(100 as TypeParam);
    assert_eq!(aMap.maximal(), (75 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__Union() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(3 as TypeParam);
    aMap2.add(4 as TypeParam);
    aMap2.add(5 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::union(&mut aResult, &aMap1, &aMap2);

    assert_eq!(aResult.extent(), 5);
    assert!(aResult.contains(1 as TypeParam));
    assert!(aResult.contains(2 as TypeParam));
    assert!(aResult.contains(3 as TypeParam));
    assert!(aResult.contains(4 as TypeParam));
    assert!(aResult.contains(5 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__Unite() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(3 as TypeParam);
    aMap2.add(4 as TypeParam);
    aMap2.add(5 as TypeParam);

    assert!(packed_map_algo::unite(&mut aMap1, &aMap2));

    assert_eq!(aMap1.extent(), 5);
    assert!(aMap1.contains(1 as TypeParam));
    assert!(aMap1.contains(2 as TypeParam));
    assert!(aMap1.contains(3 as TypeParam));
    assert!(aMap1.contains(4 as TypeParam));
    assert!(aMap1.contains(5 as TypeParam));

    assert!(!packed_map_algo::unite(&mut aMap1, &aMap2));
}

#[test]
fn NCollection_PackedMapTypedTest__UniteInPlace() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);

    packed_map_algo::unite(&mut aMap1, &aMap2);

    assert_eq!(aMap1.extent(), 3);
    assert!(aMap1.contains(1 as TypeParam));
    assert!(aMap1.contains(2 as TypeParam));
    assert!(aMap1.contains(3 as TypeParam));
}
