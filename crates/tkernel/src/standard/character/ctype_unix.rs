#![allow(unsafe_code)]

// SAFETY: u8 is in the C ctype domain; this module contains the Unix FFI boundary.
use libc::c_int;

fn classify(byte: u8, predicate: unsafe extern "C" fn(c_int) -> c_int) -> bool {
    unsafe { predicate(c_int::from(byte)) != 0 }
}

fn convert(byte: u8, conversion: unsafe extern "C" fn(c_int) -> c_int) -> u8 {
    unsafe { conversion(c_int::from(byte)) as u8 }
}

pub(super) fn is_alpha(byte: u8) -> bool {
    classify(byte, libc::isalpha)
}
pub(super) fn is_digit(byte: u8) -> bool {
    classify(byte, libc::isdigit)
}
pub(super) fn is_xdigit(byte: u8) -> bool {
    classify(byte, libc::isxdigit)
}
pub(super) fn is_control(byte: u8) -> bool {
    classify(byte, libc::iscntrl)
}
pub(super) fn is_graphic(byte: u8) -> bool {
    classify(byte, libc::isgraph)
}
pub(super) fn is_lower(byte: u8) -> bool {
    classify(byte, libc::islower)
}
pub(super) fn is_print(byte: u8) -> bool {
    classify(byte, libc::isprint)
}
pub(super) fn is_space(byte: u8) -> bool {
    classify(byte, libc::isspace)
}
pub(super) fn is_upper(byte: u8) -> bool {
    classify(byte, libc::isupper)
}
pub(super) fn to_lower(byte: u8) -> u8 {
    convert(byte, libc::tolower)
}
pub(super) fn to_upper(byte: u8) -> u8 {
    convert(byte, libc::toupper)
}
