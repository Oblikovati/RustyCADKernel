// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_CharReference.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_CharReference`: the XML character and entity references of text and attribute
//! values (ocaf-design.md section 9).
use super::c_text::{c_str, strtol};

/// `NORMAL_C`: a character written as is.
const NORMAL_C: i8 = 0;
/// `CHAR_REF`: a character written as `&#xNN;`.
const CHAR_REF: i8 = -1;
/// `ENTI_QUOT`: the code of `"` (an entity only in attribute values).
const ENTI_QUOT: i8 = 4;
/// The predefined entities by code (`entity_ref`; code 0 is unused).
const ENTITIES: [&[u8]; 6] = [b"", b"&amp;", b"&lt;", b"&gt;", b"&quot;", b"&apos;"];
/// The entities `Decode` replaces, after the `&`, with their characters.
const DECODED: [(&[u8], u8); 5] = [
    (b"amp;", b'&'),
    (b"lt;", b'<'),
    (b"gt;", b'>'),
    (b"quot;", b'"'),
    (b"apos;", b'\''),
];

/// `LDOM_CharReference`: `decode` and `encode` of character references.
///
/// ```
/// use rustycad_tkcdf::ldom::LdomCharReference;
/// let mut text = b"a&lt;b&#x41;".to_vec();
/// let mut length = 0;
/// assert!(LdomCharReference::decode(&mut text, &mut length));
/// assert_eq!(&text[..length as usize], b"a<bA");
/// let encoded = LdomCharReference::encode(b"\"<\x01", &mut length, true);
/// assert_eq!(encoded, b"&quot;&lt;&#x01;");
/// ```
pub struct LdomCharReference;

impl LdomCharReference {
    /// `myTab`: the class of every byte (`LDOM_ALLOW_LATIN_1` is not defined).
    fn class(byte: u8) -> i8 {
        match byte {
            0 | b'\t' | b'\n' | b'\r' | 0x20..=0x7f => match byte {
                b'&' => 1,
                b'<' => 2,
                b'>' => 3,
                b'"' => ENTI_QUOT,
                _ => NORMAL_C,
            },
            _ => CHAR_REF,
        }
    }
    /// `Decode(theSrc, theLen)`: replaces in place, in the C string at the start of
    /// `source`, the references `&#NN;`, `&#xNN;` (1..=255) and the five predefined
    /// entities; an unknown `&` stays. `length` takes the decoded length. False (the native
    /// null result) for an invalid character reference.
    pub fn decode(source: &mut [u8], length: &mut i32) -> bool {
        let mut cursor = DecodeCursor {
            source: 0,
            increment: 0,
        };
        loop {
            let end = c_str(source).len();
            let Some(amp) = source[cursor.source..end]
                .iter()
                .position(|byte| *byte == b'&')
                .map(|at| cursor.source + at)
            else {
                *length = cursor.finish(source, end) as i32;
                return true;
            };
            let destination = cursor.source - cursor.increment;
            if amp > cursor.source && destination != cursor.source {
                source.copy_within(cursor.source..amp, destination);
            }
            cursor.source = amp;
            if !cursor.replace(source) {
                return false;
            }
        }
    }
    /// `Encode(theSrc, theLen, isAttribute)`: the C string with its special characters as
    /// references (`"` only in an attribute value); `length` takes the result length.
    pub fn encode(source: &[u8], length: &mut i32, is_attribute: bool) -> Vec<u8> {
        let source = c_str(source);
        let mut encoded = Vec::with_capacity(source.len());
        for byte in source {
            match Self::class(*byte) {
                NORMAL_C => encoded.push(*byte),
                CHAR_REF => encoded.extend_from_slice(format!("&#x{byte:02x};").as_bytes()),
                ENTI_QUOT if !is_attribute => encoded.push(*byte),
                code => encoded.extend_from_slice(ENTITIES[code as usize]),
            }
        }
        *length = encoded.len() as i32;
        encoded
    }
}

/// The two moving pointers of `Decode`: the next source byte and how many bytes the
/// destination is behind it.
struct DecodeCursor {
    source: usize,
    increment: usize,
}

impl DecodeCursor {
    /// The tail after the last `&`: moved behind the decoded text; returns the length.
    fn finish(&self, text: &mut [u8], end: usize) -> usize {
        if self.increment == 0 {
            return end;
        }
        let destination = self.source - self.increment;
        let with_terminator = (end + 1).min(text.len());
        text.copy_within(self.source..with_terminator, destination);
        if end == text.len() {
            text[destination + end - self.source] = 0;
        }
        destination + end - self.source
    }
    /// The reference at `self.source` (an `&`): its character written behind, or the `&`
    /// itself copied when it is no known reference; false for an invalid character reference.
    fn replace(&mut self, text: &mut [u8]) -> bool {
        let at = |index: usize| text.get(index).copied().unwrap_or(0);
        if at(self.source + 1) == b'#' {
            return self.replace_number(text);
        }
        let rest = &text[self.source + 1..];
        if let Some((name, character)) = DECODED.iter().find(|(name, _)| rest.starts_with(name)) {
            text[self.source - self.increment] = *character;
            self.increment += name.len();
            self.source += name.len() + 1;
            return true;
        }
        text[self.source - self.increment] = text[self.source];
        self.source += 1;
        true
    }
    /// `&#NN;` or `&#xNN;` (C `strtoul`, so leading spaces and a sign are read too).
    fn replace_number(&mut self, text: &mut [u8]) -> bool {
        let hexadecimal = text.get(self.source + 2) == Some(&b'x');
        let digits = self.source + if hexadecimal { 3 } else { 2 };
        let parsed = strtol(
            &text[digits.min(text.len())..],
            if hexadecimal { 16 } else { 10 },
        );
        let end = digits + parsed.consumed;
        let valid =
            text.get(end) == Some(&b';') && !parsed.overflow && (1..=255).contains(&parsed.value);
        if !valid {
            return false;
        }
        text[self.source - self.increment] = parsed.value as u8;
        self.increment += end - self.source;
        self.source = end + 1;
        true
    }
}
