//! OCCT 3d097a's static empty buffer differs from an owned buffer truncated to zero.
use super::AsciiString;

// An empty Vec marks native THE_DEFAULT_CHAR_STRING without allocating or mutating it.
pub(super) static SHARED_EMPTY_TERMINATOR: [u8; 1] = [0];

pub(super) fn padded_byte_capacity(length: usize) -> usize {
    // OCCT's calculatePaddedSize reserves a terminator and rounds to four bytes.
    (length + 4) & !3
}

impl AsciiString {
    pub(super) fn ensure_owned_terminator(&mut self) {
        if self.terminated.is_empty() {
            self.terminated.push(0);
        }
    }
}

impl Clone for AsciiString {
    fn clone(&self) -> Self {
        Self::from_raw(self.as_bytes())
    }

    fn clone_from(&mut self, source: &Self) {
        self.copy(source);
    }
}

impl PartialEq for AsciiString {
    fn eq(&self, other: &Self) -> bool {
        // Native equality compares logical bytes, independent of empty buffer ownership.
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for AsciiString {}

#[cfg(test)]
mod tests {
    use super::padded_byte_capacity;

    #[test]
    fn padding_includes_a_terminator_at_every_four_byte_boundary() {
        for (length, expected) in [(0, 4), (1, 4), (3, 4), (4, 8), (7, 8), (8, 12)] {
            assert_eq!(padded_byte_capacity(length), expected);
        }
    }
}
