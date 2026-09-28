//! Comparisons, searches, tokens, element access and numeric parsing.
use super::{class, string, units};
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn span_comparisons_answer_null_and_negative_like_occt() {
    let text = string("Hello");
    assert!(!text.is_equal_span(None, 5) && text.is_different_span(None, 5));
    assert!(ExtendedString::default().is_equal_span(None, 0));
    assert!(!text.is_less_span(None, 0) && !text.is_less_span(Some(&[]), -1));
    assert!(text.is_greater_span(None, 0) && text.is_greater_span(Some(&[]), i32::MIN));
    assert!(!ExtendedString::default().is_greater_span(None, 0));
    assert!(ExtendedString::default().is_greater_span(None, 2));
    assert!(text.starts_with_span(None, 0) && !text.starts_with_span(None, 1));
    assert!(text.ends_with_span(Some(&units("lo")), 2));
    assert!(!text.ends_with_span(Some(&units("Hello!")), 6));
}
#[test]
fn span_cores_read_only_the_units_occt_reads() {
    let text = string("ab");
    // OCCT compares min(2, 9) units and stops at the first difference.
    assert!(text.is_less_span(Some(&units("b")), 9));
    // The length differs from the string, so IsEqual reads nothing.
    assert!(!text.is_equal_span(Some(&[]), 3));
    assert!(!text.starts_with_span(Some(&[]), 3));
    assert_eq!(text.search_span(Some(&[]), 3), -1);
}
#[test]
#[should_panic(expected = "code-unit span")]
fn span_cores_panic_where_occt_would_read_past_the_buffer() {
    let _ = string("ab").is_equal_span(Some(&units("a")), 2);
}
#[test]
fn whole_comparisons_order_unsigned_units() {
    let text = string("ab");
    assert!(text.is_less(units("b")) && text.is_less(string("abc")));
    assert!(text.is_greater(units("aa")) && string("\u{FFFF}").is_greater(string("a")));
    assert!(ExtendedString::is_equal_strings(&text, &string("ab")));
    assert!(text.starts_with(units("")) && text.ends_with(string("b")));
    assert!(text.is_same_string(&string("AB"), false) && !text.is_same_string(&string("AB"), true));
    assert!(!string("é").is_same_string(&string("É"), false));
}
#[test]
fn searches_return_one_based_positions() {
    let text = string("abcabcab");
    assert_eq!(
        (text.search(units("ab")), text.search_from_end(units("ab"))),
        (1, 7)
    );
    assert_eq!(text.search(units("")), -1);
    assert_eq!(text.search(units("abcabcabc")), -1);
    assert_eq!(text.search_span(Some(&units("ca")), 2), 3);
    assert_eq!(text.search_from_end_span(Some(&units("ca")), 2), 6);
    assert_eq!(text.search_span(None, 1), -1);
}
#[test]
fn token_follows_the_native_scan() {
    let text = string("1234; test:message   , value");
    let separators = units("; :,");
    assert_eq!(text.token(&separators, 4), string("value"));
    assert_eq!(text.token(&separators, 2), string("test"));
    assert_eq!(text.token(&separators, 9), ExtendedString::default());
    assert_eq!(text.token(&separators, 0), ExtendedString::default());
    assert!(text.token(&[], 1).is_empty());
    assert!(text.token(&[0, 32], 1).is_empty());
    assert_eq!(
        ExtendedString::from_raw(&[32, 97, 0, 98]).token(&[32], 1),
        string("a")
    );
}
#[test]
fn sub_string_and_set_locators_check_bounds() {
    let text = string("abcdefg");
    assert_eq!(text.sub_string(3, 6).unwrap(), string("cdef"));
    assert_eq!(class(text.sub_string(0, 2)), "Standard_OutOfRange");
    assert_eq!(class(text.sub_string(3, 8)), "Standard_OutOfRange");
    assert_eq!(class(text.sub_string(4, 3)), "Standard_OutOfRange");
    let set = string("dx");
    assert_eq!(text.first_location_in_set(&set, 1, 7).unwrap(), 4);
    assert_eq!(
        text.first_location_not_in_set(&string("ab"), 1, 7).unwrap(),
        3
    );
    assert_eq!(text.first_location_in_set(&set, 5, 2).unwrap(), 0);
    assert_eq!(
        text.first_location_not_in_set(&string(""), 3, 2).unwrap(),
        0
    );
    assert_eq!(
        text.first_location_not_in_set(&string(""), 2, 3).unwrap(),
        2
    );
    assert_eq!(
        class(text.first_location_in_set(&set, 1, 8)),
        "Standard_OutOfRange"
    );
    assert_eq!(
        class(text.first_location_not_in_set(&set, 0, 1)),
        "Standard_OutOfRange"
    );
}
#[test]
fn value_length_and_ascii_range() {
    let text = ExtendedString::from_raw(&[0xFF, 0x100]);
    assert_eq!((text.value(1).unwrap(), text.length()), (0xFF, 2));
    assert_eq!(class(text.value(0)), "Standard_OutOfRange");
    assert_eq!(class(text.value(3)), "Standard_OutOfRange");
    assert!(!text.is_ascii() && ExtendedString::from_char(0xFF).is_ascii());
}
#[test]
fn numeric_parsing_goes_through_utf8() {
    assert_eq!(string("  -42abc").integer_value().unwrap(), -42);
    assert!(string("12").is_integer_value() && !string("1.5").is_integer_value());
    assert_eq!(string("3.5e2x").real_value().unwrap(), 350.0);
    assert!(string("1.5").is_real_value(true) && !string("3.5e2x").is_real_value(true));
    assert_eq!(ExtendedString::default().integer_value().unwrap(), 0);
    assert!(!ExtendedString::default().is_real_value(false));
    assert_eq!(class(string("x").integer_value()), "Standard_NumericError");
    assert_eq!(
        class(string("\u{663}").real_value()),
        "Standard_NumericError"
    );
}
