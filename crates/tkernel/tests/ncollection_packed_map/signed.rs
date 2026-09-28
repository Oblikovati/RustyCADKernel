// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

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
