// Rust port of ApplicationFramework/TKCDF/LDOM/LDOMBasicString.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOMBasicString`: an integer or a byte string, free or owned by a document (ldom
//! section of ocaf-design.md, section 9).
use super::LdomMemManager;
use super::c_text::{c_str, strtol};
use rustycad_tkernel::{
    standard::Handle,
    tcollection::{AsciiString, ExtendedString},
};
use std::sync::Arc;

/// The bytes of one string (without the C terminator) and, for a hashed name, the hash
/// index the native allocation keeps in the byte before the string.
#[derive(Debug)]
pub struct LdomChars {
    pub(crate) bytes: Box<[u8]>,
    pub(crate) hash: Option<u8>,
}

impl LdomChars {
    /// A new allocation of `bytes`.
    pub(crate) fn new(bytes: &[u8], hash: Option<u8>) -> Arc<Self> {
        Arc::new(Self {
            bytes: bytes.into(),
            hash,
        })
    }
}

/// `LDOMBasicString::StringType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LdomStringType {
    /// `LDOM_NULL`.
    #[default]
    LdomNull,
    /// `LDOM_Integer`.
    LdomInteger,
    /// `LDOM_AsciiFree`: not connected to any document.
    LdomAsciiFree,
    /// `LDOM_AsciiDoc`: connected to a document.
    LdomAsciiDoc,
    /// `LDOM_AsciiDocClear`: connected to a document, only XML-valid characters.
    LdomAsciiDocClear,
    /// `LDOM_AsciiHashed`: connected to the hash table of a document.
    LdomAsciiHashed,
}

/// The native union `myVal` with its type `myType`.
#[derive(Debug, Default)]
pub(crate) enum StringValue {
    #[default]
    Null,
    Integer(i32),
    Text(LdomStringType, Arc<LdomChars>),
}

/// `LDOMBasicString`; `LdomBasicString::from_integer(5)`.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomBasicString, LdomStringType};
/// let text = LdomBasicString::from_c_str(Some(b"12"));
/// assert_eq!(text.string_type(), LdomStringType::LdomAsciiFree);
/// assert!(text.equals(&LdomBasicString::from_integer(12)));
/// assert!(text != text.clone(), "a copy of a free string is a new allocation");
/// ```
#[derive(Debug, Default)]
pub struct LdomBasicString {
    pub(crate) value: StringValue,
}

impl Clone for LdomBasicString {
    /// The copy constructor: a free string is copied, a document string is shared.
    fn clone(&self) -> Self {
        let value = match &self.value {
            StringValue::Null => StringValue::Null,
            StringValue::Integer(value) => StringValue::Integer(*value),
            StringValue::Text(LdomStringType::LdomAsciiFree, text) => StringValue::Text(
                LdomStringType::LdomAsciiFree,
                LdomChars::new(&text.bytes, text.hash),
            ),
            StringValue::Text(kind, text) => StringValue::Text(*kind, Arc::clone(text)),
        };
        Self { value }
    }
}

impl PartialEq for LdomBasicString {
    /// `operator==`: the same type and the same integer or the same string allocation.
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (StringValue::Null, StringValue::Null) => true,
            (StringValue::Integer(a), StringValue::Integer(b)) => a == b,
            (StringValue::Text(kind_a, a), StringValue::Text(kind_b, b)) => {
                kind_a == kind_b && Arc::ptr_eq(a, b)
            }
            _ => false,
        }
    }
}

impl LdomBasicString {
    /// `LDOMBasicString()`: the null string.
    pub fn new() -> Self {
        Self::default()
    }
    /// `LDOMBasicString(aValue)` with an integer.
    pub fn from_integer(value: i32) -> Self {
        Self {
            value: StringValue::Integer(value),
        }
    }
    /// `LDOMBasicString(const char*)`: an `LDOM_AsciiFree` copy, null for a null pointer.
    pub fn from_c_str(value: Option<&[u8]>) -> Self {
        Self::text(LdomStringType::LdomAsciiFree, value.map(c_str))
    }
    /// `LDOMBasicString(aValue, aDoc)`: an `LDOM_AsciiDoc` copy, null for a null pointer.
    pub fn with_document(value: Option<&[u8]>, _document: &Handle<LdomMemManager>) -> Self {
        Self::text(LdomStringType::LdomAsciiDoc, value.map(c_str))
    }
    /// `LDOMBasicString(aValue, aLen, aDoc)`: an `LDOM_AsciiDoc` copy of `length` bytes,
    /// null for a null pointer or a zero length.
    pub fn with_length(
        value: Option<&[u8]>,
        length: i32,
        _document: &Handle<LdomMemManager>,
    ) -> Self {
        let length = usize::try_from(length).unwrap_or(0);
        let value = value
            .filter(|_| length > 0)
            .map(|bytes| &bytes[..length.min(bytes.len())]);
        Self::text(LdomStringType::LdomAsciiDoc, value)
    }
    /// A string of `kind` over a new allocation of `bytes`, or the null string.
    fn text(kind: LdomStringType, bytes: Option<&[u8]>) -> Self {
        bytes.map_or_else(Self::new, |bytes| Self {
            value: StringValue::Text(kind, LdomChars::new(bytes, None)),
        })
    }
    /// `Type`.
    pub fn string_type(&self) -> LdomStringType {
        match &self.value {
            StringValue::Null => LdomStringType::LdomNull,
            StringValue::Integer(_) => LdomStringType::LdomInteger,
            StringValue::Text(kind, _) => *kind,
        }
    }
    /// `GetString`: the C string of a text, empty for an integer or the null string.
    pub fn get_string(&self) -> &[u8] {
        match &self.value {
            StringValue::Text(_, text) => c_str(&text.bytes),
            _ => b"",
        }
    }
    /// The shared allocation of a text (the native pointer).
    pub(crate) fn text_ref(&self) -> Option<&Arc<LdomChars>> {
        match &self.value {
            StringValue::Text(_, text) => Some(text),
            _ => None,
        }
    }
    /// `SetDirect(aType, aValue)`: the type and the allocation, without copy.
    pub(crate) fn set_direct(&mut self, kind: LdomStringType, text: Arc<LdomChars>) {
        self.value = StringValue::Text(kind, text);
    }
    /// Retypes a document string (`SetValueClear` turns `LDOM_AsciiDoc` into
    /// `LDOM_AsciiDocClear`).
    pub(crate) fn retype(&mut self, kind: LdomStringType) {
        if let StringValue::Text(current, _) = &mut self.value {
            *current = kind;
        }
    }
    /// `GetInteger(aResult)`: the integer, or the C `strtol` of the text; false without
    /// digits or on overflow (the `long` result is truncated to `int`, as natively).
    pub fn get_integer(&self, result: &mut i32) -> bool {
        match &self.value {
            StringValue::Integer(value) => *result = *value,
            StringValue::Text(..) => {
                let parsed = strtol(self.get_string(), 10);
                if parsed.consumed == 0 || parsed.overflow {
                    return false;
                }
                *result = parsed.value as i32;
            }
            StringValue::Null => return false,
        }
        true
    }
    /// `equals`: the same contents; an integer equals a text whose `strtol` gives it (a text
    /// without digits reads as 0, as natively).
    pub fn equals(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (StringValue::Null, _) => other.is_null(),
            (StringValue::Integer(a), StringValue::Integer(b)) => a == b,
            (StringValue::Integer(a), StringValue::Text(..)) => {
                integer_matches(*a, other.get_string())
            }
            (StringValue::Text(..), StringValue::Integer(b)) => {
                integer_matches(*b, self.get_string())
            }
            (StringValue::Text(..), StringValue::Text(..)) => {
                self.get_string() == other.get_string()
            }
            _ => false,
        }
    }
    /// `operator==(LDOM_NullPtr*)`: the null string.
    pub fn is_null(&self) -> bool {
        matches!(self.value, StringValue::Null)
    }
    /// `operator=(LDOM_NullPtr*)`: becomes the null string.
    pub fn set_null(&mut self) {
        self.value = StringValue::Null;
    }
    /// `operator TCollection_AsciiString`: `%d` of an integer, the C string of a text.
    pub fn to_ascii_string(&self) -> AsciiString {
        match &self.value {
            StringValue::Integer(value) => AsciiString::from_integer(*value),
            StringValue::Text(..) => AsciiString::from_bytes(self.get_string()),
            StringValue::Null => AsciiString::default(),
        }
    }
    /// `operator TCollection_ExtendedString`: `%d` of an integer; a text `##feff` followed by
    /// four hexadecimal digits per character is decoded as UTF-16, any other text widens
    /// each byte.
    pub fn to_extended_string(&self) -> ExtendedString {
        match &self.value {
            StringValue::Integer(value) => ExtendedString::from_integer(*value),
            StringValue::Text(..) => unicode_hex_or_widened(self.get_string()),
            StringValue::Null => ExtendedString::default(),
        }
    }
}

/// The `strtol` comparison of `equals`.
fn integer_matches(value: i32, text: &[u8]) -> bool {
    let parsed = strtol(text, 10);
    !parsed.overflow && parsed.value == i64::from(value)
}

/// The text branch of `operator TCollection_ExtendedString`.
fn unicode_hex_or_widened(text: &[u8]) -> ExtendedString {
    let group =
        |start: usize| -> &[u8] { text.get(start..(start + 4).min(text.len())).unwrap_or(b"") };
    if !text.starts_with(b"##") || strtol(c_str(group(2)), 16).value != 0xfeff {
        return ExtendedString::from_c_string(text, false);
    }
    let units_after_header = (text.len() - 2) / 4;
    let mut units = Vec::new();
    for index in 1..units_after_header {
        units.push(strtol(c_str(group(2 + 4 * index)), 16).value as u16);
    }
    let length = units
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(units.len());
    ExtendedString::from_raw(&units[..length])
}
