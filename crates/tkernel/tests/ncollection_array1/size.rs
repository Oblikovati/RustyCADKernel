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
fn NCollection_Array1Test__SizeConstructor_AllocatesCorrectly() {
    let array = Array1::<i32>::with_size(10);
    assert_eq!(10, array.len());
    assert_eq!(0, array.lower());
    assert_eq!(9, array.upper());
    assert!(!array.is_empty());
    assert!(array.is_deletable());
}

#[test]
fn NCollection_Array1Test__SizeConstructor_ZeroSize() {
    let array = Array1::<i32>::with_size(0);
    assert_eq!(0, array.len());
    assert!(array.is_empty());
}

#[test]
fn NCollection_Array1Test__SizeConstructor_AtAccess() {
    let mut array = Array1::<i32>::with_size(5);
    for i in 0..5 { *array.change_at(i).unwrap() = i as i32 * 10; }
    for i in 0..5 { assert_eq!(i as i32 * 10, *array.at(i).unwrap()); }
}

#[test]
fn NCollection_Array1Test__SizeConstructor_IteratorAccess() {
    let mut array = Array1::<i32>::with_size(6);
    for (index, value) in array.iter_mut().enumerate() { *value = index as i32; }
    let mut check = 0;
    for value in &array { assert_eq!(check, *value); check += 1; }
    assert_eq!(6, check);
}

#[test]
fn NCollection_Array1Test__SizeConstructor_LegacyValueOperator() {
    let mut array = Array1::<i32>::with_size(4);
    for i in 0..4 { array[i] = i + 100; }
    for i in 0..4 { assert_eq!(i + 100, array[i]); }
}

#[test]
fn NCollection_Array1Test__SizeConstructor_BufferReuse_NotOwner() {
    let mut buffer = [10, 20, 30, 40, 50, 60, 70, 80];
    let array = Array1::from_slice(&mut buffer);
    assert!(!array.is_deletable());
    assert_eq!(8, array.len());
    assert_eq!(0, array.lower());
    assert_eq!(7, array.upper());
}

#[test]
fn NCollection_Array1Test__SizeConstructor_BufferReuse_DataPreserved() {
    let mut buffer = [1, 2, 3, 4, 5];
    let array = Array1::from_slice(&mut buffer);
    for i in 0..5 { assert_eq!(i as i32 + 1, *array.at(i).unwrap()); }
}

#[test]
fn NCollection_Array1Test__SizeConstructor_BufferReuse_WritesGoToBuffer() {
    let mut buffer = [0; 4];
    {
        let mut array = Array1::from_slice(&mut buffer);
        for i in 0..4 { *array.change_at(i).unwrap() = i as i32 + 7; }
    }
    for (i, value) in buffer.iter().enumerate() { assert_eq!(i as i32 + 7, *value); }
}

#[test]
fn NCollection_Array1Test__SizeConstructor_StdVectorBufferReuse() {
    let mut vector = vec![1, 2, 3, 4];
    let size = vector.len();
    let mut array = Array1::from_slice(&mut vector);
    assert!(!array.is_deletable());
    assert_eq!(size, array.len());
    *array.change_at(2).unwrap() = 30;
    assert_eq!(30, vector[2]);
}

#[test]
fn NCollection_Array1Test__SizeConstructor_Resize_Grow() {
    let mut array = Array1::with_size(3);
    *array.change_at(0).unwrap() = 10;
    *array.change_at(1).unwrap() = 20;
    *array.change_at(2).unwrap() = 30;
    array.resize(5, true);
    assert_eq!(5, array.len());
    assert_eq!(10, *array.at(0).unwrap());
    assert_eq!(20, *array.at(1).unwrap());
    assert_eq!(30, *array.at(2).unwrap());
}

#[test]
fn NCollection_Array1Test__SizeConstructor_Resize_Shrink() {
    let mut array = Array1::with_size(5);
    for i in 0..5 { *array.change_at(i).unwrap() = i as i32; }
    array.resize(3, true);
    assert_eq!(3, array.len());
    assert_eq!(0, *array.at(0).unwrap());
    assert_eq!(1, *array.at(1).unwrap());
    assert_eq!(2, *array.at(2).unwrap());
}

#[test]
fn NCollection_Array1Test__SizeConstructor_Resize_NoData() {
    let mut array = Array1::with_size(4);
    *array.change_at(0).unwrap() = 99;
    array.resize(6, false);
    assert_eq!(6, array.len());
    assert!(array.is_deletable());
}

#[test]
fn NCollection_Array1Test__SizeConstructor_MoveSemantics() {
    let mut source = Array1::with_size(3);
    *source.change_at(0).unwrap() = 1;
    *source.change_at(1).unwrap() = 2;
    *source.change_at(2).unwrap() = 3;
    let dest = std::mem::take(&mut source);
    assert_eq!(3, dest.len());
    assert_eq!(1, *dest.at(0).unwrap());
    assert_eq!(2, *dest.at(1).unwrap());
    assert_eq!(3, *dest.at(2).unwrap());
    assert_eq!(0, source.len());
    assert!(source.is_empty());
}
