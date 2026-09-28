//! Standard_Character and Standard_ExtCharacter over the C locale and byte-level conversions.
use rustycad_tkernel::standard::{character, ext_character};

#[test]
fn bytes_above_127_belong_to_no_class() {
    for byte in 128..=255_u8 {
        let classes = [
            character::is_alphabetic(byte),
            character::is_digit(byte),
            character::is_xdigit(byte),
            character::is_alphanumeric(byte),
            character::is_control(byte),
            character::is_graphic(byte),
            character::is_lower_case(byte),
            character::is_printable(byte),
            character::is_punctuation(byte),
            character::is_space(byte),
            character::is_upper_case(byte),
        ];
        assert_eq!(classes, [false; 11], "byte {byte:#04x}");
        assert_eq!(character::lower_case(byte), byte);
        assert_eq!(character::upper_case(byte), byte);
    }
}

#[test]
fn c_locale_classes_of_ascii_boundaries() {
    assert!(character::is_space(0x0B) && character::is_space(b'\r') && !character::is_space(0x1C));
    assert!(character::is_control(0x7F) && !character::is_printable(0x7F));
    assert!(character::is_printable(b' ') && !character::is_graphic(b' '));
    assert!(character::is_punctuation(b'_') && !character::is_punctuation(b'a'));
    assert!(character::is_xdigit(b'F') && !character::is_xdigit(b'g'));
    assert!(character::is_alphanumeric(b'0') && character::is_equal(b'x', b'x'));
    assert_eq!(
        (character::lower_case(b'Z'), character::upper_case(b'z')),
        (b'z', b'Z')
    );
}

#[test]
fn ext_character_conversions_are_byte_level() {
    assert_eq!(ext_character::to_ext_character(0xFF), 0x00FF);
    assert_eq!(ext_character::to_character(0x0141), 0x41);
    assert!(ext_character::is_an_ascii(0x00E9));
    assert!(!ext_character::is_an_ascii(0x0100));
    assert!(ext_character::is_equal(0xD800, 0xD800) && !ext_character::is_equal(1, 2));
}
