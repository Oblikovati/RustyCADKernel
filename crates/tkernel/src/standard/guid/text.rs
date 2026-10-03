// Rust translation of FoundationClasses/TKernel/Standard/Standard_GUID.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text forms of `Standard_GUID`: CheckGUIDFormat, the string constructors, ToCString,
//! ToExtString and ShallowDump.

use super::{GUID_SIZE, GUID_SIZE_ALLOC, Guid, Uuid};
use crate::standard::Failure;
use crate::standard::character::is_xdigit;
use crate::standard::exceptions::range_error;

/// The message of every `Standard_RangeError` the string constructors throw.
const INVALID_FORMAT: &str = "Invalid format of GUID";

/// Value of one hexadecimal digit that `CheckGUIDFormat` accepted.
fn hex_value(digit: u8) -> u8 {
    if digit.is_ascii_digit() {
        digit - b'0'
    } else {
        digit.to_ascii_lowercase() - b'a' + 10
    }
}

/// The 16 field bytes of a validated text, in displayed order (what the `strtoul` calls read).
fn field_bytes(text: &[u8]) -> [u8; 16] {
    let mut bytes: [u8; 16] = [0; 16];
    let digits = text[..GUID_SIZE].iter().filter(|&&ch| ch != b'-');
    for (nibble, &digit) in digits.enumerate() {
        bytes[nibble / 2] = bytes[nibble / 2] * 16 + hex_value(digit);
    }
    bytes
}

impl Guid {
    /// `CheckGUIDFormat(const char*)`: a C string of exactly 36 characters, hexadecimal
    /// digits (C-locale `isxdigit`) with dashes at 8, 13, 18 and 23. `None` is `nullptr`.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// assert!(Guid::check_format(Some(b"00000000-0000-0000-0000-00000000000a")));
    /// ```
    pub fn check_format(text: Option<&[u8]>) -> bool {
        let Some(text) = text else {
            return false;
        };
        let length: usize = text.iter().position(|&c| c == 0).unwrap_or(text.len());
        length == GUID_SIZE
            && text[..GUID_SIZE].iter().enumerate().all(|(i, &byte)| {
                if matches!(i, 8 | 13 | 18 | 23) {
                    byte == b'-'
                } else {
                    is_xdigit(byte)
                }
            })
    }

    /// `Standard_GUID(const char*)`: parses the C string in `text` (it ends at the first NUL).
    ///
    /// # Errors
    /// `Standard_RangeError` "Invalid format of GUID" when [`Guid::check_format`] rejects it.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// let guid = Guid::from_ascii(b"0000000A-0000-0000-0000-000000000000").unwrap();
    /// assert_eq!(guid.to_uuid().data1, 10);
    /// ```
    pub fn from_ascii(text: &[u8]) -> Result<Self, Failure> {
        if !Self::check_format(Some(text)) {
            return Err(range_error(INVALID_FORMAT));
        }
        let bytes: [u8; 16] = field_bytes(text);
        let [a, b, c, d, e, f, g, h, data4 @ ..] = bytes;
        Ok(Self(Uuid {
            data1: u32::from_be_bytes([a, b, c, d]),
            data2: u16::from_be_bytes([e, f]),
            data3: u16::from_be_bytes([g, h]),
            data4,
        }))
    }

    /// `Standard_GUID(const char16_t*)`: keeps the low byte of each of the first 36 units
    /// (`(char)aGuid[i]`), then parses them as [`Guid::from_ascii`] does.
    ///
    /// # Errors
    /// `Standard_RangeError` "Invalid format of GUID" for a rejected text. A slice shorter
    /// than 36 units is also a `Standard_RangeError` naming its length: OCCT reads 36 units
    /// unconditionally, past the end of a shorter buffer.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// let text: Vec<u16> = "0000000a-0000-0000-0000-000000000000".encode_utf16().collect();
    /// assert_eq!(Guid::from_utf16(&text).unwrap().to_uuid().data1, 10);
    /// ```
    pub fn from_utf16(text: &[u16]) -> Result<Self, Failure> {
        if text.len() < GUID_SIZE {
            return Err(range_error(&format!(
                "{INVALID_FORMAT}: {} UTF-16 units, expected at least {GUID_SIZE}",
                text.len()
            )));
        }
        let bytes: [u8; GUID_SIZE] = std::array::from_fn(|i| text[i] as u8);
        Self::from_ascii(&bytes)
    }

    /// `ToCString(Standard_PCharacter)`: the lowercase text and its NUL terminator.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// assert_eq!(Guid::new().to_c_string()[36], 0);
    /// ```
    pub fn to_c_string(self) -> [u8; GUID_SIZE_ALLOC] {
        let mut output: [u8; GUID_SIZE_ALLOC] = [0; GUID_SIZE_ALLOC];
        self.write_c_string(&mut output);
        output
    }

    /// Writes lowercase text and a NUL into an OCCT-sized output buffer.
    ///
    /// ```
    /// use rustycad_tkernel::standard::{Guid, GUID_SIZE_ALLOC};
    /// let mut output = [b'X'; GUID_SIZE_ALLOC];
    /// Guid::new().write_c_string(&mut output);
    /// assert_eq!(output[36], 0);
    /// ```
    pub fn write_c_string(&self, output: &mut [u8; GUID_SIZE_ALLOC]) {
        output[..GUID_SIZE].copy_from_slice(self.to_string().as_bytes());
        output[GUID_SIZE] = 0;
    }

    /// `ToExtString(Standard_PExtCharacter)`: the UTF-16 text and its NUL terminator.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// assert_eq!(Guid::new().to_ext_string()[0], u16::from(b'0'));
    /// ```
    pub fn to_ext_string(self) -> [u16; GUID_SIZE_ALLOC] {
        self.to_c_string().map(u16::from)
    }

    /// `ShallowDump(Standard_OStream&)`: writes the text form.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// let mut output = String::new();
    /// Guid::new().shallow_dump(&mut output).unwrap();
    /// assert_eq!(output.len(), 36);
    /// ```
    pub fn shallow_dump(self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(output, "{self}")
    }
}

impl std::str::FromStr for Guid {
    type Err = Failure;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_ascii(s.as_bytes())
    }
}

/// The `Sprintf` format `%.8x-%.4x-%.4x-%.4x-%.2x%.2x%.2x%.2x%.2x%.2x` of `ToCString`.
impl std::fmt::Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uuid: Uuid = self.0;
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-",
            uuid.data1, uuid.data2, uuid.data3, uuid.data4[0], uuid.data4[1]
        )?;
        for byte in &uuid.data4[2..] {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{GUID_SIZE, GUID_SIZE_ALLOC, Guid};

    #[test]
    fn writes_complete_text_and_replaces_previous_terminator() {
        let guid = Guid::from_components(
            0x12345678, 0x9abc, 0xdef0, 0x1234, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        );
        let mut output = [b'X'; GUID_SIZE_ALLOC];
        guid.write_c_string(&mut output);

        assert_eq!(
            &output[..GUID_SIZE],
            b"12345678-9abc-def0-1234-56789abcdef0"
        );
        assert_eq!(output[GUID_SIZE], 0);
    }
}
