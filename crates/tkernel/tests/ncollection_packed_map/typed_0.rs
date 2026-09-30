#[test]
fn NCollection_PackedMapTypedTest__DefaultConstructor() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    assert!(aMap.is_empty());
    assert_eq!(aMap.extent(), 0);
}

#[test]
fn NCollection_PackedMapTypedTest__ConstructorWithBuckets() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::new(100);
    assert!(aMap.is_empty());
    assert_eq!(aMap.extent(), 0);
}

#[test]
fn NCollection_PackedMapTypedTest__CopyConstructor() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(10 as TypeParam);
    aMap1.add(100 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = aMap1.clone();

    assert_eq!(aMap2.extent(), 3);
    assert!(aMap2.contains(1 as TypeParam));
    assert!(aMap2.contains(10 as TypeParam));
    assert!(aMap2.contains(100 as TypeParam));

    // Ensure deep copy
    aMap1.add(1000 as TypeParam);
    assert_eq!(aMap1.extent(), 4);
    assert_eq!(aMap2.extent(), 3);
    assert!(!aMap2.contains(1000 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__MoveConstructor() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(1 as TypeParam);
    aMap1.add(10 as TypeParam);
    aMap1.add(100 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::move_from(&mut aMap1);

    assert_eq!(aMap2.extent(), 3);
    assert!(aMap2.contains(1 as TypeParam));
    assert!(aMap2.contains(10 as TypeParam));
    assert!(aMap2.contains(100 as TypeParam));
    assert!(aMap1.is_empty());
}

#[test]
fn NCollection_PackedMapTypedTest__CopyAssignment() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(5 as TypeParam);
    aMap1.add(15 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(100 as TypeParam);
    aMap2.add(200 as TypeParam);

    aMap2.assign(&aMap1);

    assert_eq!(aMap2.extent(), 2);
    assert!(aMap2.contains(5 as TypeParam));
    assert!(aMap2.contains(15 as TypeParam));
    assert!(!aMap2.contains(100 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__MoveAssignment() {
    let mut aMap1: PackedMap<TypeParam> = PackedMap::default();
    aMap1.add(5 as TypeParam);
    aMap1.add(15 as TypeParam);

    let mut aMap2: PackedMap<TypeParam> = PackedMap::default();
    aMap2.add(100 as TypeParam);

    aMap2.move_assign(&mut aMap1);

    assert_eq!(aMap2.extent(), 2);
    assert!(aMap2.contains(5 as TypeParam));
    assert!(aMap2.contains(15 as TypeParam));
    assert!(aMap1.is_empty());
}

#[test]
fn NCollection_PackedMapTypedTest__AddAndContains() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    assert!(aMap.add(10 as TypeParam));
    assert!(aMap.add(20 as TypeParam));
    assert!(aMap.add(30 as TypeParam));

    assert_eq!(aMap.extent(), 3);
    assert!(aMap.contains(10 as TypeParam));
    assert!(aMap.contains(20 as TypeParam));
    assert!(aMap.contains(30 as TypeParam));
    assert!(!aMap.contains(40 as TypeParam));
}

#[test]
fn NCollection_PackedMapTypedTest__AddDuplicate() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();

    assert!(aMap.add(10 as TypeParam));
    assert_eq!(aMap.extent(), 1);

    assert!(!aMap.add(10 as TypeParam));
    assert_eq!(aMap.extent(), 1);
}

#[test]
fn NCollection_PackedMapTypedTest__Remove() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(10 as TypeParam);
    aMap.add(20 as TypeParam);
    aMap.add(30 as TypeParam);

    assert_eq!(aMap.extent(), 3);

    assert!(aMap.remove(20 as TypeParam));
    assert_eq!(aMap.extent(), 2);
    assert!(!aMap.contains(20 as TypeParam));

    assert!(!aMap.remove(20 as TypeParam));
    assert!(!aMap.remove(40 as TypeParam));
    assert_eq!(aMap.extent(), 2);
}

#[test]
fn NCollection_PackedMapTypedTest__Clear() {
    let mut aMap: PackedMap<TypeParam> = PackedMap::default();
    aMap.add(1 as TypeParam);
    aMap.add(2 as TypeParam);
    aMap.add(3 as TypeParam);

    assert_eq!(aMap.extent(), 3);

    aMap.clear();

    assert!(aMap.is_empty());
    assert_eq!(aMap.extent(), 0);

    aMap.add(100 as TypeParam);
    assert_eq!(aMap.extent(), 1);
    assert!(aMap.contains(100 as TypeParam));
}
