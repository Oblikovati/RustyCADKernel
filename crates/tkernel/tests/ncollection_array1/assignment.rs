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

fn filled_array(lower: i32, upper: i32) -> Array1<'static, i32> {
    let mut array = Array1::with_bounds(lower, upper).unwrap();
    for index in lower..=upper { array[index] = index * 10; }
    array
}

#[test]
fn NCollection_Array1Test__AssignmentValue() {
    let array = filled_array(1, 5);
    for i in array.lower()..=array.upper() { assert_eq!(i * 10, array[i]); }
    for i in array.lower()..=array.upper() { assert_eq!(*array.value(i).unwrap(), array[i]); }
}

#[test]
fn NCollection_Array1Test__CopyConstructor() {
    let mut source = filled_array(1, 5);
    let copy = source.clone();
    assert_eq!(source.len(), copy.len());
    assert_eq!(source.lower(), copy.lower());
    assert_eq!(source.upper(), copy.upper());
    for i in source.lower()..=source.upper() { assert_eq!(source[i], copy[i]); }
    source[3] = 999;
    assert_ne!(source[3], copy[3]);
}

#[test]
fn NCollection_Array1Test__AssignmentOperator() {
    let source = filled_array(1, 5);
    let mut dest = Array1::with_bounds(11, 15).unwrap();
    dest.assign(&source);
    assert_eq!(source.len(), dest.len());
    assert_eq!(source.lower(), dest.lower());
    assert_eq!(source.upper(), dest.upper());
    for i in source.lower()..=source.upper() { assert_eq!(source[i], dest[i]); }
}

#[test]
fn NCollection_Array1Test__AssignmentOperatorDifferentSize() {
    let source = filled_array(5, 8);
    let mut dest = Array1::with_bounds(1, 2).unwrap();
    dest.assign(&source);
    assert_eq!(4, dest.len());
    assert_eq!(5, dest.lower());
    assert_eq!(8, dest.upper());
    for i in source.lower()..=source.upper() { assert_eq!(source[i], dest[i]); }
}

#[test]
fn NCollection_Array1Test__AssignmentOperatorReusesSameSizeOwnedStorage() {
    let mut dest = Array1::with_bounds(1, 6).unwrap();
    let initial = dest.as_slice().as_ptr();
    let source = filled_array(20, 25);
    dest.assign(&source);
    assert_eq!(initial, dest.as_slice().as_ptr());
    assert_eq!(20, dest.lower());
    assert_eq!(25, dest.upper());
    for i in source.lower()..=source.upper() { assert_eq!(source[i], dest[i]); }
}

#[test]
fn NCollection_Array1Test__AssignmentOperatorReusesSameSizeExternalBuffer() {
    let mut buffer = [0; 3];
    let mut dest = Array1::from_slice(&mut buffer);
    let source = filled_array(7, 9);
    dest.assign(&source);
    assert!(!dest.is_deletable());
    assert_eq!(7, dest.lower());
    assert_eq!(9, dest.upper());
    assert_eq!(70, buffer[0]);
    assert_eq!(80, buffer[1]);
    assert_eq!(90, buffer[2]);
}

#[test]
fn NCollection_Array1Test__AssignmentOperatorDetachesDifferentSizeExternalBuffer() {
    let mut buffer = [1, 2, 3];
    let pointer = buffer.as_ptr();
    let mut dest = Array1::from_slice(&mut buffer);
    let source = filled_array(7, 10);
    dest.assign(&source);
    assert!(dest.is_deletable());
    assert_ne!(pointer, dest.as_slice().as_ptr());
    assert_eq!(7, dest.lower());
    assert_eq!(10, dest.upper());
    for i in source.lower()..=source.upper() { assert_eq!(source[i], dest[i]); }
    // Inspect the original buffer after the safe view's last use.
    assert_eq!(1, buffer[0]);
    assert_eq!(2, buffer[1]);
    assert_eq!(3, buffer[2]);
}

#[test]
fn NCollection_Array1Test__AssignmentOperatorEmptySource() {
    let mut dest = Array1::<i32>::with_bounds(1, 5).unwrap();
    dest.assign(&Array1::default());
    assert_eq!(0, dest.len());
    assert_eq!(1, dest.lower());
    assert!(dest.is_empty());
}

#[test]
fn NCollection_Array1Test__CopyValuesPreservesBounds() {
    let source = filled_array(1, 5);
    let mut dest = Array1::with_bounds(11, 15).unwrap();
    dest.copy_values(&source).unwrap();
    assert_eq!(11, dest.lower());
    assert_eq!(15, dest.upper());
    for i in 0..dest.len() as i32 { assert_eq!(source[source.lower()+i], dest[dest.lower()+i]); }
}

#[test]
fn NCollection_Array1Test__CopyValuesDifferentSize() {
    let mut dest = Array1::<i32>::with_bounds(1, 5).unwrap();
    let source = Array1::with_bounds(1, 4).unwrap();
    assert_eq!(dest.copy_values(&source), Err(rustycad_tkernel::ncollection::ArrayError::DimensionMismatch { expected: 5, actual: 4 }));
}

#[test]
fn NCollection_Array1Test__Move() {
    let mut source = filled_array(1, 5);
    let mut dest = Array1::with_bounds(11, 15).unwrap();
    dest.move_from(&mut source);
    assert_eq!(5, dest.len());
    assert_eq!(1, dest.lower());
    assert_eq!(5, dest.upper());
    assert_eq!(0, source.len());
    assert_eq!(1, source.lower());
}
