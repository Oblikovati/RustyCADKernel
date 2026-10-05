//! OCCT's pointer overloads and string-view overloads have different NUL semantics.
use rustycad_tkernel::tcollection::AsciiString;

#[test]
fn string_equality_operator_uses_the_native_c_string_prefix() {
    let text = AsciiString::new("a");
    assert!(text == "a\0suffix");
    assert!(!(text != "a\0suffix"));
    assert!(!text.is_equal(b"a\0suffix"));
}

fn comparison_flags(text: &AsciiString, candidate: Option<&[u8]>) -> String {
    [
        text.is_equal_c_string(candidate),
        text.is_different_c_string(candidate),
        text.is_less_c_string(candidate),
        text.is_greater_c_string(candidate),
        text.starts_with_c_string(candidate),
    ]
    .map(|flag| if flag { '1' } else { '0' })
    .iter()
    .collect()
}

#[test]
fn nullable_c_string_comparisons_match_the_native_matrix() {
    let strings: [&[u8]; 4] = [b"", b"a", b"a\0b", b"\xff"];
    let candidates: [Option<&[u8]>; 5] =
        [None, Some(b""), Some(b"a\0tail"), Some(b"b"), Some(b"\xff")];
    let expected = [
        ["10001", "10001", "01100", "01100", "01100"],
        ["01010", "01011", "10001", "01100", "01100"],
        ["01010", "01011", "01011", "01100", "01100"],
        ["01010", "01011", "01010", "01010", "10001"],
    ];
    for (bytes, row) in strings.into_iter().zip(expected) {
        let text = AsciiString::from_raw(bytes);
        for (candidate, flags) in candidates.into_iter().zip(row) {
            assert_eq!(comparison_flags(&text, candidate), flags);
        }
    }
}

#[test]
fn literal_suffix_and_full_view_keep_distinct_nul_semantics() {
    let text = AsciiString::new("a");
    assert!(text.ends_with_literal(b"a\0tail"));
    assert!(!text.ends_with(b"a\0tail"));
    assert!(text.is_equal_c_string(Some(b"a\0tail")));
    assert!(!text.is_equal(b"a\0tail"));
}
