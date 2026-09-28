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
fn NCollection_PackedMap64BitTest__LargeValues64Bit() {
    let mut aMap: PackedMap<i64> = PackedMap::default();

    // Values that require 64-bit storage
    let aLargeVal1: i64 = 1_i64 << 40;
    let aLargeVal2: i64 = 1_i64 << 50;
    let aLargeVal3: i64 = 1_i64 << 60;

    aMap.add(aLargeVal1);
    aMap.add(aLargeVal2);
    aMap.add(aLargeVal3);

    assert_eq!(aMap.extent(), 3);
    assert!(aMap.contains(aLargeVal1));
    assert!(aMap.contains(aLargeVal2));
    assert!(aMap.contains(aLargeVal3));

    assert_eq!(aMap.minimal(), aLargeVal1);
    assert_eq!(aMap.maximal(), aLargeVal3);
}

#[test]
fn NCollection_PackedMap64BitTest__Negative64BitValues() {
    let mut aMap: PackedMap<i64> = PackedMap::default();

    let aNegVal1: i64 = -(1_i64 << 40);
    let aNegVal2: i64 = -(1_i64 << 50);

    aMap.add(aNegVal1);
    aMap.add(aNegVal2);
    aMap.add(0 as i64);

    assert_eq!(aMap.extent(), 3);
    assert!(aMap.contains(aNegVal1));
    assert!(aMap.contains(aNegVal2));
    assert!(aMap.contains(0 as i64));
}

#[test]
fn NCollection_PackedMap64BitTest__SizeTypeValues() {
    let mut aMap: PackedMap<usize> = PackedMap::default();

    aMap.add(0 as usize);
    aMap.add((1 as usize) << 32);
    aMap.add(usize::MAX / 2);

    assert_eq!(aMap.extent(), 3);
    assert!(aMap.contains(0 as usize));
    assert!(aMap.contains((1 as usize) << 32));
    assert!(aMap.contains(usize::MAX / 2));
}

#[test]
fn NCollection_PackedMapConfigTest__ConfigFor32BitTypes() {
    assert!(!PackedMap::<i32>::IS_64_BIT);
    assert!(!PackedMap::<u32>::IS_64_BIT);
    assert_eq!(PackedMap::<i32>::BITS_PER_BLOCK, 32);
    assert_eq!(PackedMap::<u32>::BITS_PER_BLOCK, 32);
}

#[test]
fn NCollection_PackedMapConfigTest__ConfigFor64BitTypes() {
    assert!(PackedMap::<i64>::IS_64_BIT);
    assert!(PackedMap::<u64>::IS_64_BIT);
    assert!(PackedMap::<usize>::IS_64_BIT);
    assert_eq!(PackedMap::<i64>::BITS_PER_BLOCK, 64);
    assert_eq!(PackedMap::<usize>::BITS_PER_BLOCK, 64);
}
