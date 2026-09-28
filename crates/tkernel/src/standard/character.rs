//! Character classification for the recorded C-locale oracle profile.
//! Byte input preserves all 256 character values independently of platform char signedness.
/// Applies C-locale is alphabetic; `is_alphabetic(b'A')`.
pub const fn is_alphabetic(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}
/// Applies C-locale is digit; `is_digit(b'A')`.
pub const fn is_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}
/// Applies C-locale is xdigit; `is_xdigit(b'A')`.
pub const fn is_xdigit(byte: u8) -> bool {
    byte.is_ascii_hexdigit()
}
/// Applies C-locale is alphanumeric; `is_alphanumeric(b'A')`.
pub const fn is_alphanumeric(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}
/// Applies C-locale is control; `is_control(b'A')`.
pub const fn is_control(byte: u8) -> bool {
    byte.is_ascii_control()
}
/// Applies C-locale is graphic; `is_graphic(b'A')`.
pub const fn is_graphic(byte: u8) -> bool {
    byte.is_ascii_graphic()
}
/// Applies C-locale is lower case; `is_lower_case(b'A')`.
pub const fn is_lower_case(byte: u8) -> bool {
    byte.is_ascii_lowercase()
}
/// Applies C-locale is printable; `is_printable(b'A')`.
pub const fn is_printable(byte: u8) -> bool {
    byte >= b' ' && byte <= b'~'
}
/// Applies C-locale is punctuation; `is_punctuation(b'A')`.
pub const fn is_punctuation(byte: u8) -> bool {
    byte.is_ascii_punctuation()
}
/// Applies C-locale is space; `is_space(b'A')`.
pub const fn is_space(byte: u8) -> bool {
    byte == b' ' || (byte >= 9 && byte <= 13)
}
/// Applies C-locale is upper case; `is_upper_case(b'A')`.
pub const fn is_upper_case(byte: u8) -> bool {
    byte.is_ascii_uppercase()
}
/// Applies C-locale lower case; `lower_case(b'A')`.
pub const fn lower_case(byte: u8) -> u8 {
    byte.to_ascii_lowercase()
}
/// Applies C-locale upper case; `upper_case(b'A')`.
pub const fn upper_case(byte: u8) -> u8 {
    byte.to_ascii_uppercase()
}
/// Compares byte values; `is_equal(b'A', b'A')`.
pub const fn is_equal(first: u8, second: u8) -> bool {
    first == second
}
