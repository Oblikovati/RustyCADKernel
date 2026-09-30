#[test]
fn TCollection_AsciiStringTest__Search_BasicCases() {
    let string = AsciiString::new("This is a test string");
    for (query, expected) in [
        ("This", 1),
        ("test", 11),
        ("string", 16),
        ("xyz", -1),
        ("notfound", -1),
    ] {
        assert_eq!(string.search(query), expected);
    }
}
#[test]
fn TCollection_AsciiStringTest__Search_EdgeCases() {
    let string = AsciiString::new("abcabc");
    for (query, expected) in [("abc", 1), ("a", 1), ("b", 2), ("c", 3), ("", -1)] {
        assert_eq!(string.search(query), expected);
    }
    assert_eq!(AsciiString::default().search("test"), -1);
}
#[test]
fn TCollection_AsciiStringTest__Search_RepeatedPattern() {
    assert_eq!(AsciiString::new("aaaaaaa").search("aaa"), 1);
    let string = AsciiString::new("abcabcabc");
    assert_eq!(string.search("abc"), 1);
    assert_eq!(string.search("abcabc"), 1);
}
#[test]
fn TCollection_AsciiStringTest__SearchFromEnd_BasicCases() {
    let string = AsciiString::new("This is a test string with test");
    assert_eq!(string.search_from_end("test"), 28);
    assert_eq!(string.search_from_end("string"), 16);
    assert_eq!(string.search_from_end("xyz"), -1);
}
#[test]
fn TCollection_AsciiStringTest__SearchFromEnd_EdgeCases() {
    let string = AsciiString::new("abcabc");
    for (query, expected) in [("abc", 4), ("a", 4), ("c", 6), ("", -1)] {
        assert_eq!(string.search_from_end(query), expected);
    }
    assert_eq!(AsciiString::default().search_from_end("test"), -1);
}
#[test]
fn TCollection_AsciiStringTest__Search_LongerThanString() {
    assert_eq!(AsciiString::new("Short").search("VeryLongSubstring"), -1);
}
#[test]
fn TCollection_AsciiStringTest__IsSameString_CaseSensitive() {
    let first = AsciiString::new("Test");
    let second = AsciiString::new("Test");
    let third = AsciiString::new("test");
    let fourth = AsciiString::new("Different");
    assert!(first.is_same_string(&second, true));
    assert!(!first.is_same_string(&third, true));
    assert!(!first.is_same_string(&fourth, true));
}
#[test]
fn TCollection_AsciiStringTest__IsSameString_CaseInsensitive() {
    let first = AsciiString::new("Test");
    for other in [
        AsciiString::new("TEST"),
        AsciiString::new("test"),
        AsciiString::new("TeSt"),
    ] {
        assert!(first.is_same_string(&other, false));
    }
}
#[test]
fn TCollection_AsciiStringTest__IsSameString_EmptyStrings() {
    let first = AsciiString::default();
    let second = AsciiString::new("");
    let third = AsciiString::new("Test");
    assert!(first.is_same_string(&second, true));
    assert!(first.is_same_string(&second, false));
    assert!(!first.is_same_string(&third, true));
}
#[test]
fn TCollection_AsciiStringTest__IsSameString_DifferentLengths() {
    let first = AsciiString::new("Short");
    let second = AsciiString::new("VeryLong");
    assert!(!first.is_same_string(&second, true));
    assert!(!first.is_same_string(&second, false));
}
#[test]
fn TCollection_AsciiStringTest__IsSameString_WithCStrings() {
    assert!(AsciiString::same_bytes(&b"Test"[..4], &b"Test"[..4], true));
    assert!(!AsciiString::same_bytes(&b"Test"[..4], &b"test"[..4], true));
    assert!(AsciiString::same_bytes(&b"Test"[..4], &b"test"[..4], false));
}
#[test]
fn TCollection_AsciiStringTest__IsEqual_AllOverloads() {
    let string = AsciiString::new("Test");
    let second = AsciiString::new("Test");
    assert!(string.is_equal(&second));
    assert!(string.is_equal("Test"));
    assert!(!string.is_equal("Different"));
    assert!(!string.is_equal(AsciiString::new("test")));
}
#[test]
fn TCollection_AsciiStringTest__IsDifferent_AllOverloads() {
    let string = AsciiString::new("Test");
    assert!(string.is_different(AsciiString::new("Different")));
    assert!(string.is_different("Different"));
    assert!(!string.is_different("Test"));
}
#[test]
fn TCollection_AsciiStringTest__IsLess_AllOverloads() {
    let first = AsciiString::new("Apple");
    let second = AsciiString::new("Banana");
    assert!(first.is_less(&second));
    assert!(!second.is_less(&first));
    assert!(first.is_less("Banana"));
    assert!(!second.is_less("Apple"));
    assert!(!first.is_less(&first));
}
#[test]
fn TCollection_AsciiStringTest__IsGreater_AllOverloads() {
    let first = AsciiString::new("Apple");
    let second = AsciiString::new("Banana");
    assert!(second.is_greater(&first));
    assert!(!first.is_greater(&second));
    assert!(second.is_greater("Apple"));
    assert!(!first.is_greater("Banana"));
    assert!(!first.is_greater(&first));
}
#[test]
fn TCollection_AsciiStringTest__Comparison_EmptyStrings() {
    let empty = AsciiString::default();
    let nonempty = AsciiString::new("Test");
    assert!(empty.is_equal(""));
    assert!(!empty.is_equal(&nonempty));
    assert!(empty.is_different(&nonempty));
    assert!(empty.is_less(&nonempty));
    assert!(!empty.is_greater(&nonempty));
}
