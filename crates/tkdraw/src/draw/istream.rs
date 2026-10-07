//! `Standard_IStream` as the restore factories read it: whitespace-separated words and
//! numbers with `operator>>` semantics.
use rustycad_tkernel::standard::strtod;

/// An input stream over the bytes of a saved DRAW file.
///
/// ```
/// use rustycad_tkdraw::draw::DrawIStream;
/// let mut stream = DrawIStream::new(b"Draw_Number\n  2.5e+00 rest".to_vec());
/// assert_eq!(stream.read_word().as_deref(), Some("Draw_Number"));
/// assert_eq!(stream.read_real(), Some(2.5));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawIStream {
    bytes: Vec<u8>,
    position: usize,
    failed: bool,
}

impl DrawIStream {
    /// A stream positioned at the first byte.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            position: 0,
            failed: false,
        }
    }

    /// `fail()`: an extraction found nothing to read.
    pub fn fail(&self) -> bool {
        self.failed
    }

    /// `seekg(0, beg)`: rewinds and clears the state.
    pub fn rewind(&mut self) {
        self.position = 0;
        self.failed = false;
    }

    /// Skips leading whitespace (the sentry of formatted input).
    fn skip_whitespace(&mut self) {
        while self
            .bytes
            .get(self.position)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.position += 1;
        }
    }

    /// `operator>>(char*)`: the next whitespace-delimited word; `None` (and `fail`) at
    /// the end.
    pub fn read_word(&mut self) -> Option<String> {
        self.skip_whitespace();
        let start = self.position;
        while self
            .bytes
            .get(self.position)
            .is_some_and(|b| !b.is_ascii_whitespace())
        {
            self.position += 1;
        }
        if start == self.position {
            self.failed = true;
            return None;
        }
        Some(String::from_utf8_lossy(&self.bytes[start..self.position]).into_owned())
    }

    /// `operator>>(double&)`: the longest number at the position; `None` (and `fail`;
    /// C++11 then stores 0) when none is there.
    pub fn read_real(&mut self) -> Option<f64> {
        self.skip_whitespace();
        let rest = &self.bytes[self.position..];
        let length = number_prefix_length(rest);
        let text = String::from_utf8_lossy(&rest[..length]).into_owned();
        let (value, consumed) = strtod(&text);
        if length == 0 || consumed == 0 {
            self.failed = true;
            return None;
        }
        self.position += consumed;
        Some(value)
    }
}

/// The length of the `[+-]digits[.digits][(e|E)[+-]digits]` prefix of `bytes`.
fn number_prefix_length(bytes: &[u8]) -> usize {
    let mut index = sign_length(bytes, 0);
    index = digits_end(bytes, index);
    if bytes.get(index) == Some(&b'.') {
        index = digits_end(bytes, index + 1);
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent = digits_end(bytes, sign_length(bytes, index + 1));
        if exponent > sign_length(bytes, index + 1) {
            index = exponent;
        }
    }
    index
}

/// `index` advanced over an optional sign.
fn sign_length(bytes: &[u8], index: usize) -> usize {
    if matches!(bytes.get(index), Some(b'+' | b'-')) {
        index + 1
    } else {
        index
    }
}

/// The index after the digits starting at `index`.
fn digits_end(bytes: &[u8], mut index: usize) -> usize {
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_stop_at_the_first_invalid_byte() {
        let mut stream = DrawIStream::new(b"-1.5e-3x 7".to_vec());
        assert_eq!(stream.read_real(), Some(-1.5e-3));
        assert_eq!(stream.read_word().as_deref(), Some("x"));
        assert_eq!(stream.read_real(), Some(7.0));
        assert_eq!(stream.read_real(), None);
        assert!(stream.fail());
    }

    #[test]
    fn a_dangling_exponent_is_not_consumed() {
        assert_eq!(number_prefix_length(b"2e+"), 1);
        assert_eq!(number_prefix_length(b"abc"), 0);
    }
}
