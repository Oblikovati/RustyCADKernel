#[test]
fn TCollection_ExtendedStringTest__IsEqual_NullPointer() {
    let string = ExtendedString::new("Hello");
    assert!(!(string.is_equal_span(None, 5)));
}
#[test]
fn TCollection_ExtendedStringTest__IsEqual_NegativeLength() {
    let string = ExtendedString::new("Hello");
    let other = units("Hello");
    assert!(!(string.is_equal_span(Some(other.as_slice()), -1)));
}
#[test]
fn TCollection_ExtendedStringTest__IsDifferent_NullPointer() {
    let string = ExtendedString::new("Hello");
    assert!(string.is_different_span(None, 5));
}
#[test]
fn TCollection_ExtendedStringTest__IsDifferent_NegativeLength() {
    let string = ExtendedString::new("Hello");
    let other = units("Hello");
    assert!(string.is_different_span(Some(other.as_slice()), -1));
}
#[test]
fn TCollection_ExtendedStringTest__IsLess_NullPointer() {
    let string = ExtendedString::new("Hello");
    assert!(!(string.is_less_span(None, 5)));
}
#[test]
fn TCollection_ExtendedStringTest__IsLess_NegativeLength() {
    let string = ExtendedString::new("Hello");
    let other = units("Hello");
    assert!(!(string.is_less_span(Some(other.as_slice()), -1)));
}
#[test]
fn TCollection_ExtendedStringTest__IsGreater_NullPointer() {
    let string = ExtendedString::new("Hello");
    assert!(string.is_greater_span(None, 5));
}
#[test]
fn TCollection_ExtendedStringTest__IsGreater_NegativeLength() {
    let string = ExtendedString::new("Hello");
    let other = units("Hello");
    assert!(string.is_greater_span(Some(other.as_slice()), -1));
}
#[test]
fn TCollection_ExtendedStringTest__StartsWith_NullPointer() {
    let string = ExtendedString::new("Hello");
    assert!(!(string.starts_with_span(None, 3)));
}
#[test]
fn TCollection_ExtendedStringTest__StartsWith_NegativeLength() {
    let string = ExtendedString::new("Hello");
    let other = units("Hel");
    assert!(!(string.starts_with_span(Some(other.as_slice()), -1)));
}
#[test]
fn TCollection_ExtendedStringTest__EndsWith_NullPointer() {
    let string = ExtendedString::new("Hello");
    assert!(!(string.ends_with_span(None, 3)));
}
#[test]
fn TCollection_ExtendedStringTest__EndsWith_NegativeLength() {
    let string = ExtendedString::new("Hello");
    let other = units("llo");
    assert!(!(string.ends_with_span(Some(other.as_slice()), -1)));
}
#[test]
fn TCollection_ExtendedStringTest__IsEqual_ZeroLengthNullPointer() {
    assert!(ExtendedString::default().is_equal_span(None, 0));
}
#[test]
fn TCollection_ExtendedStringTest__StartsWith_ZeroLength() {
    assert!(ExtendedString::new("Hello").starts_with_span(None, 0));
}
#[test]
fn TCollection_ExtendedStringTest__EndsWith_ZeroLength() {
    assert!(ExtendedString::new("Hello").ends_with_span(None, 0));
}
#[test]
fn TCollection_ExtendedStringTest__StartsWith_NoMatchLongerPrefix() {
    let string = ExtendedString::new("hello");
    let other = ExtendedString::new("help");
    assert!(!(string.starts_with(&other)));
}
#[test]
fn TCollection_ExtendedStringTest__StartsWith_Match() {
    let string = ExtendedString::new("hello");
    let other = ExtendedString::new("he");
    assert!(string.starts_with(&other));
}
#[test]
fn TCollection_ExtendedStringTest__EndsWith_NoMatchMiddlePart() {
    let string = ExtendedString::new("hello");
    let other = ExtendedString::new("ll");
    assert!(!(string.ends_with(&other)));
}
#[test]
fn TCollection_ExtendedStringTest__EndsWith_Match() {
    let string = ExtendedString::new("hello");
    let other = ExtendedString::new("lo");
    assert!(string.ends_with(&other));
}
