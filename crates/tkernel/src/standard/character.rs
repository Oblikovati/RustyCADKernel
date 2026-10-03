// Rust translation of FoundationClasses/TKernel/Standard/Standard_Character.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_Character classification through the active C character locale.
//!
//! The C++ functions take `char` and cast it to `unsigned char` before calling `<cctype>`,
//! so a negative `char` is classified as its byte value 128..255. The port takes that byte
//! (`u8`) directly, which keeps all 256 values independent of the platform's `char`
//! signedness. Unix builds use the host C library and follow its active `LC_CTYPE` locale.
//! Non-Unix builds use the ASCII fallback and have not been verified for non-C locales.

#[cfg(unix)]
#[path = "character/ctype_unix.rs"]
mod ctype;
#[cfg(not(unix))]
#[path = "character/ctype_fallback.rs"]
mod ctype;

/// `IsAlphabetic(char)`: C `isalpha`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_alphabetic(b'A'));
/// ```
pub fn is_alphabetic(byte: u8) -> bool {
    ctype::is_alpha(byte)
}

/// `IsDigit(char)`: C `isdigit`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_digit(b'7'));
/// ```
pub fn is_digit(byte: u8) -> bool {
    ctype::is_digit(byte)
}

/// `IsXDigit(char)`: C `isxdigit`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_xdigit(b'f'));
/// ```
pub fn is_xdigit(byte: u8) -> bool {
    ctype::is_xdigit(byte)
}

/// `IsAlphanumeric(char)`: `IsAlphabetic || IsDigit`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_alphanumeric(b'z'));
/// ```
pub fn is_alphanumeric(byte: u8) -> bool {
    is_alphabetic(byte) || is_digit(byte)
}

/// `IsControl(char)`: C `iscntrl` (0..=31 and 127).
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_control(0x7F));
/// ```
pub fn is_control(byte: u8) -> bool {
    ctype::is_control(byte)
}

/// `IsGraphic(char)`: C `isgraph` (`!`..=`~`).
///
/// ```
/// assert!(!rustycad_tkernel::standard::character::is_graphic(b' '));
/// ```
pub fn is_graphic(byte: u8) -> bool {
    ctype::is_graphic(byte)
}

/// `IsLowerCase(char)`: C `islower`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_lower_case(b'q'));
/// ```
pub fn is_lower_case(byte: u8) -> bool {
    ctype::is_lower(byte)
}

/// `IsPrintable(char)`: C `isprint` (space..=`~`).
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_printable(b' '));
/// ```
pub fn is_printable(byte: u8) -> bool {
    ctype::is_print(byte)
}

/// `IsPunctuation(char)`: `IsGraphic && !IsAlphanumeric`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_punctuation(b'#'));
/// ```
pub fn is_punctuation(byte: u8) -> bool {
    is_graphic(byte) && !is_alphanumeric(byte)
}

/// `IsSpace(char)`: C `isspace`, which unlike `u8::is_ascii_whitespace` includes `\v` (11).
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_space(11));
/// ```
pub fn is_space(byte: u8) -> bool {
    ctype::is_space(byte)
}

/// `IsUpperCase(char)`: C `isupper`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_upper_case(b'Q'));
/// ```
pub fn is_upper_case(byte: u8) -> bool {
    ctype::is_upper(byte)
}

/// `LowerCase(char)`: C `tolower`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::character::lower_case(b'A'), b'a');
/// ```
pub fn lower_case(byte: u8) -> u8 {
    ctype::to_lower(byte)
}

/// `UpperCase(char)`: C `toupper`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::character::upper_case(b'a'), b'A');
/// ```
pub fn upper_case(byte: u8) -> u8 {
    ctype::to_upper(byte)
}

/// `IsEqual(char, char)`: value equality.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_equal(b'A', b'A'));
/// ```
pub const fn is_equal(first: u8, second: u8) -> bool {
    first == second
}
