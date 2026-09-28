// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! All four native runtime-stride array regressions.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::AliasedArray;
#[test]
fn NCollection_AliasedArrayTest__ResizeEmptyWithCopy() {
    let mut array = AliasedArray::empty(size_of::<i32>()).unwrap();
    array.resize(2, true).unwrap();
    assert_eq!(2, array.len());
    assert_eq!(2, array.length());
    *array.change_value::<i32>(0).unwrap() = 10;
    *array.change_last::<i32>().unwrap() = 20;
    assert_eq!(10, *array.value::<i32>(0).unwrap());
    assert_eq!(20, *array.value::<i32>(1).unwrap());
}
#[test]
fn NCollection_AliasedArrayTest__ResizeWithoutCopy() {
    let mut array = AliasedArray::new(size_of::<i32>(), 2).unwrap();
    assert_eq!(size_of::<i32>(), array.stride());
    assert!(array.is_deletable());
    *array.change_value::<i32>(0).unwrap() = 10;
    *array.change_value::<i32>(1).unwrap() = 20;
    array.resize(4, false).unwrap();
    assert_eq!(4, array.len());
    assert_eq!(4, array.length());
    *array.change_last::<i32>().unwrap() = 40;
    assert_eq!(40, *array.value::<i32>(3).unwrap());
}
#[test]
fn NCollection_AliasedArrayTest__WrapExternalArray() {
    let mut external: [i32; 2] = [10, 20];
    {
        let mut array = AliasedArray::from_slice(&mut external).unwrap();
        assert_eq!(size_of::<i32>(), array.stride());
        assert!(!array.is_deletable());
        assert_eq!(10, *array.value::<i32>(0).unwrap());
        *array.change_value::<i32>(1).unwrap() = 30;
    }
    assert_eq!(30, external[1]);
}
#[test]
fn NCollection_AliasedArrayTest__TypedDataAndInit() {
    let mut array = AliasedArray::new(size_of::<i32>(), 3).unwrap();
    array.init(7_i32).unwrap();
    let typed: &[i32] = array.typed_data().unwrap();
    assert_eq!(7, typed[0]);
    assert_eq!(7, typed[1]);
    assert_eq!(7, typed[2]);
    let mutable: &mut [i32] = array.change_data().unwrap();
    mutable[1] = 9;
    assert_eq!(9, *array.value::<i32>(1).unwrap());
}
