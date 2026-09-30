#[test]
fn NCollection_PackedMapTypedTest__Intersection() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);
    aMap1.add(4 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);
    aMap2.add(5 as TypeParam);
    aMap2.add(6 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::intersection(&mut aResult, &aMap1, &aMap2);

    assert_eq!(aResult.extent(), 2);
    assert!(aResult.contains(2 as TypeParam));
    assert!(aResult.contains(3 as TypeParam));
    assert!(!aResult.contains(1 as TypeParam));
    assert!(!aResult.contains(4 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__Intersect() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);
    aMap1.add(4 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);
    aMap2.add(5 as TypeParam);

    assert!(packed_map_algo::intersect(&mut aMap1, &aMap2));

    assert_eq!(aMap1.extent(), 2);
    assert!(aMap1.contains(2 as TypeParam));
    assert!(aMap1.contains(3 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__IntersectInPlace() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);
    aMap2.add(4 as TypeParam);

    packed_map_algo::intersect(&mut aMap1, &aMap2);

    assert_eq!(aMap1.extent(), 2);
    assert!(aMap1.contains(2 as TypeParam));
    assert!(aMap1.contains(3 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__IntersectionDisjoint() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(3 as TypeParam);
    aMap2.add(4 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::intersection(&mut aResult, &aMap1, &aMap2);

    assert!(aResult.is_empty());
}

#[test]
fn NCollection_PackedMapTypedTest__Subtraction() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);
    aMap1.add(4 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);
    aMap2.add(5 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::subtraction(&mut aResult, &aMap1, &aMap2);

    assert_eq!(aResult.extent(), 2);
    assert!(aResult.contains(1 as TypeParam));
    assert!(aResult.contains(4 as TypeParam));
    assert!(!aResult.contains(2 as TypeParam));
    assert!(!aResult.contains(3 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__Subtract() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);
    aMap1.add(4 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);
    aMap2.add(5 as TypeParam);

    assert!(packed_map_algo::subtract(&mut aMap1, &aMap2));

    assert_eq!(aMap1.extent(), 2);
    assert!(aMap1.contains(1 as TypeParam));
    assert!(aMap1.contains(4 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__SubtractInPlace() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);

    packed_map_algo::subtract(&mut aMap1, &aMap2);

    assert_eq!(aMap1.extent(), 2);
    assert!(aMap1.contains(1 as TypeParam));
    assert!(aMap1.contains(3 as TypeParam));
    assert!(!aMap1.contains(2 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__Difference() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);
    aMap2.add(4 as TypeParam);

    let mut aResult: PackedMap<TypeParam> = PackedMap::default();
    packed_map_algo::difference(&mut aResult, &aMap1, &aMap2);

    assert_eq!(aResult.extent(), 2);
    assert!(aResult.contains(1 as TypeParam));
    assert!(aResult.contains(4 as TypeParam));
    assert!(!aResult.contains(2 as TypeParam));
    assert!(!aResult.contains(3 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__Differ() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);
    aMap1.add(3 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);
    aMap2.add(4 as TypeParam);

    assert!(packed_map_algo::differ(&mut aMap1, &aMap2));

    assert_eq!(aMap1.extent(), 2);
    assert!(aMap1.contains(1 as TypeParam));
    assert!(aMap1.contains(4 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__DifferInPlace() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(2 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(2 as TypeParam);
    aMap2.add(3 as TypeParam);

    packed_map_algo::differ(&mut aMap1, &aMap2);

    assert_eq!(aMap1.extent(), 2);
    assert!(aMap1.contains(1 as TypeParam));
    assert!(aMap1.contains(3 as TypeParam));
    assert!(!aMap1.contains(2 as TypeParam));
}
