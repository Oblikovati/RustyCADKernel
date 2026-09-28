//! Append, insert, replace, remove, split, copy and ownership edits.
use super::{class, string, units};
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn append_forms_convert_their_input() {
    let mut text = string("v=");
    text.assign_cat(42);
    text += b'!';
    text.assign_cat(0_u16);
    text += units("ok").as_slice();
    assert_eq!(text.to_utf8(), "v=42!ok");
    assert_eq!((&text + 2.5).to_utf8(), "v=42!ok2.5");
    assert_eq!(text.cat(0_u8), text);
}
#[test]
fn span_appends_ignore_null_or_non_positive_lengths() {
    let mut text = string("ab");
    text.assign_cat_span(None, 5);
    text.assign_cat_span(Some(&[99]), -1);
    text.prepend_span(Some(&[99, 100]), 1);
    assert_eq!(text, string("cab"));
    assert_eq!(text.cat_span(Some(&[100, 101]), 1).unwrap(), string("cabd"));
    assert_eq!(text.cat_span(None, 0).unwrap(), text);
    assert_eq!(class(text.cat_span(Some(&[]), -4)), "Standard_OutOfRange");
    assert_eq!(class(text.cat_span(Some(&[]), -1)), "Standard_OutOfRange");
}
#[test]
fn prepend_and_self_ranges() {
    let mut text = string("World");
    text.prepend(string("Hello "));
    text.prepend(units("> "));
    assert_eq!(text.to_utf8(), "> Hello World");
    let mut aliased = string("abc");
    aliased.assign_cat_self(1..3);
    aliased.prepend_self(0..1);
    assert_eq!(aliased, string("aabcbc"));
}
#[test]
fn insert_checks_too_big_before_below_one() {
    let mut text = string("ac");
    text.insert_char(2, 0).unwrap();
    assert_eq!(text.as_utf16(), &[97, 0, 99]);
    text.insert(4, string("!")).unwrap();
    text.insert(1, units(">").as_slice()).unwrap();
    assert_eq!(text.as_utf16(), &[62, 97, 0, 99, 33]);
    assert_eq!(class(text.insert_char(7, 65)), "Standard_OutOfRange");
    assert_eq!(class(text.insert(0, string("x"))), "Standard_OutOfRange");
    assert_eq!(class(text.insert_span(9, None, 0)), "Standard_OutOfRange");
    text.insert_span(1, None, 3).unwrap();
    text.insert_span(1, Some(&[120, 121]), 1).unwrap();
    assert_eq!(text.as_utf16()[0], 120);
}
#[test]
fn set_value_replaces_and_grows() {
    let mut text = string("Garbake");
    text.set_value(6, 103).unwrap();
    assert_eq!(text, string("Garbage"));
    assert_eq!(class(text.set_value(8, 65)), "Standard_OutOfRange");
    text.set_value_units(6, units("XYZ")).unwrap();
    assert_eq!(text, string("GarbaXYZ"));
    text.set_value_units(9, string("!")).unwrap();
    assert_eq!(class(text.set_value_units(11, [65])), "Standard_OutOfRange");
    assert_eq!(
        class(text.set_value_span(0, None, 0)),
        "Standard_OutOfRange"
    );
    text.set_value_span(1, Some(&[103, 104]), 1).unwrap();
    assert_eq!(text, string("garbaXYZ!"));
}
#[test]
fn remove_trunc_and_split_bounds() {
    let mut text = string("Hello");
    text.remove(2, 2).unwrap();
    assert_eq!(text, string("Hlo"));
    assert_eq!(class(text.remove(1, 4)), "Standard_OutOfRange");
    assert_eq!(class(text.remove(1, -1)), "Standard_OutOfRange");
    // OCCT's int sum overflows here (undefined); the port refuses it.
    assert_eq!(class(text.remove(1, i32::MAX)), "Standard_OutOfRange");
    text.remove(3, 0).unwrap();
    assert_eq!(class(text.trunc(4)), "Standard_OutOfRange");
    text.trunc(2).unwrap();
    assert_eq!(text, string("Hl"));
    let mut parts = ExtendedString::from_raw(&[97, 98, 0, 99]);
    assert_eq!(parts.split(1).unwrap(), string("b"));
    assert_eq!(parts, string("a"));
    assert_eq!(class(parts.split(1)), "Standard_OutOfRange");
}
#[test]
fn remove_all_change_all_and_copies() {
    let mut text = ExtendedString::from_raw(&[97, 0, 98, 0]);
    text.remove_all(0);
    text.change_all(98, 99);
    assert_eq!(text, string("ac"));
    text.copy(units("xyz"));
    text.copy_span(Some(&[65, 66]), 1);
    assert_eq!(text, string("A"));
    text.copy_span(None, 1);
    assert!(text.is_empty());
    let (mut first, mut second) = (string("1"), string("2"));
    first.swap(&mut second);
    second.move_assign(&mut first);
    assert_eq!((first.is_empty(), second.clone()), (true, string("2")));
    second.clear();
    assert!(second.is_empty());
}
#[test]
fn justify_errors_name_the_member() {
    let mut text = string("ab");
    let failure = text.center(-3, 45).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains("Center") && failure.message().contains("-3"));
    text.center(5, 45).unwrap();
    assert_eq!(text, string("-ab--"));
}
