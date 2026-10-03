pub(super) fn is_alpha(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}
pub(super) fn is_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}
pub(super) fn is_xdigit(byte: u8) -> bool {
    byte.is_ascii_hexdigit()
}
pub(super) fn is_control(byte: u8) -> bool {
    byte.is_ascii_control()
}
pub(super) fn is_graphic(byte: u8) -> bool {
    byte.is_ascii_graphic()
}
pub(super) fn is_lower(byte: u8) -> bool {
    byte.is_ascii_lowercase()
}
pub(super) fn is_print(byte: u8) -> bool {
    byte.is_ascii_graphic() || byte == b' '
}
pub(super) fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t'..=b'\r')
}
pub(super) fn is_upper(byte: u8) -> bool {
    byte.is_ascii_uppercase()
}
pub(super) fn to_lower(byte: u8) -> u8 {
    byte.to_ascii_lowercase()
}
pub(super) fn to_upper(byte: u8) -> u8 {
    byte.to_ascii_uppercase()
}
