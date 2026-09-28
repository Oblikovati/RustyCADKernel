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
fn TCollection_ExtendedStringTest__FirstLocationInSet_FindsCharacter() {
    let string = ExtendedString::new("Hello World");
    let set = ExtendedString::new("aeiou");
    assert_eq!(string.first_location_in_set(&set, 1, string.len()), 2);
}
#[test]
fn TCollection_ExtendedStringTest__FirstLocationInSet_NotFound() {
    let string = ExtendedString::new("BCDFG");
    let set = ExtendedString::new("aeiou");
    assert_eq!(string.first_location_in_set(&set, 1, string.len()), 0);
}
#[test]
fn TCollection_ExtendedStringTest__FirstLocationInSet_EmptySet() {
    let string = ExtendedString::new("Hello");
    let set = ExtendedString::new("");
    assert_eq!(string.first_location_in_set(&set, 1, string.len()), 0);
}
#[test]
fn TCollection_ExtendedStringTest__FirstLocationNotInSet_FindsCharacter() {
    let string = ExtendedString::new("aaaHello");
    let set = ExtendedString::new("a");
    assert_eq!(string.first_location_not_in_set(&set, 1, string.len()), 4);
}
#[test]
fn TCollection_ExtendedStringTest__FirstLocationNotInSet_AllInSet() {
    let string = ExtendedString::new("aaa");
    let set = ExtendedString::new("a");
    assert_eq!(string.first_location_not_in_set(&set, 1, string.len()), 0);
}
#[test]
fn TCollection_ExtendedStringTest__FirstLocationNotInSet_EmptySet() {
    let string = ExtendedString::new("Hello");
    let set = ExtendedString::new("");
    assert_eq!(string.first_location_not_in_set(&set, 1, string.len()), 1);
}
#[test]
fn TCollection_ExtendedStringTest__IntegerValue_ValidInteger() {
    let string = ExtendedString::new("12345");
    assert!(string.is_integer_value());
    assert_eq!(string.integer_value().unwrap(), 12345);
}
#[test]
fn TCollection_ExtendedStringTest__IntegerValue_NegativeInteger() {
    let string = ExtendedString::new("-42");
    assert!(string.is_integer_value());
    assert_eq!(string.integer_value().unwrap(), -42);
}
#[test]
fn TCollection_ExtendedStringTest__IntegerValue_NotAnInteger() {
    assert!(!ExtendedString::new("Hello").is_integer_value());
}
#[test]
fn TCollection_ExtendedStringTest__IntegerValue_EmptyString() {
    let string = ExtendedString::default();
    assert!(!string.is_integer_value());
    assert_eq!(string.integer_value().unwrap(), 0);
}
#[test]
fn TCollection_ExtendedStringTest__RealValue_ValidReal() {
    let string = ExtendedString::new("3.14159");
    assert!(string.is_real_value(true));
    assert!((string.real_value().unwrap() - 3.14159).abs() < 0.00001);
}
#[test]
fn TCollection_ExtendedStringTest__RealValue_Integer() {
    let string = ExtendedString::new("42");
    assert!(string.is_real_value(true));
    assert!((string.real_value().unwrap() - 42.0).abs() < 0.00001);
}
#[test]
fn TCollection_ExtendedStringTest__RealValue_NotAReal() {
    assert!(!ExtendedString::new("Hello").is_real_value(true));
}
#[test]
fn TCollection_ExtendedStringTest__RealValue_EmptyString() {
    let string = ExtendedString::default();
    assert!(!string.is_real_value(false));
    assert!(string.real_value().unwrap().abs() < 0.00001);
}
#[test]
fn TCollection_ExtendedStringTest__IsSameString_CaseSensitiveMatch() {
    let first = ExtendedString::new("Hello");
    let second = ExtendedString::new("Hello");
    assert!(first.is_same_string(&second, true));
}
#[test]
fn TCollection_ExtendedStringTest__IsSameString_CaseSensitiveMismatch() {
    let first = ExtendedString::new("Hello");
    let second = ExtendedString::new("hello");
    assert!(!(first.is_same_string(&second, true)));
}
#[test]
fn TCollection_ExtendedStringTest__IsSameString_CaseInsensitiveMatch() {
    let first = ExtendedString::new("Hello");
    let second = ExtendedString::new("HELLO");
    assert!(first.is_same_string(&second, false));
}
#[test]
fn TCollection_ExtendedStringTest__IsSameString_DifferentLengths() {
    let first = ExtendedString::new("Hello");
    let second = ExtendedString::new("Hello World");
    assert!(!(first.is_same_string(&second, false)));
}
