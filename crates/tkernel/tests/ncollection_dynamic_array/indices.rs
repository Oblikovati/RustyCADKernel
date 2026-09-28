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
fn NCollection_DynamicArrayTest__SizeReturnsSizeT() {
    let mut array = DynamicArray::default();
    let size: usize = array.len();
    assert_eq!(size, 0);
    array.append(42);
    array.append(43);
    assert_eq!(array.len(), 2_usize);
}
#[test]
fn NCollection_DynamicArrayTest__LengthStillReturnsInt() {
    let array = DynamicArray::<i32>::default();
    let length: i32 = array.length();
    assert_eq!(length, 0);
}
#[test]
fn NCollection_DynamicArrayTest__UpperReturnsMinusOneForEmpty() {
    let mut array = DynamicArray::default();
    assert_eq!(array.lower(), 0);
    assert_eq!(array.upper(), -1);
    array.append(10);
    assert_eq!(array.upper(), 0);
}
#[test]
fn NCollection_DynamicArrayTest__SizeTConstructor() {
    let mut array = DynamicArray::with_increment(128_usize);
    array.append(1);
    assert_eq!(array.len(), 1_usize);
}
#[test]
fn NCollection_DynamicArrayTest__SizeTValueAccess() {
    let mut array = DynamicArray::default();
    for index in 0..10 {
        array.append(index * 7);
    }
    let index = 2_usize;
    assert_eq!(*array.value(index).unwrap(), 14);
    assert_eq!(array[index], 14);
    assert_eq!(array[index], 14);
}
#[test]
fn NCollection_DynamicArrayTest__SizeTSetValueExtends() {
    let mut array = DynamicArray::default();
    array.set_value(5_usize, 99).unwrap();
    assert_eq!(array.len(), 6);
    assert_eq!(*array.value(5_usize).unwrap(), 99);
}
#[test]
fn NCollection_DynamicArrayTest__SizeTEmplaceValue() {
    let mut array = DynamicArray::default();
    array.emplace_value(3_usize, || (1, 2.5)).unwrap();
    assert_eq!(array.len(), 4);
    assert_eq!(array[3].0, 1);
    assert_eq!(array[3].1, 2.5);
}
#[test]
fn NCollection_DynamicArrayTest__SizeTInsertBeforeAfter() {
    let mut array = values(&[0, 1, 2]);
    array.insert_before(1_usize, 99).unwrap();
    assert_eq!(array.len(), 4);
    assert_eq!(array[1], 99);
    assert_eq!(array[2], 1);
    array.insert_after(0_usize, 42).unwrap();
    assert_eq!(array.len(), 5);
    assert_eq!(array[1], 42);
}
#[test]
fn NCollection_DynamicArrayTest__IntOverloadGuardsNegative() {
    let mut array = values(&[1]);
    assert!(array.set_value(-1, 99).is_err());
    assert!(array.insert_before(-1, 99).is_err());
    assert!(array.insert_after(-1, 99).is_err());
    assert!(array.emplace_value(-1, || 99).is_err());
}
#[test]
fn NCollection_DynamicArrayTest__IntAndSizeTOverloadsAgree() {
    let mut integer = DynamicArray::default();
    let mut size = DynamicArray::default();
    for index in 0..20 {
        integer.set_value(index, index * 3).unwrap();
        size.set_value(index as usize, index * 3).unwrap();
    }
    assert_eq!(integer.len(), size.len());
    for index in 0..20 {
        assert_eq!(
            integer.value(index).unwrap(),
            size.value(index as usize).unwrap()
        );
    }
}
#[test]
fn NCollection_DynamicArrayTest__SetValue_OutOfOrderIndices_CorrectValues() {
    let mut array = DynamicArray::default();
    array.set_value(0, 1).unwrap();
    array.set_value(2, 500).unwrap();
    array.set_value(1, 2).unwrap();
    equal(&array, &[1, 2, 500]);
}
