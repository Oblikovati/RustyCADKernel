//! OCCT's static empty terminator is distinct from a heap buffer shortened to zero.
use super::ExtendedString;
pub(super) static SHARED_EMPTY_EXTENDED_TERMINATOR: [u16; 1] = [0];

pub(super) fn padded_unit_capacity(length: usize) -> usize {
    // calculatePaddedSize includes one UTF-16 terminator and rounds bytes to four.
    (length + 2) & !1
}
impl ExtendedString {
    pub(super) fn ensure_owned_terminator(&mut self) {
        if self.terminated.is_empty() {
            self.terminated = Vec::with_capacity(padded_unit_capacity(0));
            self.terminated.push(0);
        }
    }
}
impl Clone for ExtendedString {
    fn clone(&self) -> Self {
        Self::from_raw(self.as_utf16())
    }
    fn clone_from(&mut self, source: &Self) {
        self.copy(source.as_utf16());
    }
}
impl PartialEq for ExtendedString {
    fn eq(&self, other: &Self) -> bool {
        self.as_utf16() == other.as_utf16()
    }
}
impl Eq for ExtendedString {}

#[cfg(test)]
mod tests {
    use super::padded_unit_capacity;
    #[test]
    fn extended_padding_has_terminator_and_four_byte_alignment() {
        for (length, expected) in [(0, 2), (1, 2), (2, 4), (3, 4), (4, 6)] {
            assert_eq!(padded_unit_capacity(length), expected);
        }
    }
}
