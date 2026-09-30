//! OCC29925: classify every byte without signed-character indexing failures.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::character::*;
#[test]
fn Standard_Character_Test__OCC29925_CharacterClassificationFunctions() {
    for byte in 0..=255 {
        let _ = (
            is_alphabetic(byte),
            is_digit(byte),
            is_xdigit(byte),
            is_alphanumeric(byte),
            is_control(byte),
            is_graphic(byte),
            is_lower_case(byte),
            is_printable(byte),
            is_punctuation(byte),
            is_space(byte),
            is_upper_case(byte),
            lower_case(byte),
            upper_case(byte),
        );
    }
    assert!(is_alphabetic(b'A'));
    assert!(is_alphabetic(b'z'));
    assert!(!is_alphabetic(b'5'));
    assert!(is_digit(b'0'));
    assert!(is_digit(b'9'));
    assert!(!is_digit(b'A'));
    assert!(is_space(b' '));
    assert!(is_space(b'\t'));
    assert!(!is_space(b'A'));
    assert!(is_lower_case(b'a'));
    assert!(!is_lower_case(b'A'));
    assert!(is_upper_case(b'A'));
    assert!(!is_upper_case(b'a'));
    assert_eq!(lower_case(b'A'), b'a');
    assert_eq!(upper_case(b'a'), b'A');
}
