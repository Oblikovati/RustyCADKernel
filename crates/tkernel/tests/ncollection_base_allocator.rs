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

//! Original allocation scenarios retain allocator ownership through Rust values.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray};
#[derive(Clone, Debug)]
struct TestStruct(i32, f64, char);
impl PartialEq for TestStruct {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && (self.1 - other.1).abs() < 1e-10 && self.2 == other.2
    }
}
fn check(value: &TestStruct, expected: &TestStruct) {
    assert_eq!(value.0, expected.0);
    assert_eq!(value.1, expected.1);
    assert_eq!(value.2, expected.2);
}
#[test]
fn NCollection_BaseAllocatorTest__DefaultInstance() {
    let allocator = BaseAllocator::common();
    assert_eq!(allocator, BaseAllocator::common());
}
#[test]
fn NCollection_BaseAllocatorTest__Allocate() {
    let allocator = BaseAllocator::common();
    let first = allocator.allocate(10);
    let second = allocator.allocate(100);
    let third = allocator.allocate(1000);
    assert_ne!(first.as_slice().as_ptr(), second.as_slice().as_ptr());
    assert_ne!(first.as_slice().as_ptr(), third.as_slice().as_ptr());
    assert_ne!(second.as_slice().as_ptr(), third.as_slice().as_ptr());
    drop((first, second, third));
}
#[test]
fn NCollection_BaseAllocatorTest__AllocateStruct() {
    let allocator = BaseAllocator::common();
    let value = allocator.allocate_value(TestStruct(42, 3.14159, 'Z'));
    check(&value, &TestStruct(42, 3.14159, 'Z'));
    drop(value);
}
#[test]
fn NCollection_BaseAllocatorTest__AllocateArray() {
    let allocator = BaseAllocator::common();
    let array = allocator.allocate_array(5, |i| {
        TestStruct(i as i32, i as f64 * 1.5, (b'A' + i as u8) as char)
    });
    for (i, value) in array.as_slice().iter().enumerate() {
        check(
            value,
            &TestStruct(i as i32, i as f64 * 1.5, (b'A' + i as u8) as char),
        );
    }
    drop(array);
}
#[test]
fn NCollection_BaseAllocatorTest__UsageWithVector() {
    let mut vector = DynamicArray::default();
    let expected = [
        TestStruct(10, 1.0, 'X'),
        TestStruct(20, 2.0, 'Y'),
        TestStruct(30, 3.0, 'Z'),
    ];
    for value in &expected {
        vector.append(value.clone());
    }
    assert_eq!(vector.len(), 3);
    for (value, expected) in vector.iter().zip(&expected) {
        check(value, expected);
    }
    let mut custom = DynamicArray::with_allocator(5, Some(BaseAllocator::common()));
    custom.append(TestStruct(40, 4.0, 'P'));
    custom.append(TestStruct(50, 5.0, 'Q'));
    assert_eq!(custom.len(), 2);
    check(&custom[0], &TestStruct(40, 4.0, 'P'));
    check(&custom[1], &TestStruct(50, 5.0, 'Q'));
}
#[test]
fn NCollection_BaseAllocatorTest__CopyAndMove() {
    let mut original = DynamicArray::with_allocator(5, Some(BaseAllocator::common()));
    original.append(TestStruct(10, 1.0, 'A'));
    original.append(TestStruct(20, 2.0, 'B'));
    let copy = original.clone();
    assert_eq!(copy.len(), 2);
    assert_eq!(copy[0], TestStruct(10, 1.0, 'A'));
    assert_eq!(copy[1], TestStruct(20, 2.0, 'B'));
    let mut destination = DynamicArray::with_allocator(5, Some(BaseAllocator::common()));
    destination.assign(&original, false);
    assert_eq!(destination.len(), 2);
    assert_eq!(destination[0], TestStruct(10, 1.0, 'A'));
    assert_eq!(destination[1], TestStruct(20, 2.0, 'B'));
}
#[test]
fn NCollection_BaseAllocatorTest__BigAllocation() {
    let mut bytes = BaseAllocator::common().allocate(1024 * 1024);
    bytes.as_mut_slice().fill(0xAB);
    assert!(bytes.as_slice().iter().all(|value| *value == 0xAB));
    drop(bytes);
}
