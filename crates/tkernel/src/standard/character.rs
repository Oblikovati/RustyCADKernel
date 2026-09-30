// Rust translation of FoundationClasses/TKernel/Standard/Standard_Character.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_Character classification in the C locale.
//!
//! The C++ functions take `char` and cast it to `unsigned char` before calling `<cctype>`,
//! so a negative `char` is classified as its byte value 128..255. The port takes that byte
//! (`u8`) directly, which keeps all 256 values independent of the platform's `char`
//! signedness. In the C locale no byte above 127 belongs to any class, and the case
//! mappings change only `A-Z` / `a-z`.

/// `IsAlphabetic(char)`: C `isalpha`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_alphabetic(b'A'));
/// ```
pub const fn is_alphabetic(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

/// `IsDigit(char)`: C `isdigit`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_digit(b'7'));
/// ```
pub const fn is_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}

/// `IsXDigit(char)`: C `isxdigit`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_xdigit(b'f'));
/// ```
pub const fn is_xdigit(byte: u8) -> bool {
    byte.is_ascii_hexdigit()
}

/// `IsAlphanumeric(char)`: `IsAlphabetic || IsDigit`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_alphanumeric(b'z'));
/// ```
pub const fn is_alphanumeric(byte: u8) -> bool {
    is_alphabetic(byte) || is_digit(byte)
}

/// `IsControl(char)`: C `iscntrl` (0..=31 and 127).
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_control(0x7F));
/// ```
pub const fn is_control(byte: u8) -> bool {
    byte.is_ascii_control()
}

/// `IsGraphic(char)`: C `isgraph` (`!`..=`~`).
///
/// ```
/// assert!(!rustycad_tkernel::standard::character::is_graphic(b' '));
/// ```
pub const fn is_graphic(byte: u8) -> bool {
    byte.is_ascii_graphic()
}

/// `IsLowerCase(char)`: C `islower`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_lower_case(b'q'));
/// ```
pub const fn is_lower_case(byte: u8) -> bool {
    byte.is_ascii_lowercase()
}

/// `IsPrintable(char)`: C `isprint` (space..=`~`).
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_printable(b' '));
/// ```
pub const fn is_printable(byte: u8) -> bool {
    byte >= b' ' && byte <= b'~'
}

/// `IsPunctuation(char)`: `IsGraphic && !IsAlphanumeric`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_punctuation(b'#'));
/// ```
pub const fn is_punctuation(byte: u8) -> bool {
    is_graphic(byte) && !is_alphanumeric(byte)
}

/// `IsSpace(char)`: C `isspace`, which unlike `u8::is_ascii_whitespace` includes `\v` (11).
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_space(11));
/// ```
pub const fn is_space(byte: u8) -> bool {
    byte == b' ' || (byte >= 9 && byte <= 13)
}

/// `IsUpperCase(char)`: C `isupper`.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_upper_case(b'Q'));
/// ```
pub const fn is_upper_case(byte: u8) -> bool {
    byte.is_ascii_uppercase()
}

/// `LowerCase(char)`: C `tolower`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::character::lower_case(b'A'), b'a');
/// ```
pub const fn lower_case(byte: u8) -> u8 {
    byte.to_ascii_lowercase()
}

/// `UpperCase(char)`: C `toupper`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::character::upper_case(b'a'), b'A');
/// ```
pub const fn upper_case(byte: u8) -> u8 {
    byte.to_ascii_uppercase()
}

/// `IsEqual(char, char)`: value equality.
///
/// ```
/// assert!(rustycad_tkernel::standard::character::is_equal(b'A', b'A'));
/// ```
pub const fn is_equal(first: u8, second: u8) -> bool {
    first == second
}
