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
fn TCollection_AsciiStringTest__Insert_BasicCases() {
    let mut string = AsciiString::new("HelloWorld");
    string.insert(1, "Start");
    text(&string, "StartHelloWorld");
    let mut second = AsciiString::new("AC");
    second.insert(2, "B");
    text(&second, "ABC");
    let mut third = AsciiString::new("Hello");
    third.insert(6, " World");
    text(&third, "Hello World");
}
#[test]
fn TCollection_AsciiStringTest__Insert_EmptyStrings() {
    let mut string = AsciiString::default();
    string.insert(1, "First");
    text(&string, "First");
    assert_eq!(string.len(), 5);
    let mut second = AsciiString::new("Test");
    second.insert(3, "");
    text(&second, "Test");
    assert_eq!(second.len(), 4);
}
#[test]
fn TCollection_AsciiStringTest__Insert_SelfReference() {
    let mut string = AsciiString::new("XYZ");
    string.insert_self(1, 0..string.len());
    text(&string, "XYZXYZ");
    assert_eq!(string.len(), 6);
    let mut second = AsciiString::new("ABCD");
    second.insert_self(2, 1..second.len());
    text(&second, "ABCDBCD");
    assert_eq!(second.len(), 7);
}
#[test]
fn TCollection_AsciiStringTest__Insert_WithCharacter() {
    let mut string = AsciiString::new("Hllo");
    string.insert(2, b'e');
    text(&string, "Hello");
    assert_eq!(string.len(), 5);
    let mut second = AsciiString::new("ello");
    second.insert(1, b'H');
    text(&second, "Hello");
}
#[test]
fn TCollection_AsciiStringTest__Insert_BoundaryConditions() {
    let original = AsciiString::new("Test");
    let mut first = original.clone();
    first.insert(1, ">");
    text(&first, ">Test");
    let mut second = original.clone();
    second.insert(5, "<");
    text(&second, "Test<");
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapLeftSide() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(4, 0..3);
    text(&string, "ABCABCDEFGH");
    assert_eq!(string.len(), 11);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapRightSide() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(2, 3..8);
    text(&string, "ADEFGHBCDEFGH");
    assert_eq!(string.len(), 13);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapInsideShiftWindow() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(2, 2..5);
    text(&string, "ACDEBCDEFGH");
    assert_eq!(string.len(), 11);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapStartsBeforeShiftWindow() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(4, 1..5);
    text(&string, "ABCBCDEDEFGH");
    assert_eq!(string.len(), 12);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapAtInsertionPoint() {
    let mut string = AsciiString::new("ABCDEFGH");
    string.insert_self(4, 3..6);
    text(&string, "ABCDEFDEFGH");
    assert_eq!(string.len(), 11);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapComplex1() {
    let mut string = AsciiString::new("123456789");
    string.insert_self(1, 3..6);
    text(&string, "456123456789");
    assert_eq!(string.len(), 12);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapComplex2() {
    let mut string = AsciiString::new("XYZABC");
    string.insert_self(4, 0..2);
    text(&string, "XYZXYABC");
    assert_eq!(string.len(), 8);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapSpansEntireShiftWindow() {
    let mut string = AsciiString::new("ABCDEFGHIJK");
    string.insert_self(5, 1..8);
    text(&string, "ABCDBCDEFGHEFGHIJK");
    assert_eq!(string.len(), 18);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapSingleChar() {
    let mut string = AsciiString::new("ABC");
    string.insert_self(1, 1..2);
    text(&string, "BABC");
    assert_eq!(string.len(), 4);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapEndToBeginning() {
    let mut string = AsciiString::new("HELLO");
    string.insert_self(1, 3..5);
    text(&string, "LOHELLO");
    assert_eq!(string.len(), 7);
}
#[test]
fn TCollection_AsciiStringTest__Insert_OverlapLargeString() {
    let mut string = AsciiString::filled(50, b'A');
    string.set_value(25, "MARKER");
    string.insert_self(10, 20..30);
    assert_eq!(string.len(), 60);
    assert!(contains(&string, "MARKER"));
}
